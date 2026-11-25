use super::*;

fn nominal_swing_difference(rig: &LocomotionRagdoll, side: usize) -> f32 {
    let t = &rig.state().targets;
    let [x, y, z] = t[8 + side];
    let nominal = Rotation::from_euler_angles(-0.5 * t[10 + side][0], 0.0, 0.0);
    nominal.angle_to(&Rotation::from_euler_angles(x, y, z))
}

#[test]
fn neutral_after_unsampled_tap_cannot_convert_an_external_foot_departure_into_a_swing() {
    let (mut rig, _) = rotating_deck(0., 0.);
    for _ in 0..60 {
        rig.step();
    }
    assert_eq!(rig.state().phase, 0.);
    assert_eq!(rig.state().targets, [[0.; 3]; 14]);
    rig.set_input([0., -1.], true).unwrap();
    rig.set_input([0.; 2], false).unwrap();
    for h in rig.pose.rig.bodies {
        rig.pose.rig.world.bodies[h].set_linvel(-Vector::z() * 2.0, true);
    }
    rig.queue_impulse(BodyPart::RightFoot, [0., 3., 0.])
        .unwrap();
    rig.step();
    assert_eq!(rig.state().feet, [true, false]);
    assert!(
        nominal_swing_difference(&rig, 1) < 0.00001,
        "a tap created a placement correction without a simulated swing"
    );
}

#[test]
fn closed_terminal_swing_stays_closed_after_later_chest_impact() {
    let (mut rig, _) = rotating_deck(0.4, 0.);
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([1., 0.], true).unwrap();
    for _ in 0..240 {
        rig.step();
    }
    rig.set_input([0.; 2], false).unwrap();
    for tick in 1..=2 {
        rig.step();
        assert_eq!(rig.state().feet, [false, true], "release step {tick}");
        assert!(nominal_swing_difference(&rig, 0) > 0.1);
    }
    rig.queue_impulse(BodyPart::Chest, [10., 0., 0.]).unwrap();
    rig.set_input([0.; 2], false).unwrap();
    rig.step();
    assert_eq!(rig.state().feet, [false, true]);
    assert!(nominal_swing_difference(&rig, 0) < 0.00001);
    assert!(nominal_swing_difference(&rig, 1) < 0.00001);
}

#[test]
fn stationary_gait_phase_cannot_arm_a_release_step() {
    let (mut rig, _) = rotating_deck(0.0, 0.0);
    let mut config = LocomotionConfig::bundled().unwrap();
    config.run_frequency_hz = 0.0;
    config.validate().unwrap();
    rig.controller = WalkingController::new(config);
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([0.0, -1.0], true).unwrap();
    rig.step();
    assert_eq!(rig.state().phase, 0.0);
    rig.set_input([0.0; 2], false).unwrap();
    for h in rig.pose.rig.bodies {
        rig.pose.rig.world.bodies[h].set_linvel(-Vector::z() * 2.0, true);
    }
    rig.queue_impulse(BodyPart::RightFoot, [0.0, 3.0, 0.0])
        .unwrap();
    rig.step();
    assert_eq!(rig.state().feet, [true, false]);
    assert!(nominal_swing_difference(&rig, 1) < 0.00001);
}

#[test]
fn a_supported_swing_is_cleared_by_support_loss_disable_or_reset() {
    for exit in 0..3 {
        let (mut rig, deck) = rotating_deck(0.4, 0.0);
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([1.0, 0.0], true).unwrap();
        for _ in 0..240 {
            rig.step();
        }
        rig.set_input([0.; 2], false).unwrap();
        rig.step();
        assert!(nominal_swing_difference(&rig, 0) > 0.1);
        match exit {
            0 => {
                let w = &mut rig.pose.rig.world;
                w.bodies[deck].set_enabled(false);
                rig.controller.advance(&mut rig.pose.rig, true);
                assert_eq!(rig.state().feet, [false, false]);
                rig.pose.rig.world.bodies[deck].set_enabled(true);
            }
            1 => {
                rig.set_enabled(false);
                rig.set_enabled(true);
            }
            _ => {
                rig.controller.reset();
            }
        }
        rig.controller.advance(&mut rig.pose.rig, true);
        assert_eq!(rig.state().feet, [false, true]);
        assert!(nominal_swing_difference(&rig, 0) < 1e-5, "exit path {exit}");
    }
}

#[test]
fn release_correction_respects_rounded_foot_clearance_for_inward_motion() {
    let (mut rig, deck) = rotating_deck(0.4, 0.0);
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([1.0, 0.0], true).unwrap();
    for _ in 0..240 {
        rig.step();
    }
    let p = &rig.pose.rig;
    let forwards = p.world.bodies[p.bodies[BodyPart::Pelvis.index()]].rotation() * -Vector::z();
    let heading = Rotation::from_axis_angle(&Vector::y_axis(), (-forwards.x).atan2(-forwards.z));
    for h in p.bodies {
        let deck_v = rig.pose.rig.world.bodies[deck]
            .velocity_at_point(rig.pose.rig.world.bodies[h].center_of_mass());
        rig.pose.rig.world.bodies[h].set_linvel(deck_v + heading * Vector::x() * 3.0, true);
    }
    rig.set_input([0.; 2], false).unwrap();
    rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.state().feet, [false, true]);
    let gap = crate::character::gait::geometry_tests::foot_separation(
        &rig.pose.rig.definition,
        rig.state().targets,
    );
    assert!(gap >= -1e-5);
}
