use super::*;
use rapier3d::prelude::*;

fn walker(spawn: [f32; 3]) -> LocomotionRagdoll {
    LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        spawn,
    )
    .unwrap()
}

fn finite_and_bounded(rig: &LocomotionRagdoll) {
    for body in rig.pose.rig.body_states() {
        assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
    }
    assert!(Vector::from(rig.state().acceleration).norm() <= 12.0001);
    assert!(rig.state().turn_torque.abs() <= 20.0001);
    for joint in rig.pose.rig.joint_states() {
        assert!(joint.anchor_error < 0.05);
    }
}

#[test]
fn rotating_support_carries_an_idle_character_without_double_counting_velocity() {
    let mut rig = walker([2.0, 0.0, 0.0]);
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let platform = world.bodies.insert(
        RigidBodyBuilder::kinematic_velocity_based()
            .translation(vector![0.0, -0.5, 0.0])
            .angvel(vector![0.0, 0.4, 0.0]),
    );
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(6.0, 0.5, 6.0),
        platform,
        &mut world.bodies,
    );
    for tick in 0..600 {
        rig.step();
        finite_and_bounded(&rig);
        let states = rig.pose.rig.body_states();
        let pelvis = states[2].state.position;
        assert!(
            states[0].state.position[1] > 1.1,
            "rotation tick={tick}, pelvis={pelvis:?}"
        );
        let local = rig.pose.rig.world.bodies[platform]
            .position()
            .inverse_transform_point(&Point::from(pelvis));
        assert!(
            (vector![local.x, 0.0, local.z] - vector![2.0, 0.0, 0.0]).norm() < 0.4,
            "rotation tick={tick}, local={local:?}"
        );
    }
    let state = rig.pose.rig.body_states()[2].state;
    let expected = vector![0.4 * state.position[2], 0.0, -0.4 * state.position[0]];
    let actual = Vector::from(state.linear_velocity);
    assert!(
        (vector![actual.x, 0.0, actual.z] - expected).norm() < 0.3,
        "rotating velocity actual={actual:?}, expected={expected:?}"
    );
}

fn moving_floor(velocity: Vector<Real>) -> (LocomotionRagdoll, RigidBodyHandle) {
    let mut rig = walker([0.0; 3]);
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let platform = world.bodies.insert(
        RigidBodyBuilder::kinematic_velocity_based()
            .translation(vector![0.0, -0.5, 0.0])
            .linvel(velocity),
    );
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(6.0, 0.5, 6.0),
        platform,
        &mut world.bodies,
    );
    (rig, platform)
}

#[test]
fn idle_character_rides_cardinal_support_motion_and_stops_with_the_deck() {
    for velocity in [
        vector![0.8, 0.0, 0.0],
        vector![-0.8, 0.0, 0.0],
        vector![0.0, 0.0, 0.8],
        vector![0.0, 0.0, -0.8],
    ] {
        let (mut rig, platform) = moving_floor(velocity);
        for tick in 0..300 {
            rig.step();
            finite_and_bounded(&rig);
            let states = rig.pose.rig.body_states();
            assert!(
                states[0].state.position[1] > 1.1,
                "velocity={velocity:?} tick={tick}"
            );
            let local = rig.pose.rig.world.bodies[platform]
                .position()
                .inverse_transform_point(&Point::from(states[2].state.position));
            assert!(
                local.x.hypot(local.z) < 0.25,
                "velocity={velocity:?} tick={tick} local={local:?}"
            );
        }
        rig.pose.rig.world.bodies[platform].set_linvel(Vector::zeros(), true);
        for tick in 0..300 {
            rig.step();
            finite_and_bounded(&rig);
            assert!(
                rig.pose.rig.body_states()[0].state.position[1] > 1.1,
                "stopping velocity={velocity:?} tick={tick}"
            );
        }
        let state = rig.pose.rig.body_states()[2].state;
        assert!(state.linear_velocity[0].hypot(state.linear_velocity[2]) < 0.2);
        assert_eq!(rig.state().motor_strength, 0.0);
        assert!(Vector::from(rig.state().acceleration).norm() <= 1.0001);
    }
}

#[test]
fn moving_support_does_not_prevent_a_chest_impact_from_toppling_the_character() {
    let (mut rig, _) = moving_floor(vector![0.0, 0.0, -0.8]);
    for _ in 0..120 {
        rig.step();
    }
    for _ in 0..3 {
        rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])
            .unwrap();
        rig.step();
    }
    let mut lowest = 2.0_f32;
    for _ in 0..240 {
        rig.step();
        finite_and_bounded(&rig);
        lowest = lowest.min(rig.pose.rig.body_states()[0].state.position[1]);
    }
    assert!(
        lowest < 0.8,
        "moving support resisted chest impulses: {lowest}"
    );
}

#[test]
fn removing_moving_support_stops_assistance_without_removing_departure_momentum() {
    for removal in 0..3 {
        let (mut rig, platform) = moving_floor(vector![0.8, 0.0, 0.0]);
        for _ in 0..120 {
            rig.step();
        }
        let collider = rig.pose.rig.world.bodies[platform].colliders()[0];
        match removal {
            0 => rig.pose.rig.world.bodies[platform].set_enabled(false),
            1 => rig.pose.rig.world.colliders[collider].set_enabled(false),
            _ => {
                assert!(rig.pose.rig.world.remove_collider(collider).is_some());
            }
        }
        let momentum = |rig: &LocomotionRagdoll| {
            rig.pose
                .rig
                .body_states()
                .iter()
                .fold(Vector::zeros(), |sum, body| {
                    sum + Vector::from(body.state.linear_velocity) * body.state.mass
                })
        };
        let before = momentum(&rig);
        for _ in 0..30 {
            rig.step();
            assert_eq!(rig.state().acceleration, [0.0; 3]);
            assert_eq!(rig.state().turn_torque, 0.0);
        }
        let after = momentum(&rig);
        assert!((after.x - before.x).abs() < 1e-3 && (after.z - before.z).abs() < 1e-3);
        assert!(before.x > 20.0);
    }
}

#[test]
fn support_acceleration_preserves_one_total_impulse_budget_without_pose_changes() {
    let (mut rig, platform) = moving_floor(Vector::zeros());
    for _ in 0..60 {
        rig.step();
    }
    // Exercise the full budget from rest, independently of small standing sway.
    for handle in rig.pose.rig.bodies {
        rig.pose.rig.world.bodies[handle].set_linvel(Vector::zeros(), true);
    }
    rig.pose.rig.world.bodies[platform].set_linvel(vector![8.0, 0.0, 0.0], true);
    let before = rig.pose.rig.body_states();
    rig.controller.advance(&mut rig.pose.rig, true);
    let after = rig.pose.rig.body_states();
    let mut impulse = Vector::zeros();
    let mut absolute = 0.0;
    for (before, after) in before.iter().zip(after) {
        assert_eq!(before.state.position, after.state.position);
        assert_eq!(before.state.rotation, after.state.rotation);
        let delta = (Vector::from(after.state.linear_velocity)
            - Vector::from(before.state.linear_velocity))
            * before.state.mass;
        impulse += delta;
        absolute += delta.norm();
    }
    let expected = Vector::from(rig.state().acceleration) * 40.0 * crate::TICK_SECONDS;
    assert!((impulse - expected).norm() < 1e-4);
    assert!(absolute <= 8.0001);
    assert!(impulse.x > 7.9);
    assert_eq!(impulse.y, 0.0);
}

#[test]
fn reset_discards_pending_support_motion_before_rebuilding_the_default_floor() {
    let (mut rig, platform) = moving_floor(Vector::zeros());
    for _ in 0..60 {
        rig.step();
    }
    rig.pose.rig.world.bodies[platform].set_linvel(vector![8.0, 0.0, 0.0], true);
    rig.step();
    rig.reset();
    let mut fresh = walker([0.0; 3]);
    for _ in 0..60 {
        rig.step();
        fresh.step();
        assert_eq!(rig.pose.rig.body_states(), fresh.pose.rig.body_states());
        assert_eq!(rig.state(), fresh.state());
    }
}

#[test]
fn jumping_from_a_translating_deck_keeps_existing_horizontal_momentum() {
    let (mut rig, _) = moving_floor(Vector::x() * 0.8);
    for _ in 0..180 {
        rig.step();
    }
    let bodies = rig.pose.rig.body_states();
    let velocity = |bodies: &[crate::RagdollBodyState]| {
        bodies
            .iter()
            .map(|b| Vector::from(b.state.linear_velocity) * b.state.mass / 40.)
            .sum::<Vector<Real>>()
    };
    let before = velocity(&bodies);
    assert!((before.x - 0.8).abs() < 0.05);
    rig.set_jump(true);
    rig.step();
    let change = velocity(&rig.pose.rig.body_states()) - before;
    assert!(change.x.hypot(change.z) < 0.01, "{change:?}");
    assert!((change.y - (4.5 - 9.81 / 60.)).abs() < 0.025);
}
