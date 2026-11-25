use super::types::Fault;
use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub(super) type SessionId = [u8; 32];
pub(super) const IDLE: Duration = Duration::from_secs(30 * 60);
pub(super) const COOKIE: &str = "__Host-skyline";
pub(super) const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub(super) fn entropy<const N: usize>() -> Result<[u8; N], Fault> {
    let mut b = [0; N];
    getrandom::getrandom(&mut b).map_err(|_| Fault::SERVICE)?;
    Ok(b)
}

#[derive(Default)]
pub(super) struct Sessions(pub HashMap<SessionId, Instant>);

impl Sessions {
    pub fn issue(&mut self, now: Instant) -> Result<String, Fault> {
        if self.0.len() >= 512 {
            return Err(Fault::SERVICE);
        }
        let random = entropy::<32>()?;
        let raw: String = random.iter().map(|b| format!("{b:02x}")).collect();
        // Retain only a digest. A room code and public roster never confer a player's authority.
        let id = Sha256::digest(raw.as_bytes()).into();
        if self.0.contains_key(&id) {
            return Err(Fault::SERVICE);
        }
        self.0.insert(id, now);
        Ok(format!(
            "{COOKIE}={raw}; Path=/; HttpOnly; Secure; SameSite=Strict"
        ))
    }
    pub fn authenticate(&mut self, headers: &HeaderMap, now: Instant) -> Result<SessionId, Fault> {
        let mut found = None;
        for value in headers.get_all("cookie") {
            let cookies = value.to_str().map_err(|_| Fault::UNAUTHORIZED)?;
            for item in cookies.split(';') {
                let Some((name, value)) = item.trim().split_once('=') else {
                    continue;
                };
                if name != COOKIE {
                    continue;
                }
                if found.is_some()
                    || value.len() != 64
                    || !value.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(Fault::UNAUTHORIZED);
                }
                found = Some(SessionId::from(Sha256::digest(value.as_bytes())));
            }
        }
        let id = found.ok_or(Fault::UNAUTHORIZED)?;
        let last = self.0.get_mut(&id).ok_or(Fault::UNAUTHORIZED)?;
        if now.duration_since(*last) >= IDLE {
            return Err(Fault::UNAUTHORIZED);
        }
        *last = now;
        Ok(id)
    }
}
