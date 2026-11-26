//! Release-mode service-time and snapshot-size profile for a single Skyline room.
#[cfg(test)]
mod tests;
#[path = "../support/combat.rs"]
mod workload;

use brawler_engine::{Consciousness, FighterId, Hand, MeleePhase};
use brawler_server::Room;
use serde::Serialize;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use workload::Drivers;

#[derive(Serialize)]
struct Timing {
    samples: usize,
    p50_us: u128,
    p95_us: u128,
    p99_us: u128,
    max_us: u128,
    over_tick_budget: usize,
}

impl Timing {
    fn from_samples(mut values: Vec<Duration>) -> Self {
        values.sort_unstable();
        // Nearest rank retains measured tail samples. Test overruns before rounding to microseconds.
        let percentile = |p: usize| values[(values.len() * p).div_ceil(100) - 1].as_micros();
        Self {
            samples: values.len(),
            p50_us: percentile(50),
            p95_us: percentile(95),
            p99_us: percentile(99),
            max_us: percentile(100),
            over_tick_budget: values
                .iter()
                .filter(|v| v.as_nanos() * 60 > 1_000_000_000)
                .count(),
        }
    }
}

#[derive(Serialize)]
struct Report {
    total: Timing,
    room_step: Timing,
    publication: Timing,
    playing_ticks: usize,
    eight_survivor_playing_ticks: usize,
    grip_ticks: usize,
    hits: usize,
    inputs: usize,
    snapshot_bytes: [u64; 8],
}

fn profile(warmup: usize, ticks: usize, compact: bool) -> Report {
    let mut room = Room::skyline_with_players(8).unwrap();
    let mut drivers = Drivers::default();
    let mut total = Vec::with_capacity(ticks);
    let mut steps = Vec::with_capacity(ticks);
    let mut publications = Vec::with_capacity(ticks / 2);
    let mut snapshot_bytes = [0; 8];
    let (mut playing_ticks, mut eight_survivor_playing_ticks, mut grip_ticks, mut hits, mut inputs) =
        (0, 0, 0, 0, 0);
    for index in 0..warmup + ticks {
        let measuring = index >= warmup;
        let frames = drivers.inputs(&room);
        let state = room.game().match_state().unwrap();
        let round = u64::from(state.round_number);
        let playing = state.phase == MeleePhase::Playing;
        let has_inputs = frames.is_some();
        let start = Instant::now();
        if let Some(frames) = frames {
            for (id, bytes) in FighterId::ALL.into_iter().zip(frames) {
                room.submit(id, round, &bytes).unwrap();
            }
        }
        let step_start = Instant::now();
        room.step();
        let stepped = Instant::now();
        let publish = room.snapshot_due();
        if publish {
            let shared = room.capture().unwrap();
            for id in FighterId::ALL {
                let packet = black_box(if compact {
                    shared.compact_with_landings(Some(id))
                } else {
                    shared.for_recipient(id)
                });
                if measuring {
                    snapshot_bytes[id as usize] += packet.len() as u64;
                }
            }
        }
        let end = Instant::now();
        if measuring {
            total.push(end - start);
            steps.push(stepped - step_start);
            if publish {
                publications.push(end - stepped);
            }
            if has_inputs {
                inputs += 8;
            }
            if playing {
                playing_ticks += 1;
                let game = room.game();
                if FighterId::ALL
                    .iter()
                    .all(|&id| game.vitals(id).phase != Consciousness::Eliminated)
                {
                    eight_survivor_playing_ticks += 1;
                }
                hits += game.hits().len();
                if FighterId::ALL.iter().any(|&id| {
                    [Hand::Left, Hand::Right]
                        .into_iter()
                        .any(|h| game.grip(id, h).is_some())
                }) {
                    grip_ticks += 1;
                }
            }
        }
    }
    Report {
        total: Timing::from_samples(total),
        room_step: Timing::from_samples(steps),
        publication: Timing::from_samples(publications),
        playing_ticks,
        eight_survivor_playing_ticks,
        grip_ticks,
        hits,
        inputs,
        snapshot_bytes,
    }
}

fn main() {
    let compact = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => false,
        [option] if option == "--compact" => true,
        _ => panic!("usage: room_profile [--compact]"),
    };
    for _ in 0..5 {
        let report = profile(30 * 60, 120 * 60, compact);
        assert_eq!(report.eight_survivor_playing_ticks, 7200);
        assert!(report.grip_ticks > 0 && report.hits > 0);
        println!("{}", serde_json::to_string(&report).unwrap());
    }
}
