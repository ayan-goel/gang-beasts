use brawler_protocol::snapshot::{BodyPose, FighterState, Snapshot};

fn sample() -> Snapshot {
    let mut fighters = vec![FighterState::default(); 2];
    fighters[0] = FighterState {
        consciousness: 0,
        footstep: None,
        landing: None,
        score: 1,
        hands: [5, 4],
        health: 76.5,
        stamina: 41.25,
        grips: [65543, 0],
    };
    fighters[1] = FighterState {
        consciousness: 2,
        score: 2,
        health: 60.,
        stamina: 100.,
        ..Default::default()
    };
    let bodies = (1..=15)
        .chain(17..=31)
        .chain([65543])
        .map(|id| BodyPose {
            id,
            position: [id as f32 / 8., -1.25, 3.],
            rotation: [0., 0., 0., 1.],
        })
        .collect();
    Snapshot {
        tick: 0xfffffff0,
        applied: Some(0),
        round: 0x20000000000001,
        number: 3,
        arena_tick: 7201,
        phase: 1,
        winner: None,
        remaining: 0,
        bodies,
        fighters,
    }
}

#[test]
fn snapshot_output_matches_the_independent_wire_fixture() {
    assert_eq!(
        sample().encode().unwrap(),
        include_bytes!("../../../tests/fixtures/protocol/full-skyline-v1.bin").as_slice()
    );
}

#[test]
fn invalid_rotation_missing_bodies_and_stale_grips_fail_before_publication() {
    let mut s = sample();
    s.bodies[0].rotation = [0.; 4];
    assert!(s.encode().is_err());
    let mut s = sample();
    s.bodies.pop();
    assert!(s.encode().is_err());
    let mut s = sample();
    s.bodies.remove(2);
    assert!(s.encode().is_err());
    let mut s = sample();
    s.fighters[0].grips[0] = 1;
    assert!(s.encode().is_err());
}

#[test]
fn compact_frames_match_the_golden_record_and_keep_recipient_acks_independent() {
    let mut s = sample();
    let rotations = [
        [0., 0., 0., -1.],
        [-0.5, 0.5, -0.5, 0.5],
        [0., -0.8, 0., 0.6],
        [0.6, 0., 0.8, 0.],
    ];
    for (index, body) in s.bodies.iter_mut().enumerate() {
        body.rotation = rotations[index % rotations.len()];
    }
    let shared = s.encode_shared().unwrap();
    let packet = shared.compact_for_recipient(Some(0));
    assert_eq!(
        packet,
        include_bytes!("../../../tests/fixtures/protocol/compact-skyline-v1.bin").as_slice()
    );
    let spectator = shared.compact_for_recipient(None);
    assert_eq!(spectator[2], 5);
    assert_eq!(
        shared.compact_for_recipient(Some(42))[8..12],
        42u32.to_le_bytes()
    );
    assert_eq!(packet[12..], spectator[12..]);
}

#[test]
fn opted_in_landings_share_one_packet_size_with_legacy_frames() {
    let mut s = sample();
    s.fighters[0].landing = Some((7100, 55));
    let shared = s.encode_shared().unwrap();
    let legacy = shared.compact_for_recipient(None);
    let current = shared.compact_with_landings(None);
    let block = 20 + 23 * s.bodies.len();
    assert_eq!(legacy.len(), current.len());
    assert_eq!(legacy[block], 1);
    assert_eq!(current[block], 2);
    assert_eq!(&legacy[block + 29..block + 32], &[0, 0, 0]);
    assert_eq!(&current[block + 29..block + 32], &[55, 188, 27]);
    s.fighters[0].landing = Some((7250, 55));
    assert!(s.encode_shared().is_err());
}

#[test]
fn compact_footsteps_encode_only_recent_contact_and_preserve_older_consumers() {
    let mut s = sample();
    let block = 20 + 23 * s.bodies.len();
    s.fighters[1].footstep = Some(7189);
    s.fighters[0].footstep = Some(7201);
    s.fighters[1].landing = Some((7198, 42));
    let shared = s.encode_shared().unwrap();
    let latest = shared.compact_with_footsteps(Some(37));
    let older = shared.compact_with_landings(Some(37));
    assert_eq!(latest.len(), older.len());
    assert_eq!(latest[block], 3);
    assert_eq!(latest[block + 24], 8);
    assert_eq!(latest[block + 48], 13 * 8 + 1);
    assert_eq!(older[block + 48], 1);
    assert_eq!(&latest[block + 49..], &older[block + 49..]);
    s.arena_tick += 1;
    assert_eq!(
        s.encode_shared().unwrap().compact_with_footsteps(None)[block + 48],
        1
    );
    s.fighters[1].footstep = Some(7203);
    assert!(s.encode_shared().is_err());
}
