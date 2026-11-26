use super::{directory::Directory, lobby::Lobby, sessions::SessionId};
use brawler_protocol::lobby::LobbyState;
use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) struct Connection {
    pub member: u64,
    pub session: SessionId,
    pub slot: usize,
    pub generation: u64,
}

impl Lobby {
    fn state(&self) -> LobbyState {
        let mut state = LobbyState {
            revision: self.revision,
            occupied: 0,
            ready: 0,
            host: self.host as u8,
        };
        for (i, seat) in self.members.iter().enumerate() {
            if let Some(member) = seat {
                state.occupied |= 1 << i;
                if member.ready {
                    state.ready |= 1 << i;
                }
            }
        }
        state
    }
}

impl Directory {
    pub fn connect(&mut self, code: &str, session: SessionId) -> Option<Connection> {
        self.attach_at(code, session, None, Instant::now())
    }
    pub fn resume_at(
        &mut self,
        code: &str,
        session: SessionId,
        ticket: u64,
        now: Instant,
    ) -> Option<Connection> {
        self.attach_at(code, session, Some(ticket), now)
    }
    fn attach_at(
        &mut self,
        code: &str,
        session: SessionId,
        ticket: Option<u64>,
        now: Instant,
    ) -> Option<Connection> {
        self.prune(now);
        let room = self.rooms.get_mut(code)?;
        let slot = room.slot(session)?;
        let member = room.members[slot].as_mut()?;
        // A retry may reclaim only its own disconnected seat, even after a newer tab disconnects.
        if ticket.is_some() && (member.resume_id != ticket || member.disconnected_at.is_none()) {
            return None;
        }
        self.next_connection = self.next_connection.checked_add(1)?;
        let generation = self.next_connection;
        member.resume_id = ticket.or(Some(generation));
        if let (Some(game), Some(previous)) = (&room.game, member.generation) {
            game.lineup.lock().unwrap().cancel_wait(previous);
            game.client.unbind_slot(slot, previous);
        }
        member.disconnected_at = None;
        member.generation = Some(generation);
        member.ready = false;
        Some(Connection {
            member: *member.identity.get_or_insert(generation),
            session,
            slot,
            generation,
        })
    }

    pub fn live_state(
        &mut self,
        code: &str,
        owner: Connection,
        ready: Option<bool>,
    ) -> Option<LobbyState> {
        let now = Instant::now();
        let last = self.sessions.0.get_mut(&owner.session)?;
        if now.duration_since(*last) >= super::sessions::IDLE {
            return None;
        }
        let room = self.rooms.get_mut(code)?;
        let member = room.members.get_mut(owner.slot)?.as_mut()?;
        if member.session != owner.session || member.generation != Some(owner.generation) {
            return None;
        }
        if let Some(value) = ready {
            member.ready = value;
            *last = now;
        }
        Some(room.state())
    }

    pub fn disconnect(&mut self, code: &str, owner: Connection) {
        self.disconnect_at(code, owner, Instant::now());
    }
    pub fn disconnect_at(&mut self, code: &str, owner: Connection, now: Instant) {
        let Some(member) = self
            .rooms
            .get_mut(code)
            .and_then(|r| r.members[owner.slot].as_mut())
        else {
            return;
        };
        // A replaced tab can close after its successor is ready.
        if member.session == owner.session && member.generation == Some(owner.generation) {
            member.ready = false;
            member.generation = None;
            member.disconnected_at = Some(now);
        }
    }
}
