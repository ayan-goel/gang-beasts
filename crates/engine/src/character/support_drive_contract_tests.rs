use super::*;

fn isolated_drive(spin: f32, translation: f32, cap: f32, one_foot: bool) -> ([f32; 3], [f32; 3]) {
    let (mut walking, deck) = rotating_deck(0.0, 0.0);
    for _ in 0..60 {
        walking.step();
    }
    let rig = &mut walking.pose.rig;
    rig.world.bodies[deck].set_angvel(Vector::y() * spin, true);
    rig.world.bodies[deck].set_linvel(Vector::x() * translation, true);
    for handle in rig.bodies {
        rig.world.bodies[handle].set_linvel(Vector::x() * (translation + 4.0), true);
        rig.world.bodies[handle].set_angvel(Vector::zeros(), true);
    }
    if one_foot {
        let foot = &rig.world.bodies[rig.bodies[BodyPart::RightFoot.index()]];
        rig.world.colliders[foot.colliders()[0]].set_enabled(false);
    }
    let mut config = LocomotionConfig::bundled().unwrap();
    config.support_moment_cap_nm = cap;
    let mut controller = crate::character::walking_controller::WalkingController::new(config);
    // A translated frame also needs a translated support-history state.
    // Prime the carry reference without advancing geometry, then restore motion.
    for _ in 0..100 {
        controller.advance(rig, true);
        for handle in rig.bodies {
            rig.world.bodies[handle].set_linvel(Vector::x() * (translation + 4.0), true);
            rig.world.bodies[handle].set_angvel(Vector::zeros(), true);
        }
    }
    let before = rig.body_states();
    let centers = rig
        .bodies
        .map(|h| rig.world.bodies[h].center_of_mass().coords);
    let center = before
        .iter()
        .enumerate()
        .fold(Vector::zeros(), |sum, (i, b)| {
            sum + centers[i] * b.state.mass
        })
        / 40.0;
    controller.advance(rig, true);
    let mut impulse = Vector::zeros();
    let mut angular = Vector::zeros();
    let mut absolute = 0.0;
    for (i, (old, new)) in before.iter().zip(rig.body_states()).enumerate() {
        assert_eq!(old.state.position, new.state.position);
        assert_eq!(old.state.rotation, new.state.rotation);
        let change = (Vector::from(new.state.linear_velocity)
            - Vector::from(old.state.linear_velocity))
            * old.state.mass;
        assert_eq!(change.y, 0.0);
        impulse += change;
        absolute += change.norm();
        angular += (centers[i] - center).cross(&change);
    }
    let moment = angular / crate::TICK_SECONDS;
    assert!((moment - Vector::from(controller.state.support_moment)).norm() < 0.003);
    assert!(moment.norm() <= cap + 0.003);
    assert!(
        (impulse - Vector::from(controller.state.acceleration) * 40.0 * crate::TICK_SECONDS).norm()
            < 0.0002
    );
    assert!(absolute <= 8.0002);
    assert_eq!(controller.state.motor_strength, 0.0);
    assert_eq!(controller.state.stopping_moment, [0.0; 3]);
    (
        controller.state.support_moment,
        controller.state.acceleration,
    )
}

#[test]
fn support_compensation_has_a_measured_moment_cap_and_no_vertical_or_unbudgeted_impulse() {
    for spin in [-4.0, 4.0] {
        for cap in [0.0, 30.0, 120.0] {
            for one_foot in [false, true] {
                let (moment, _) = isolated_drive(spin, 2.0, cap, one_foot);
                assert!((Vector::from(moment).norm() - cap).abs() < 0.003);
            }
        }
    }
}

#[test]
fn support_compensation_is_translation_invariant_and_vanishes_with_rotation() {
    for spin in [-0.1, 0.0, 0.1] {
        let (moment, acceleration) = isolated_drive(spin, 0.0, 120.0, false);
        for translation in [-8.0, 8.0] {
            let (shifted, command) = isolated_drive(spin, translation, 120.0, false);
            assert!((Vector::from(shifted) - Vector::from(moment)).norm() < 0.001);
            assert!((Vector::from(command) - Vector::from(acceleration)).norm() < 0.001);
        }
    }
    for spin in [-0.000001, 0.000001] {
        let (moment, _) = isolated_drive(spin, 0.0, 120.0, false);
        assert!(Vector::from(moment).norm() < 0.001);
    }
}

#[test]
fn overlapping_stop_and_support_moments_match_applied_impulses_after_shared_scaling() {
    let mut stopping = [0.0; 2];
    for (index, cap) in [0.0, 120.0].into_iter().enumerate() {
        let (mut walking, deck) = rotating_deck(0.0, 0.0);
        for _ in 0..60 {
            walking.step();
        }
        let rig = &mut walking.pose.rig;
        rig.world.bodies[deck].set_angvel(Vector::y() * 4.0, true);
        for handle in rig.bodies {
            rig.world.bodies[handle].set_linvel(Vector::x() * 4.0, true);
            rig.world.bodies[handle].set_angvel(Vector::zeros(), true);
        }
        let mut config = LocomotionConfig::bundled().unwrap();
        config.support_moment_cap_nm = cap;
        let mut controller = WalkingController::new(config);
        controller.state.motor_strength = 1.0;
        controller.set_input([1.0, 0.0], true).unwrap();
        controller.set_input([0.0; 2], false).unwrap();
        let before = rig.body_states();
        let centers = rig
            .bodies
            .map(|h| rig.world.bodies[h].center_of_mass().coords);
        let center = centers
            .iter()
            .zip(&before)
            .fold(Vector::zeros(), |sum, (p, body)| sum + p * body.state.mass)
            / 40.0;
        controller.advance(rig, true);
        let mut angular = Vector::zeros();
        let mut absolute = 0.0;
        let mut total = Vector::zeros();
        for (i, (old, new)) in before.iter().zip(rig.body_states()).enumerate() {
            let impulse = (Vector::from(new.state.linear_velocity)
                - Vector::from(old.state.linear_velocity))
                * old.state.mass;
            assert_eq!(impulse.y, 0.0);
            absolute += impulse.norm();
            total += impulse;
            angular += (centers[i] - center).cross(&impulse);
        }
        let state = &controller.state;
        let reported = Vector::from(state.stopping_moment) + Vector::from(state.support_moment);
        assert!((angular / crate::TICK_SECONDS - reported).norm() < 0.003);
        assert!(
            (total - Vector::from(state.acceleration) * 40.0 * crate::TICK_SECONDS).norm() < 0.0002
        );
        assert!(absolute <= 8.0002);
        stopping[index] = Vector::from(state.stopping_moment).norm();
        assert!(stopping[index] > 1.0);
        assert_eq!(state.turn_torque, 0.0);
        if cap > 0.0 {
            assert!(Vector::from(state.support_moment).norm() > 1.0);
        }
    }
    // The combined request must exercise normalization, not just two idle caps.
    assert!(stopping[1] < stopping[0] - 1.0, "{stopping:?}");
}
