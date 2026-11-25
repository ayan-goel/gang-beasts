use super::{Admission, connection::Connection, directory::Directory, sessions::SessionId};
use crate::RoomClient;
use brawler_engine::FighterId;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
struct Participant {
    session: SessionId,
    generation: u64,
    slot: u8,
    name: String,
    color: u8,
}

pub(super) struct Match {
    pub client: RoomClient,
    players: [Participant; 2],
}

impl Match {
    pub fn fighter(&self, session: SessionId) -> Option<FighterId> {
        self.players
            .iter()
            .position(|p| p.session == session)
            .map(|i| [FighterId::First, FighterId::Second][i])
    }

    pub fn welcome(&self, session: SessionId) -> Vec<u8> {
        let mut bytes = vec![23, 1, self.fighter(session).map_or(255, |id| id as u8)];
        for p in &self.players {
            bytes.extend([p.slot, p.color, p.name.len() as u8]);
            bytes.extend(p.name.bytes());
        }
        bytes
    }
}

impl Directory {
    fn starters(&mut self, code: &str, owner: Connection) -> Option<[Participant; 2]> {
        self.live_state(code, owner, None)?;
        let room = self.rooms.get(code)?;
        if room.host != owner.slot || room.game.is_some() {
            return None;
        }
        let mut players = Vec::with_capacity(2);
        for (slot, member) in room.members.iter().enumerate() {
            let Some(member) = member else { continue };
            if !member.ready || players.len() == 2 {
                return None;
            }
            let seat = room.seat(slot);
            players.push(Participant {
                session: member.session,
                generation: member.generation?,
                slot: seat.slot,
                name: seat.name,
                color: seat.color,
            });
        }
        players.try_into().ok()
    }
}

pub(super) async fn start(state: &Admission, code: &str, owner: Connection) -> Result<(), u8> {
    let ticket = state
        .directory
        .lock()
        .unwrap()
        .starters(code, owner)
        .ok_or(1u8)?;
    let pool = Arc::clone(&state.pool);
    let client = tokio::task::spawn_blocking(move || pool.open())
        .await
        .map_err(|_| 2u8)?
        .map_err(|_| 2u8)?;
    // Worker initialization can overlap a leave or socket replacement. Commit only the exact ready roster.
    let mut dir = state.directory.lock().unwrap();
    if dir.starters(code, owner).as_ref() != Some(&ticket) {
        return Err(1);
    }
    dir.rooms.get_mut(code).unwrap().game = Some(Arc::new(Match {
        client,
        players: ticket,
    }));
    Ok(())
}
