use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

#[test]
fn walking_and_running_advance_with_a_physical_gait_and_bounded_effort() {
    for running in [false, true] {
        let mut rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.0, 0.0, 4.0],
        )
        .unwrap();
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([0.0, -1.0], running).unwrap();
        let mut speed = 0.0;
        let mut lifted = [false; 2];
        let mut previous_swing = 0;
        let mut alternating_swings = 0;
        for tick in 0..120 {
            rig.step();
            let states = rig.pose().rig().body_states();
            let head = &states[BodyPart::Head.index()].state;
            assert!(
                head.position[1] > 1.1,
                "running={running}, tick={tick}, head={:?}",
                head.position
            );
            for body in &states {
                assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
            }
            for (index, foot) in [BodyPart::LeftFoot, BodyPart::RightFoot]
                .into_iter()
                .enumerate()
            {
                lifted[index] |= states[foot.index()].state.position[1] > 0.11;
            }
            let left = states[BodyPart::LeftFoot.index()].state.position[1];
            let right = states[BodyPart::RightFoot.index()].state.position[1];
            let swing = if left > 0.11 && left - right > 0.025 && rig.state().feet[1] {
                1
            } else if right > 0.11 && right - left > 0.025 && rig.state().feet[0] {
                -1
            } else {
                0
            };
            if swing != 0 && swing != previous_swing {
                alternating_swings += 1;
                previous_swing = swing;
            }
            assert!(
                rig.state()
                    .acceleration
                    .iter()
                    .map(|v| v * v)
                    .sum::<f32>()
                    .sqrt()
                    <= 12.0 + 1e-4
            );
            assert!(
                rig.pose()
                    .rig()
                    .joint_states()
                    .iter()
                    .all(|joint| joint.anchor_error < 0.05)
            );
            for (joint, effort) in rig
                .pose()
                .rig()
                .definition()
                .joints
                .iter()
                .zip(rig.pose().solver_motor_efforts().unwrap())
            {
                let cap = rig
                    .pose()
                    .motor_configuration()
                    .joints
                    .iter()
                    .find(|motor| motor.child == joint.child)
                    .unwrap()
                    .max_torque_nm;
                assert!(effort.iter().map(|value| value * value).sum::<f32>().sqrt() <= cap + 1e-3);
            }
            assert!((0.0..=1.0).contains(&rig.state().motor_strength));
            if tick >= 90 {
                speed -= states[BodyPart::Pelvis.index()].state.linear_velocity[2] / 30.0;
            }
        }
        let head = rig.pose().rig().body_states()[BodyPart::Head.index()]
            .state
            .position;
        println!(
            "running={running}: head_z={:.4}, mean final speed={speed:.4}, alternating swings={alternating_swings}",
            head[2]
        );
        assert!(head[2] < 2.0, "running={running}, head={head:?}");
        assert!(
            speed > if running { 3.2 } else { 1.6 },
            "running={running}, speed={speed}"
        );
        assert!(speed < if running { 5.0 } else { 3.0 });
        assert!(
            alternating_swings >= 4,
            "running={running}: only {alternating_swings} distinct swings"
        );
        assert_eq!(
            lifted, [true; 2],
            "both feet must leave their resting height"
        );
    }
}

#[test]
fn invalid_intent_is_rejected_and_reset_discards_movement() {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    for input in [[f32::NAN, 0.0], [0.0, f32::INFINITY]] {
        assert!(rig.set_input(input, false).is_err());
    }
    rig.set_input([1.0, -1.0], true).unwrap();
    rig.step();
    rig.reset();
    let mut fresh = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    for _ in 0..60 {
        rig.step();
        fresh.step();
        assert_eq!(
            rig.pose().rig().body_states(),
            fresh.pose().rig().body_states()
        );
    }
}

#[test]
fn cardinal_movement_stops_without_a_position_tether() {
    for direction in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
        let mut rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.0; 3],
        )
        .unwrap();
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input(direction, false).unwrap();
        for tick in 0..90 {
            rig.step();
            assert!(
                rig.pose().rig().body_states()[0].state.position[1] > 1.1,
                "direction={direction:?} tick={tick}"
            );
        }
        let before = rig.pose().rig().body_states()[2].state.position;
        assert!(
            before[0] * direction[0] + before[2] * direction[1] > 1.5,
            "direction={direction:?}, pelvis={before:?}"
        );
        rig.set_input([0.0; 2], false).unwrap();
        for _ in 0..120 {
            rig.step();
        }
        let state = &rig.pose().rig().body_states()[2].state;
        assert!(
            state.position[1] > 0.6,
            "fell while stopping {direction:?}: {state:?}"
        );
        assert!(
            state.linear_velocity[0].hypot(state.linear_velocity[2]) < 0.2,
            "stop {direction:?}: {state:?}"
        );
        assert!(state.position[0].hypot(state.position[2]) > 1.5);
    }
}

#[test]
fn airborne_and_disabled_characters_receive_no_movement_assistance() {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap();
    rig.set_input([1.0, 1.0], true).unwrap();
    for _ in 0..20 {
        rig.step();
        assert_eq!(rig.state().acceleration, [0.0; 3]);
        assert_eq!(rig.state().turn_torque, 0.0);
        assert_eq!(rig.state().motor_strength, 0.0);
    }
    rig.set_enabled(false);
    rig.reset();
    let mut passive = brawler_engine::PassiveRagdoll::new(
        PhysicsConfig::default(),
        brawler_engine::CharacterDefinition::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap();
    for _ in 0..120 {
        rig.step();
        passive.step();
        assert_eq!(rig.pose().rig().body_states(), passive.body_states());
    }
}

#[test]
fn idle_locomotion_stays_standing_but_still_yields_to_chest_impacts() {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    for _ in 0..1200 {
        rig.step();
        let head = &rig.pose().rig().body_states()[0].state;
        assert!(head.position[1] > 1.15);
        assert!(head.position[0].hypot(head.position[2]) < 0.2);
    }
    for _ in 0..3 {
        rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])
            .unwrap();
        rig.step();
    }
    let mut lowest = 2.0_f32;
    for _ in 0..480 {
        rig.step();
        lowest = lowest.min(rig.pose().rig().body_states()[0].state.position[1]);
    }
    assert!(
        lowest < 0.8,
        "idle assistance resisted the impacts: {lowest}"
    );
}

#[test]
fn release_relaxes_to_the_idle_motor_preset_after_settling() {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    rig.set_input([1.0, 0.0], false).unwrap();
    for _ in 0..90 {
        rig.step();
    }
    rig.set_input([0.0; 2], false).unwrap();
    for _ in 0..600 {
        rig.step();
    }
    assert_eq!(rig.state().motor_strength, 0.0);
    let idle = brawler_engine::MotorConfig::standing().unwrap();
    assert_eq!(
        serde_json::to_value(rig.pose().motor_configuration()).unwrap(),
        serde_json::to_value(idle).unwrap()
    );
}

#[test]
fn large_diagonal_inputs_normalize_and_invalid_updates_preserve_intent() {
    let build = || {
        LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.0; 3],
        )
        .unwrap()
    };
    let mut first = build();
    let mut second = build();
    first.set_input([1.0, 1.0], false).unwrap();
    second.set_input([f32::MAX, f32::MAX], false).unwrap();
    assert!(second.set_input([f32::NAN, 0.0], true).is_err());
    for _ in 0..120 {
        first.step();
        second.step();
        assert_eq!(
            first.pose().rig().body_states(),
            second.pose().rig().body_states()
        );
    }
}
