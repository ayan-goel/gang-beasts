use super::{directory::Directory, lobby::Lobby, sessions::SessionId};
use brawler_protocol::lobby::LobbyState;
use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) struct Connection {
    pub session: SessionId,
    pub slot: usize,
    generation: u64,
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
        let room = self.rooms.get_mut(code)?;
        let slot = room.slot(session)?;
        self.next_connection = self.next_connection.checked_add(1)?;
        let generation = self.next_connection;
        let member = room.members[slot].as_mut()?;
        member.generation = Some(generation);
        member.ready = false;
        Some(Connection {
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
        }
    }
}
