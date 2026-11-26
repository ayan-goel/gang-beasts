//! Records one recipient's compact publications from the eight-fighter room profile.
#[path = "../support/combat.rs"]
mod combat;

use brawler_engine::{Consciousness, ENGINE_HASH, FighterId, MapDefinition};
use brawler_server::Room;
use std::io::{self, BufWriter, Write};

fn main() -> io::Result<()> {
    let mut room = Room::skyline_with_players(8).unwrap();
    let mut drivers = combat::Drivers::default();
    let mut out = BufWriter::new(io::stdout().lock());
    write!(out, "{ENGINE_HASH}{}", MapDefinition::tour_hash())?;
    let mut hits = 0;
    for _ in 0..(151 * 60) {
        if let Some(frames) = drivers.inputs(&room) {
            let round = room.game().match_state().unwrap().round_number;
            for (id, frame) in FighterId::ALL.into_iter().zip(frames) {
                room.submit(id, u64::from(round), &frame).unwrap();
            }
        }
        room.step();
        hits += room.game().hits().len();
        if room.snapshot_due() {
            assert!(
                FighterId::ALL
                    .into_iter()
                    .all(|id| { room.game().vitals(id).phase != Consciousness::Eliminated })
            );
            let packet = room.capture().unwrap().compact_for(Some(FighterId::First));
            out.write_all(&u32::try_from(packet.len()).unwrap().to_le_bytes())?;
            out.write_all(&packet)?;
        }
    }
    assert!(hits > 0);
    eprintln!("Recorded {} ticks with {hits} hits.", room.tick());
    out.flush()
}
