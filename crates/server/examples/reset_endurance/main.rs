//! Advance production Skyline rooms through consecutive rounds including the scheduled hazards.
mod checks;
#[path = "../support/combat.rs"]
mod combat;
mod driver;
#[cfg(test)]
mod tests;

use brawler_engine::{Consciousness, FighterId, MeleePhase, SkylineEvent};
use brawler_server::Room;
use serde::Serialize;

#[derive(Default, Serialize)]
struct Report {
    round_resets: u32,
    fresh_matches: u32,
    scored_rounds: u32,
    draws: u32,
    elapsed_ticks: u64,
    playing_ticks: u64,
    inputs: u64,
    hits: u64,
    grip_ticks: u64,
    detached_panels: u64,
    gusts: u64,
    verified_snapshots: u64,
}

fn run(required_resets: u32) -> Report {
    let mut room = Room::skyline_with_players(8).unwrap();
    let spawn = checks::Spawn::capture(&room);
    let mut drivers = combat::Drivers::default();
    let mut report = Report::default();
    let mut scores = [0; 8];
    let mut check_epoch = true;

    while report.round_resets < required_resets {
        let state = room.game().match_state().unwrap();
        let phase = state.phase;
        let round = state.round_number;
        let ordinal = report.scored_rounds + report.draws;
        let sent = if phase == MeleePhase::Playing {
            driver::submit(&mut room, &mut drivers, ordinal, round, &mut check_epoch)
        } else {
            None
        };
        room.step();
        report.elapsed_ticks += 1;
        if let Some(sequence) = sent {
            report.inputs += 8;
            assert!(
                FighterId::ALL
                    .iter()
                    .all(|&id| room.applied(id) == Some(sequence))
            );
        }
        if phase == MeleePhase::Playing {
            report.playing_ticks += 1;
            report.hits += room.game().hits().len() as u64;
            report.grip_ticks += u64::from(checks::verify_motion(&room));
            for event in room.game().arena_events() {
                match event {
                    SkylineEvent::PanelDetached(_) => report.detached_panels += 1,
                    SkylineEvent::GustStarted => report.gusts += 1,
                    SkylineEvent::EdgeWarning => (),
                }
            }
        }
        if room.snapshot_due() {
            // The production encoder checks poses, scalar state, entity order and grip references.
            room.capture().expect("valid published world");
            report.verified_snapshots += 1;
        }
        let state = room.game().match_state().unwrap();
        if phase == MeleePhase::Playing && state.phase == MeleePhase::RoundResults {
            let alive: Vec<_> = FighterId::ALL
                .into_iter()
                .filter(|&id| room.game().vitals(id).phase != Consciousness::Eliminated)
                .collect();
            if let Some(winner) = state.winner {
                assert_eq!(alive.as_slice(), &[FighterId::ALL[winner as usize]]);
                scores[winner as usize] += 1;
                report.scored_rounds += 1;
            } else {
                assert!(alive.len() != 1);
                report.draws += 1;
            }
        }
        assert_eq!(state.scores(), scores);
        if state.round_number != round {
            spawn.verify(&room, round + 1, &scores);
            report.round_resets += 1;
            if report.round_resets == 1 || report.round_resets.is_multiple_of(10) {
                eprintln!(
                    "reset {} after {} ticks",
                    report.round_resets, report.elapsed_ticks
                );
            }
        } else if state.phase == MeleePhase::MatchResults {
            assert!(scores.contains(&3));
            room = Room::skyline_with_players(8).unwrap();
            scores = [0; 8];
            spawn.verify(&room, 1, &scores);
            report.fresh_matches += 1;
        } else {
            continue;
        }
        drivers = combat::Drivers::default();
        check_epoch = true;
    }
    report
}

fn main() {
    let report = run(100);
    assert!(
        report.gusts >= 1
            && report.detached_panels >= 2
            && report.hits > 0
            && report.grip_ticks > 0
    );
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
