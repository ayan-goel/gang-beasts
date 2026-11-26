use super::{
    Admission, body,
    sessions::{SessionId, entropy},
    types::{Empty, Fault},
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

const TTL: Duration = Duration::from_secs(15);
pub(super) const PROTOCOL: &str = "skyline.v1";

#[derive(Default)]
pub(super) struct Tickets(HashMap<[u8; 32], (SessionId, String, Instant)>);

impl Tickets {
    fn mint(&mut self, session: SessionId, room: String, now: Instant) -> Result<String, Fault> {
        // One outstanding upgrade per session bounds storage and invalidates abandoned attempts.
        self.0
            .retain(|_, (owner, _, expiry)| *owner != session && *expiry > now);
        let ticket: String = entropy::<32>()?
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        self.0.insert(
            Sha256::digest(ticket.as_bytes()).into(),
            (session, room, now + TTL),
        );
        Ok(ticket)
    }

    pub fn redeem(&mut self, protocols: &str, now: Instant) -> Result<(SessionId, String), Fault> {
        let mut offered = protocols.split(',').map(str::trim);
        if offered.next() != Some(PROTOCOL) {
            return Err(Fault::UNAUTHORIZED);
        }
        let raw = offered
            .next()
            .and_then(|p| p.strip_prefix("ticket."))
            .ok_or(Fault::UNAUTHORIZED)?;
        if offered.next().is_some()
            || raw.len() != 64
            || !raw.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Fault::UNAUTHORIZED);
        }
        let key: [u8; 32] = Sha256::digest(raw.as_bytes()).into();
        let (owner, room, expiry) = self.0.remove(&key).ok_or(Fault::UNAUTHORIZED)?;
        if expiry <= now {
            return Err(Fault::UNAUTHORIZED);
        }
        Ok((owner, room))
    }
}

#[derive(Serialize)]
pub(super) struct Grant {
    ticket: String,
}

pub(super) async fn issue(
    State(state): State<Admission>,
    headers: HeaderMap,
    data: Result<Json<Empty>, JsonRejection>,
) -> Result<Json<Grant>, Fault> {
    body(data)?;
    let mut dir = state.directory.lock().unwrap();
    let now = Instant::now();
    let session = dir.sessions.authenticate(&headers, now)?;
    let room = dir
        .current(session)
        .ok_or(Fault(StatusCode::FORBIDDEN, "not_a_member"))?;
    Ok(Json(Grant {
        ticket: dir.tickets.mint(session, room, now)?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_is_single_use_expires_and_replaces_abandoned_attempts() {
        let now = Instant::now();
        let mut tickets = Tickets::default();
        let issue = |store: &mut Tickets| {
            let raw = store.mint([3; 32], "ABCDEF".into(), now).ok().unwrap();
            format!("{PROTOCOL}, ticket.{raw}")
        };
        let stale = issue(&mut tickets);
        let valid = issue(&mut tickets);
        assert!(tickets.redeem(&stale, now).is_err());
        assert_eq!(
            tickets.redeem(&valid, now).ok(),
            Some(([3; 32], "ABCDEF".into()))
        );
        assert!(tickets.redeem(&valid, now).is_err());
        let expired = issue(&mut tickets);
        assert!(tickets.redeem(&expired, now + TTL).is_err());
    }
}
