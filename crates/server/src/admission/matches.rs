use super::{Admission, connection::Connection, directory::Directory};
use crate::RoomClient;
use crate::lineup::{Lineup, Participant};
use brawler_engine::FighterId;
use std::sync::{Arc, Mutex};

pub(super) struct Match {
    pub client: RoomClient,
    pub lineup: Arc<Mutex<Lineup>>,
}

pub(super) fn welcome(players: &[Participant], fighter: Option<FighterId>) -> Vec<u8> {
    let own = fighter.map_or(255, |id| id as u8);
    let mut bytes = if players.len() == 2 {
        vec![23, 1, own]
    } else {
        vec![26, 1, own, players.len() as u8]
    };
    for p in players {
        bytes.extend([p.slot, p.color, p.name.len() as u8]);
        bytes.extend(p.name.bytes());
    }
    bytes
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
            players.push(room.participant(slot)?);
        }
        (players.len() >= 2).then_some(players)
    }
}

fn series_seed() -> Result<u32, u8> {
    // Browser physics regressions need a reproducible opening roof. Production binaries omit the override.
    #[cfg(feature = "browser-fixtures")]
    if let Ok(value) = std::env::var("BRAWLER_TEST_TOUR_SEED") {
        return value
            .parse::<std::num::NonZeroU32>()
            .map(|seed| seed.get())
            .map_err(|_| 2);
    }
    Ok(u32::from_le_bytes(super::sessions::entropy().map_err(|_| 2u8)?).max(1))
}

pub(super) async fn start(state: &Admission, code: &str, owner: Connection) -> Result<(), u8> {
    let (ticket, serial, wins) = {
        let mut dir = state.directory.lock().unwrap();
        let players = dir.starters(code, owner).ok_or(1u8)?;
        (
            players,
            dir.rooms[code].matches_started,
            dir.rooms[code].wins,
        )
    };
    let next = serial.checked_add(1).ok_or(2u8)?;
    let pool = Arc::clone(&state.pool);
    let lineup = Arc::new(Mutex::new(Lineup::new(ticket.clone())));
    let worker_lineup = lineup.clone();
    // Each private series gets an independent shuffle. Zero is reserved for Skyline fixtures.
    let seed = series_seed()?;
    let client = tokio::task::spawn_blocking(move || pool.open_hosted(seed, wins, worker_lineup))
        .await
        .map_err(|_| 2u8)?
        .map_err(|_| 2u8)?;
    // Worker initialization can overlap a leave or socket replacement. Commit only the exact ready roster.
    let mut dir = state.directory.lock().unwrap();
    if dir.starters(code, owner).as_ref() != Some(&ticket)
        || dir.rooms[code].wins != wins
        || dir.rooms[code].matches_started != serial
    {
        return Err(1);
    }
    let room = dir.rooms.get_mut(code).unwrap();
    room.matches_started = next;
    room.game = Some(Arc::new(Match { client, lineup }));
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
                    wins: 3,
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
        let former = dir.rooms[&code].participant(1).unwrap().member;
        let last = dir.rooms[&code].participant(3).unwrap().member;
        dir.leave([1; 32], &code, now).ok().unwrap();
        let players = dir.starters(&code, connection).unwrap();
        assert!(players.iter().all(|p| p.member != former));
        assert_eq!(players[2].member, last);
        let expected = [26, 1, 2, 3, 0, 7, 1, b'A', 2, 5, 1, b'C', 3, 4, 1, b'D'];
        assert_eq!(welcome(&players, Some(FighterId::Third)), expected);
        let spectator = welcome(&players, None);
        assert_eq!(spectator[2], 255);
        assert_eq!(spectator[3..], expected[3..]);
    }
}
