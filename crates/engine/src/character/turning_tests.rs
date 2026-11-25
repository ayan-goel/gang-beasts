use super::*;

fn verify_reversals(mut rig: LocomotionRagdoll, direction: Vector<Real>, running: bool) {
    for _ in 0..60 {
        rig.step();
    }
    for leg in 0..8 {
        let sign = if leg % 2 == 0 { 1.0 } else { -1.0 };
        let request = direction * sign;
        for _ in 0..45 {
            // Repeated input samples must not restart braking for a held direction.
            rig.set_input([request.x, request.z], running).unwrap();
            rig.step();
            bounded(&rig, Vector::y());
        }
        let velocity = Vector::from(rig.pose.rig.body_states()[2].state.linear_velocity);
        assert!(
            velocity.dot(&request) > 0.5,
            "leg {leg}, velocity {velocity:?}"
        );
    }
}

#[test]
fn repeated_reversals_keep_posture_and_change_velocity_in_four_orientations() {
    for yaw in [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
    ] {
        for running in [false, true] {
            let (rig, _, forward) = slope(0.0, yaw);
            verify_reversals(rig, forward.cross(&Vector::y()), running);
        }
    }
}

#[test]
fn repeated_reversals_also_work_from_the_default_platform_spawn() {
    for running in [false, true] {
        let rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.0; 3],
        )
        .unwrap();
        verify_reversals(rig, Vector::x(), running);
    }
}

fn reversing_runner() -> LocomotionRagdoll {
    let (mut rig, _, _) = slope(0.0, 0.0);
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([1.0, 0.0], true).unwrap();
    for _ in 0..45 {
        rig.step();
    }
    assert!(rig.pose.rig.body_states()[2].state.linear_velocity[0] > 3.0);
    rig.set_input([-1.0, 0.0], true).unwrap();
    rig
}

#[test]
fn reversal_braking_respects_the_absolute_impulse_budget_and_does_not_teleport() {
    let mut rig = reversing_runner();
    let before = rig.pose.rig.body_states();
    rig.controller.advance(&mut rig.pose.rig, true);
    let mut total = Vector::zeros();
    let mut absolute = 0.0;
    for (old, new) in before.iter().zip(rig.pose.rig.body_states()) {
        assert_eq!(old.state.position, new.state.position);
        assert_eq!(old.state.rotation, new.state.rotation);
        let impulse = (Vector::from(new.state.linear_velocity)
            - Vector::from(old.state.linear_velocity))
            * old.state.mass;
        total += impulse;
        absolute += impulse.norm();
    }
    assert!(total.x < 0.0);
    assert_eq!(total.y, 0.0);
    assert!(absolute <= 8.0001);
    assert!(
        (total - Vector::from(rig.state().acceleration) * 40.0 * crate::TICK_SECONDS).norm()
            < 0.0001
    );
    assert_eq!(rig.state().turn_torque, 0.0);
}

#[test]
fn removed_support_clears_reversal_before_a_later_contact() {
    let mut rig = reversing_runner();
    rig.step();
    assert_eq!(rig.state().turn_torque, 0.0);
    let surfaces: Vec<_> = rig
        .pose
        .rig
        .world
        .colliders
        .iter()
        .filter(|(_, collider)| collider.parent().is_none() && collider.is_enabled())
        .map(|(handle, _)| handle)
        .collect();
    for handle in &surfaces {
        rig.pose.rig.world.colliders[*handle].set_enabled(false);
    }
    let before = rig.pose.rig.body_states();
    rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.state().feet, [false; 2]);
    assert_eq!(rig.pose.rig.body_states(), before);
    for handle in surfaces {
        rig.pose.rig.world.colliders[handle].set_enabled(true);
    }
    rig.controller.advance(&mut rig.pose.rig, true);
    assert!(rig.state().turn_torque.abs() > 1.0);
}

#[test]
fn disable_discards_an_active_reversal() {
    let mut rig = reversing_runner();
    rig.step();
    assert_eq!(rig.state().turn_torque, 0.0);
    rig.set_enabled(false);
    rig.set_enabled(true);
    rig.set_input([-1.0, 0.0], true).unwrap();
    rig.controller.advance(&mut rig.pose.rig, true);
    assert!(rig.state().turn_torque.abs() > 1.0);
}

#[test]
fn changing_analog_speed_during_reversal_uses_the_latest_request() {
    for amount in [-0.35] {
        let mut rig = reversing_runner();
        for _ in 0..5 {
            rig.step();
        }
        rig.set_input([amount, 0.0], false).unwrap();
        for _ in 0..180 {
            rig.step();
            bounded(&rig, Vector::y());
        }
        let velocity = Vector::from(rig.pose.rig.body_states()[2].state.linear_velocity);
        assert!(
            (velocity.x - amount * 2.5).abs() < 0.3,
            "request {amount}, velocity {velocity:?}"
        );
    }
}

#[test]
fn key_release_and_opposite_press_in_one_tick_match_a_direct_reversal() {
    let mut direct = reversing_runner();
    let mut key_events = reversing_runner();
    key_events.set_input([0.0, 0.0], false).unwrap();
    key_events.set_input([-1.0, 0.0], true).unwrap();
    for _ in 0..45 {
        direct.step();
        key_events.step();
        assert_eq!(
            direct.pose.rig.body_states(),
            key_events.pose.rig.body_states()
        );
        assert_eq!(direct.state(), key_events.state());
        bounded(&key_events, Vector::y());
    }
}

fn verify_neutral_gaps(mut rig: LocomotionRagdoll, running: bool, gap: usize) {
    for _ in 0..60 {
        rig.step();
    }
    for leg in 0..8 {
        let sign = if leg % 2 == 0 { 1.0 } else { -1.0 };
        for tick in 0..45 {
            let request = if leg > 0 && tick < gap { 0.0 } else { sign };
            rig.set_input([request, 0.0], running && request != 0.0)
                .unwrap();
            rig.step();
            bounded(&rig, Vector::y());
        }
        let velocity = rig.pose.rig.body_states()[2].state.linear_velocity[0];
        assert!(
            velocity * sign > 0.5,
            "gap {gap}, leg {leg}, velocity {velocity}"
        );
    }
}

#[test]
fn brief_neutral_key_gaps_still_allow_repeated_reversals() {
    for running in [false, true] {
        for gap in [1, 5] {
            let (raised, _, _) = slope(0.0, 0.0);
            let ordinary = LocomotionRagdoll::new(
                PhysicsConfig::default(),
                LocomotionConfig::bundled().unwrap(),
                [0.0; 3],
            )
            .unwrap();
            verify_neutral_gaps(raised, running, gap);
            verify_neutral_gaps(ordinary, running, gap);
        }
    }
}
