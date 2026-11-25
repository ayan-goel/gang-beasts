use super::{
    sessions::SessionId,
    types::{Fault, PublicRoom, Seat},
};
use axum::http::StatusCode;
use std::time::Instant;

pub(super) struct Member {
    pub session: SessionId,
    name: String,
    color: u8,
}
pub(super) struct Lobby {
    pub code: String,
    pub request: String,
    pub creator: SessionId,
    pub capacity: u8,
    pub members: Vec<Option<Member>>,
    pub host: usize,
    pub empty_since: Option<Instant>,
    original_name: String,
    original_color: Option<u8>,
}

impl Lobby {
    pub fn new(
        code: String,
        request: String,
        session: SessionId,
        capacity: u8,
        name: &str,
        color: Option<u8>,
    ) -> Self {
        let mut members: Vec<_> = (0..capacity).map(|_| None).collect();
        members[0] = Some(Member {
            session,
            name: name.to_owned(),
            color: color.unwrap_or(0),
        });
        Self {
            code,
            request,
            creator: session,
            capacity,
            members,
            host: 0,
            empty_since: None,
            original_name: name.to_owned(),
            original_color: color,
        }
    }
    pub fn same_request(&self, capacity: u8, name: &str, color: Option<u8>) -> bool {
        self.capacity == capacity && self.original_name == name && self.original_color == color
    }
    pub fn metadata(&self) -> PublicRoom {
        PublicRoom {
            code: self.code.clone(),
            phase: "lobby",
            capacity: self.capacity,
            players: self.members.iter().flatten().count(),
        }
    }
    pub fn update_host(&mut self, now: Instant) {
        if self.members[self.host].is_some() {
            return;
        }
        if let Some(next) = self.members.iter().position(Option::is_some) {
            self.host = next;
        } else {
            self.empty_since.get_or_insert(now);
        }
    }

    pub fn slot(&self, session: SessionId) -> Option<usize> {
        self.members
            .iter()
            .position(|m| m.as_ref().is_some_and(|m| m.session == session))
    }
    pub fn seat(&self, slot: usize) -> Seat {
        let member = self.members[slot].as_ref().expect("occupied session slot");
        Seat {
            code: self.code.clone(),
            slot: slot as u8,
            host: self.host == slot,
            name: member.name.clone(),
            color: member.color,
        }
    }
    pub fn join(
        &mut self,
        session: SessionId,
        name: &str,
        color: Option<u8>,
    ) -> Result<Seat, Fault> {
        if let Some(slot) = self.slot(session) {
            let seat = self.seat(slot);
            return if seat.name == name && color.is_none_or(|c| c == seat.color) {
                Ok(seat)
            } else {
                Err(Fault::CONFLICT)
            };
        }
        let slot = self
            .members
            .iter()
            .position(Option::is_none)
            .ok_or(Fault(StatusCode::CONFLICT, "room_full"))?;
        let used = |c: u8| self.members.iter().flatten().any(|m| m.color == c);
        let color = match color {
            Some(c) if used(c) => return Err(Fault(StatusCode::CONFLICT, "color_taken")),
            Some(c) => c,
            None => (0..8)
                .find(|&c| !used(c))
                .expect("an open slot has an available color"),
        };
        if self.empty_since.take().is_some() {
            self.host = slot;
        }
        self.members[slot] = Some(Member {
            session,
            name: name.to_owned(),
            color,
        });
        Ok(self.seat(slot))
    }
}
