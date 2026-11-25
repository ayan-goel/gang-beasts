use brawler_engine::FighterId;
use brawler_protocol::{Button, InputFrame};
use brawler_server::{DispatchError, WorkerPool};
use std::{
    thread,
    time::{Duration, Instant},
};

fn wait_for(mut check: impl FnMut() -> bool) {
    let end = Instant::now() + Duration::from_secs(8);
    while !check() {
        assert!(Instant::now() < end, "worker did not reach expected state");
        thread::sleep(Duration::from_millis(3));
    }
}

#[test]
fn bounded_worker_publishes_live_inputs_and_recycles_the_slot_for_a_fresh_room() {
    let pool = WorkerPool::new(1).unwrap();
    let room = pool.open().unwrap();
    let coral = FighterId::First;
    assert!(matches!(pool.open(), Err(DispatchError::Capacity)));
    let initial = room.latest().unwrap();
    assert_eq!(initial.for_recipient(coral)[4..8], 0u32.to_le_bytes());
    wait_for(|| room.latest().unwrap().tick() >= 180);
    let bit = Button::LeftHand as u16;
    let input = InputFrame {
        sequence: 42,
        pressed: bit,
        released: bit,
        ..Default::default()
    }
    .encode()
    .unwrap();
    room.input(coral, 1, &input).unwrap();
    wait_for(|| room.latest().unwrap().for_recipient(coral)[8..12] == 42u32.to_le_bytes());
    let latest = room.latest().unwrap();
    assert_eq!(latest.tick() % 2, 0);
    assert_eq!(latest.for_recipient(FighterId::Second)[2], 1);
    assert_eq!(initial.tick(), 0);
    drop(room);
    wait_for(|| pool.available() == 1);
    let replacement = pool.open().unwrap();
    assert_eq!(replacement.latest().unwrap().for_recipient(coral)[2], 1);
    assert!(replacement.latest().unwrap().tick() < 180);
    drop(pool);
    assert_eq!(
        replacement.input(coral, 1, &input),
        Err(DispatchError::Closed)
    );
}

#[test]
fn two_workers_advance_independently_when_one_room_is_released() {
    let pool = WorkerPool::new(2).unwrap();
    let first = pool.open().unwrap();
    let second = pool.open().unwrap();
    wait_for(|| first.latest().unwrap().tick() >= 4 && second.latest().unwrap().tick() >= 4);
    drop(first);
    let tick = second.latest().unwrap().tick();
    wait_for(|| pool.available() == 1 && second.latest().unwrap().tick() > tick);
    assert!(WorkerPool::new(0).is_err());
}

#[test]
fn eight_worker_inputs_are_acknowledged_only_to_their_owners_in_one_shared_world() {
    let pool = WorkerPool::new(1).unwrap();
    for invalid in [0, 1, 9, usize::MAX] {
        assert!(pool.open_with_players(invalid).is_err());
        assert_eq!(pool.available(), 1);
    }
    let room = pool.open_with_players(8).unwrap();
    wait_for(|| room.latest().unwrap().tick() >= 180);
    for id in FighterId::ALL {
        let bytes = InputFrame {
            sequence: 101 + id as u32,
            ..Default::default()
        }
        .encode()
        .unwrap();
        room.input(id, 1, &bytes).unwrap();
    }
    wait_for(|| {
        let frame = room.latest().unwrap();
        FighterId::ALL
            .iter()
            .all(|&id| frame.for_recipient(id)[8..12] == (101 + id as u32).to_le_bytes())
    });
    let frame = room.latest().unwrap();
    let shared = frame.for_spectator();
    let count = u16::from_le_bytes(shared[16..18].try_into().unwrap()) as usize;
    assert_eq!(count, 8 * 15 + 9);
    assert_eq!(shared[20 + count * 32 + 3], 8);
    for id in FighterId::ALL {
        assert_eq!(frame.for_recipient(id)[12..], shared[12..]);
        for limb in 0..15 {
            let offset = 20 + (id as usize * 15 + limb) * 32;
            assert_eq!(
                shared[offset..offset + 4],
                (1 + id as u32 * 16 + limb as u32).to_le_bytes()
            );
        }
    }
    assert_eq!(shared[2], 1);
}
