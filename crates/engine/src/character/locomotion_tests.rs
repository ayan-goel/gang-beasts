use super::*;
use crate::TICK_SECONDS;
use rapier3d::prelude::*;

fn walker() -> LocomotionRagdoll {
    LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap()
}

#[test]
fn applied_momentum_matches_the_total_command_without_moving_transforms() {
    let mut rig = walker();
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([1.0, -1.0], true).unwrap();
    let before = rig.pose.rig.body_states();
    rig.controller.advance(&mut rig.pose.rig, true);
    let after = rig.pose.rig.body_states();
    let mut delta = Vector::zeros();
    let mut absolute_impulses = 0.0;
    for (before, after) in before.iter().zip(after) {
        assert_eq!(before.state.position, after.state.position);
        assert_eq!(before.state.rotation, after.state.rotation);
        let impulse = (Vector::from(after.state.linear_velocity)
            - Vector::from(before.state.linear_velocity))
            * before.state.mass;
        delta += impulse;
        absolute_impulses += impulse.norm();
    }
    let expected = Vector::from(rig.state().acceleration) * 40.0 * TICK_SECONDS;
    assert!((delta - expected).norm() < 1e-4);
    assert!(absolute_impulses <= 40.0 * 12.0 * TICK_SECONDS + 1e-4);
    assert_eq!(delta.y, 0.0);
}

#[test]
fn translating_support_is_followed_without_adding_its_velocity_twice() {
    let mut rig = walker();
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let platform = world.bodies.insert(
        RigidBodyBuilder::kinematic_velocity_based()
            .translation(vector![0.0, -0.5, 0.0])
            .linvel(vector![0.8, 0.0, 0.0]),
    );
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(6.0, 0.5, 6.0),
        platform,
        &mut world.bodies,
    );
    for _ in 0..120 {
        rig.step();
    }
    rig.set_input([0.0, -1.0], false).unwrap();
    for _ in 0..90 {
        rig.step();
    }
    let states = rig.pose.rig.body_states();
    let velocity = states.iter().fold(Vector::zeros(), |v, body| {
        v + Vector::from(body.state.linear_velocity) * body.state.mass / 40.0
    });
    assert!(
        (velocity.x - 0.8).abs() < 0.3,
        "support-relative velocity {velocity:?}"
    );
    assert!(velocity.z < -1.6);
    assert!(states[0].state.position[1] > 1.1);
    assert_eq!(rig.state().support_velocity, [0.8, 0.0, 0.0]);
}

#[test]
fn a_wall_does_not_accumulate_unbounded_movement_energy() {
    let mut rig = walker();
    let wall = rig
        .pose
        .rig
        .world
        .colliders
        .insert(ColliderBuilder::cuboid(5.0, 2.0, 0.1).translation(vector![0.0, 2.0, -2.0]));
    rig.set_input([0.0, -1.0], true).unwrap();
    let mut contacted_wall = false;
    for _ in 0..600 {
        rig.step();
        contacted_wall |= rig
            .pose
            .rig
            .world
            .narrow_phase
            .contact_pairs_with(wall)
            .any(|pair| pair.has_any_active_contact);
        let energy: f32 = rig
            .pose
            .rig
            .bodies
            .iter()
            .map(|handle| rig.pose.rig.world.bodies[*handle].kinetic_energy())
            .sum();
        assert!(
            energy.is_finite() && energy < 1000.0,
            "wall energy {energy}"
        );
        let states = rig.pose.rig.body_states();
        for body in states {
            assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
            assert!(body.state.position[2] > -2.2);
            assert!(Vector::from(body.state.linear_velocity).norm() < 12.0);
        }
        assert!(Vector::from(rig.state().acceleration).norm() <= 12.0 + 1e-4);
    }
    assert!(contacted_wall, "fixture never reached its wall");
}

#[test]
fn leaving_the_platform_removes_drive_and_preserves_ballistic_momentum() {
    let mut rig = walker();
    rig.set_input([0.0, -1.0], true).unwrap();
    let mut saw_active = false;
    let mut unsupported_ticks = 0;
    let mut saw_release = false;
    let release_s = LocomotionConfig::bundled().unwrap().motor_release_s;
    let release_ticks = (release_s / TICK_SECONDS).ceil() as usize + 1;
    for _ in 0..240 {
        let previous = rig.state().motor_strength;
        rig.step();
        saw_active |= rig.state().motor_strength == 1.0;
        if saw_active && rig.state().feet == [false; 2] {
            unsupported_ticks += 1;
            assert!(
                rig.state().motor_strength <= (previous - TICK_SECONDS / release_s).max(0.0) + 1e-6
            );
            if unsupported_ticks >= release_ticks {
                assert_eq!(rig.state().motor_strength, 0.0);
                assert_eq!(
                    serde_json::to_value(rig.pose.motor_configuration()).unwrap(),
                    serde_json::to_value(MotorConfig::standing().unwrap()).unwrap()
                );
                saw_release = true;
            }
        } else {
            unsupported_ticks = 0;
        }
    }
    assert!(saw_active && saw_release);
    let states = rig.pose.rig.body_states();
    assert!(states[2].state.position[2] < -7.0);
    let momentum = |rig: &LocomotionRagdoll| {
        rig.pose
            .rig
            .body_states()
            .iter()
            .fold(Vector::zeros(), |v, body| {
                v + Vector::from(body.state.linear_velocity) * body.state.mass
            })
    };
    let before = momentum(&rig);
    for _ in 0..20 {
        rig.step();
        assert_eq!(rig.state().acceleration, [0.0; 3]);
        assert_eq!(rig.state().turn_torque, 0.0);
    }
    let after = momentum(&rig);
    assert!((before.x - after.x).abs() < 1e-3 && (before.z - after.z).abs() < 1e-3);
}

#[test]
fn nearly_perpendicular_steering_does_not_flip_the_foot_impulse() {
    let sample = |offset: f32| {
        let mut rig = walker();
        for _ in 0..60 {
            rig.step();
        }
        for handle in rig.pose.rig.bodies {
            let body = &mut rig.pose.rig.world.bodies[handle];
            body.set_linvel(vector![2.0 + offset, 0.0, 0.0], true);
            body.set_angvel(Vector::zeros(), true);
        }
        rig.set_input([0.8, 0.2], false).unwrap();
        let before = rig.pose.rig.body_states();
        rig.controller.advance(&mut rig.pose.rig, true);
        let after = rig.pose.rig.body_states();
        let feet = [BodyPart::LeftFoot, BodyPart::RightFoot].into_iter().fold(
            Vector::zeros(),
            |sum, part| {
                let index = part.index();
                sum + (Vector::from(after[index].state.linear_velocity)
                    - Vector::from(before[index].state.linear_velocity))
                    * before[index].state.mass
            },
        );
        (Vector::from(rig.state().acceleration), feet)
    };
    let before = sample(-0.00001);
    let after = sample(0.00001);
    assert!(
        (before.0 - after.0).norm() < 0.002,
        "acceleration flips: {before:?}, {after:?}"
    );
    assert!(
        (before.1 - after.1).norm() < 0.002,
        "foot impulse flips: {before:?}, {after:?}"
    );
}
