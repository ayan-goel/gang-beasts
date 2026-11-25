use super::{
    sessions::SessionId,
    types::{Fault, PublicRoom, Seat},
};
use axum::http::StatusCode;
use std::time::Instant;

pub(super) struct Member {
    pub session: SessionId,
    pub ready: bool,
    pub generation: Option<u64>,
    pub disconnected_at: Option<Instant>,
    name: String,
    color: u8,
}
pub(super) struct Lobby {
    pub game: Option<std::sync::Arc<super::matches::Match>>,
    pub code: String,
    pub request: String,
    pub creator: SessionId,
    pub capacity: u8,
    pub members: Vec<Option<Member>>,
    pub host: usize,
    pub revision: u32,
    pub matches_started: u32,
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
            ready: false,
            generation: None,
            disconnected_at: None,
            name: name.to_owned(),
            color: color.unwrap_or(0),
        });
        Self {
            game: None,
            code,
            request,
            creator: session,
            capacity,
            members,
            host: 0,
            revision: 1,
            matches_started: 0,
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
            phase: if self.game.is_some() {
                "match"
            } else {
                "lobby"
            },
            capacity: self.capacity,
            players: self.members.iter().flatten().count(),
        }
    }
    pub fn release_seats(&mut self, slots: u8, now: Instant) {
        let mut forfeits = 0;
        for (slot, seat) in self.members.iter_mut().enumerate() {
            if slots & (1 << slot) == 0 {
                continue;
            }
            let Some(member) = seat.take() else { continue };
            if let Some((game, id)) = self
                .game
                .as_ref()
                .and_then(|g| g.fighter(member.session).map(|id| (g, id)))
            {
                forfeits |= 1 << id as u8;
                if let Some(generation) = member.generation {
                    game.client.unbind(id, generation);
                }
            }
            self.revision = self.revision.wrapping_add(1);
        }
        // The worker samples all expirations from this sweep together before resolving the winner.
        if let Some(game) = &self.game {
            if forfeits != 0 {
                game.client.forfeit(forfeits);
            }
            if game.client.returning_to_lobby() {
                self.return_to_lobby();
            }
        }
        self.update_host(now);
    }

    pub fn return_to_lobby(&mut self) {
        self.game = None;
        for member in self.members.iter_mut().flatten() {
            member.ready = false;
        }
    }

    fn update_host(&mut self, now: Instant) {
        if self.members[self.host].is_some() {
            return;
        }
        let next = self
            .members
            .iter()
            .position(|seat| seat.as_ref().is_some_and(|m| m.generation.is_some()))
            .or_else(|| self.members.iter().position(Option::is_some));
        if let Some(next) = next {
            self.host = next;
        } else {
            self.game = None;
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
        self.revision = self.revision.wrapping_add(1);
        self.members[slot] = Some(Member {
            session,
            ready: false,
            generation: None,
            disconnected_at: None,
            name: name.to_owned(),
            color,
        });
        Ok(self.seat(slot))
    }
}
