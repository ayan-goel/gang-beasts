use super::combat::Drivers;
use brawler_engine::FighterId;
use brawler_protocol::{Button, INPUT_BYTES, InputFrame};
use brawler_server::{Room, SubmitError};

fn exit_inputs(room: &Room, survivor: usize) -> Option<[[u8; INPUT_BYTES]; 8]> {
    if !room.tick().is_multiple_of(6) {
        return None;
    }
    Some(FighterId::ALL.map(|id| {
        let fleeing = id as usize != survivor;
        InputFrame {
            sequence: (room.tick() / 6) as u32,
            tick: room.tick() as u32,
            axes: if fleeing { [127, 0] } else { [0, 0] },
            held: if fleeing { Button::Run as u16 } else { 0 },
            ..Default::default()
        }
        .encode()
        .unwrap()
    }))
}

pub fn submit(
    room: &mut Room,
    drivers: &mut Drivers,
    ordinal: u32,
    round: u32,
    check_epoch: &mut bool,
) -> Option<u32> {
    if *check_epoch {
        let bytes = InputFrame::default().encode().unwrap();
        assert_eq!(
            room.submit(FighterId::First, u64::from(round - 1), &bytes),
            Err(SubmitError::Round)
        );
        *check_epoch = false;
    }
    // Every 25th round keeps the central fight running into the scheduled collapses and wind.
    let fight = room.game().tick() < 360 || ordinal % 25 == 24;
    let frames = if fight {
        drivers.inputs(room)
    } else {
        exit_inputs(room, ordinal as usize % 8)
    }?;
    for (id, bytes) in FighterId::ALL.into_iter().zip(frames) {
        room.submit(id, u64::from(round), &bytes).unwrap();
    }
    Some((room.tick() / 6) as u32)
}
