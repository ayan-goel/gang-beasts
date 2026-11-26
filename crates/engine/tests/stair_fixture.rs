use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

#[test]
fn low_stairs_have_a_level_approach_three_adjacent_treads_and_a_long_landing() {
    let mut rig = LocomotionRagdoll::stairs(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
    )
    .unwrap();
    let boxes = rig.pose().rig().terrain_boxes();
    assert_eq!(boxes.len(), 4);
    assert_eq!(boxes[0], [0.0, -0.5, 0.0, 30.0, 0.5, 30.0]);
    for (shape, (front, back, top)) in
        boxes[1..]
            .iter()
            .zip([(-1.0, -1.8, 0.1), (-1.8, -2.6, 0.2), (-2.6, -12.0, 0.3)])
    {
        let [x, y, z, dx, dy, dz] = *shape;
        assert_eq!((x, dx), (0.0, 3.0));
        assert!((z + dz - front).abs() < 1e-6);
        assert!((z - dz - back).abs() < 1e-6);
        assert!((y + dy - top).abs() < 1e-6);
        assert_eq!(y - dy, 0.0);
    }
    rig.step();
    rig.reset();
    assert_eq!(rig.pose().rig().terrain_boxes(), boxes);
}

#[test]
fn initial_stance_timing_preserves_previously_reachable_landings() {
    let config = LocomotionConfig::bundled().unwrap();
    let mut falls = Vec::new();
    for settle in (1..=120).chain([180]) {
        let mut rig = LocomotionRagdoll::stairs(PhysicsConfig::default(), config.clone()).unwrap();
        for _ in 0..settle {
            rig.step();
        }
        rig.set_input([0., -1.], false).unwrap();
        for elapsed in 1..=420 {
            if elapsed == 241 {
                rig.set_input([0.; 2], false).unwrap();
            }
            rig.step();
            let bodies = rig.pose().rig().body_states();
            let pelvis = &bodies[BodyPart::Pelvis.index()].state;
            let position = pelvis.position;
            let height = bodies[BodyPart::Head.index()].state.position[1];
            if height <= 1.1 {
                falls.push((settle, elapsed));
                break;
            }
            assert!(height.is_finite(), "settle {settle}, elapsed {elapsed}");
            if elapsed == 150 || elapsed == 420 {
                assert!(
                    position[2] < -3.,
                    "settle {settle}, elapsed {elapsed}: {position:?}"
                );
            }
            if elapsed == 420 {
                assert!(
                    position[1] > 0.85
                        && pelvis.linear_velocity.iter().map(|v| v * v).sum::<f32>() < 0.04,
                    "settle {settle}"
                );
                assert_eq!(rig.state().feet, [true; 2], "settle {settle}");
            }
        }
    }
    // Track individual starts so resolving one fall cannot mask a new fall elsewhere.
    let known_starts = [19, 25, 26, 43, 108];
    eprintln!("Stair falls (settling ticks, walking ticks): {falls:?}");
    assert!(
        falls
            .iter()
            .all(|(settle, _)| known_starts.contains(settle)),
        "{falls:?}"
    );
}
