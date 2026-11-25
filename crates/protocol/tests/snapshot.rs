use brawler_protocol::snapshot::{BodyPose, FighterState, Snapshot};

fn sample() -> Snapshot {
    let mut fighters = vec![FighterState::default(); 2];
    fighters[0] = FighterState {
        consciousness: 0,
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
