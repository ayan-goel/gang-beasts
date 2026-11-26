use super::{
    lobby::Lobby,
    sessions::{ALPHABET, IDLE, SessionId, Sessions, entropy},
    types::{Create, Fault, Join, PublicRoom, Seat, player},
};
use axum::http::StatusCode;
use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct Directory {
    pub sessions: Sessions,
    pub tickets: super::socket_ticket::Tickets,
    pub(super) next_connection: u64,
    pub(super) rooms: HashMap<String, Lobby>,
    completed: HashMap<SessionId, String>,
    traffic: HashMap<IpAddr, (Instant, u16)>,
    last_sweep: Option<Instant>,
}

impl Directory {
    pub fn allow(&mut self, ip: IpAddr, now: Instant) -> Result<(), Fault> {
        if self
            .last_sweep
            .is_none_or(|t| now.duration_since(t) >= Duration::from_secs(1))
        {
            self.prune(now);
            self.last_sweep = Some(now);
        }
        if !self.traffic.contains_key(&ip) && self.traffic.len() >= 1024 {
            return Err(Fault::SERVICE);
        }
        let bucket = self.traffic.entry(ip).or_insert((now, 0));
        if now.duration_since(bucket.0) >= Duration::from_secs(60) {
            *bucket = (now, 0);
        }
        if bucket.1 >= 120 {
            return Err(Fault(StatusCode::TOO_MANY_REQUESTS, "rate_limited"));
        }
        bucket.1 += 1;
        Ok(())
    }
    pub(super) fn prune(&mut self, now: Instant) {
        self.traffic
            .retain(|_, (t, _)| now.duration_since(*t) < Duration::from_secs(60));
        self.sessions.0.retain(|_, t| now.duration_since(*t) < IDLE);
        self.completed
            .retain(|id, _| self.sessions.0.contains_key(id));
        self.rooms.retain(|_, r| {
            // Retire every seat together when the champion display ends, including disconnected seats.
            if r.game
                .as_ref()
                .is_some_and(|game| game.client.series_complete())
            {
                for member in r.members.iter().flatten() {
                    self.completed.insert(member.session, r.code.clone());
                }
                return false;
            }
            let mut released = 0;
            for (slot, member) in r.members.iter().enumerate() {
                if member.as_ref().is_some_and(|m| {
                    !self.sessions.0.contains_key(&m.session)
                        || m.disconnected_at.is_some_and(|since| {
                            now.saturating_duration_since(since) >= super::maintenance::GRACE
                        })
                }) {
                    released |= 1 << slot;
                }
            }
            r.release_seats(released, now);
            r.empty_since
                .is_none_or(|t| now.duration_since(t) < Duration::from_secs(60))
        });
    }
    pub fn create(&mut self, session: SessionId, req: Create) -> Result<(StatusCode, Seat), Fault> {
        let name = player(&req.name, req.color)?;
        if !(2..=8).contains(&req.capacity)
            || !matches!(req.wins, 3 | 5 | 7)
            || req.request_id.len() != 32
            || !req.request_id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Fault::INVALID);
        }
        for room in self.rooms.values() {
            if room.creator == session
                && room.request == req.request_id
                && room.slot(session).is_none()
            {
                return Err(Fault::CONFLICT);
            }
            if let Some(slot) = room.slot(session) {
                if room.creator == session
                    && room.request == req.request_id
                    && room.same_request(req.capacity, req.wins, name, req.color)
                {
                    return Ok((StatusCode::OK, room.seat(slot)));
                }
                return Err(Fault::CONFLICT);
            }
        }
        if self.rooms.len() >= 64 {
            return Err(Fault::SERVICE);
        }
        // Six uniform base32 characters identify a room. Retry collisions without replacing another room.
        for _ in 0..8 {
            let code: String = entropy::<6>()?
                .map(|b| ALPHABET[(b & 31) as usize] as char)
                .into_iter()
                .collect();
            if self.rooms.contains_key(&code) {
                continue;
            }
            let room = Lobby::new(
                code.clone(),
                req.request_id,
                session,
                req.capacity,
                req.wins,
                name,
                req.color,
            );
            let seat = room.seat(0);
            self.rooms.insert(code, room);
            return Ok((StatusCode::CREATED, seat));
        }
        Err(Fault::SERVICE)
    }
    pub fn join(&mut self, session: SessionId, code: &str, req: Join) -> Result<Seat, Fault> {
        self.prune(Instant::now());
        let name = player(&req.name, req.color)?;
        if self
            .rooms
            .values()
            .any(|r| r.code != code && r.slot(session).is_some())
        {
            return Err(Fault::CONFLICT);
        }
        self.rooms
            .get_mut(code)
            .ok_or(Fault::MISSING)?
            .join(session, name, req.color)
    }
    pub fn last_completed(&self, session: SessionId) -> Option<String> {
        self.completed
            .get(&session)
            .filter(|code| !self.rooms.contains_key(*code))
            .cloned()
    }

    pub fn current(&self, session: SessionId) -> Option<String> {
        self.rooms
            .values()
            .find(|r| r.slot(session).is_some())
            .map(|r| r.code.clone())
    }

    pub fn roster(&self, session: SessionId, code: &str) -> Result<super::types::Roster, Fault> {
        let r = self.rooms.get(code).ok_or_else(|| {
            if self
                .completed
                .get(&session)
                .is_some_and(|finished| finished == code)
            {
                Fault(StatusCode::GONE, "series_complete")
            } else {
                Fault::MISSING
            }
        })?;
        let you = r
            .slot(session)
            .ok_or(Fault(StatusCode::FORBIDDEN, "not_a_member"))?;
        Ok(super::types::Roster {
            code: r.code.clone(),
            capacity: r.capacity,
            wins: r.wins,
            you: you as u8,
            revision: r.revision,
            players: r
                .members
                .iter()
                .enumerate()
                .filter_map(|(i, m)| m.as_ref().map(|_| r.seat(i)))
                .collect(),
        })
    }

    pub fn leave(&mut self, session: SessionId, code: &str, now: Instant) -> Result<(), Fault> {
        let r = self.rooms.get_mut(code).ok_or(Fault::MISSING)?;
        if let Some(slot) = r.slot(session) {
            r.release_seats(1 << slot, now);
        }
        Ok(())
    }
    pub fn metadata(&self, code: &str) -> Result<PublicRoom, Fault> {
        Ok(self.rooms.get(code).ok_or(Fault::MISSING)?.metadata())
    }
}
