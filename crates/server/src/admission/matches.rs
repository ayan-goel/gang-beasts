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
    players: Vec<Participant>,
}

impl Match {
    pub fn fighter(&self, session: SessionId) -> Option<FighterId> {
        self.players
            .iter()
            .position(|p| p.session == session)
            .map(|i| FighterId::ALL[i])
            .filter(|&id| !self.client.forfeited(id))
    }

    pub fn welcome(&self, session: SessionId) -> Vec<u8> {
        let own = self.fighter(session).map_or(255, |id| id as u8);
        let mut bytes = if self.players.len() == 2 {
            vec![23, 1, own]
        } else {
            vec![26, 1, own, self.players.len() as u8]
        };
        for p in &self.players {
            bytes.extend([p.slot, p.color, p.name.len() as u8]);
            bytes.extend(p.name.bytes());
        }
        bytes
    }
}

impl Directory {
    fn starters(&mut self, code: &str, owner: Connection) -> Option<Vec<Participant>> {
        self.live_state(code, owner, None)?;
        let room = self.rooms.get(code)?;
        if room.host != owner.slot || room.game.is_some() {
            return None;
        }
        let mut players = Vec::with_capacity(room.members.len());
        for (slot, member) in room.members.iter().enumerate() {
            let Some(member) = member else { continue };
            if !member.ready {
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
        (players.len() >= 2).then_some(players)
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
    let count = ticket.len();
    let client = tokio::task::spawn_blocking(move || pool.open_with_players(count))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::types::{Create, Join};
    use std::time::Instant;

    #[test]
    fn compact_assignments_preserve_sparse_seats_and_exclude_a_former_member() {
        let mut dir = Directory::default();
        let now = Instant::now();
        for slot in 0..4u8 {
            dir.sessions.0.insert([slot; 32], now);
        }
        let (_, host) = dir
            .create(
                [0; 32],
                Create {
                    request_id: "c".repeat(32),
                    name: "A".into(),
                    capacity: 8,
                    color: Some(7),
                },
            )
            .ok()
            .unwrap();
        let code = host.code;
        let connection = dir.connect(&code, [0; 32]).unwrap();
        dir.live_state(&code, connection, Some(true)).unwrap();
        for slot in 1..4u8 {
            dir.join(
                [slot; 32],
                &code,
                Join {
                    name: char::from(b'A' + slot).to_string(),
                    color: Some(7 - slot),
                },
            )
            .ok()
            .unwrap();
            let member = dir.connect(&code, [slot; 32]).unwrap();
            dir.live_state(&code, member, Some(true)).unwrap();
        }
        dir.leave([1; 32], &code, now).ok().unwrap();
        let players = dir.starters(&code, connection).unwrap();
        let pool = crate::WorkerPool::new(1).unwrap();
        let game = Match {
            client: pool.open_with_players(players.len()).unwrap(),
            players,
        };
        let expected = [26, 1, 2, 3, 0, 7, 1, b'A', 2, 5, 1, b'C', 3, 4, 1, b'D'];
        assert_eq!(game.welcome([3; 32]), expected);
        let spectator = game.welcome([1; 32]);
        assert_eq!(spectator[2], 255);
        assert_eq!(spectator[3..], expected[3..]);
    }
}
