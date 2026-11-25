use super::*;
use crate::TICK_SECONDS;
use rapier3d::prelude::*;

fn running_pose() -> LocomotionRagdoll {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([0.0, -1.0], true).unwrap();
    for _ in 0..6 {
        rig.step();
    }
    rig
}

#[test]
fn brief_support_loss_fades_internal_pose_without_world_assistance() {
    let config = LocomotionConfig::bundled().unwrap();
    let mut rig = running_pose();
    let before = rig.state().targets;
    assert!(before[11][0].abs() > 0.4);
    assert_eq!(rig.state().motor_strength, 1.0);
    for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let bodies = rig.pose.rig.body_states();
    let targets = rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.pose.rig.body_states(), bodies);
    assert_eq!(rig.state().feet, [false; 2]);
    assert_eq!(rig.state().acceleration, [0.0; 3]);
    assert_eq!(rig.state().turn_torque, 0.0);
    assert_eq!(rig.state().stopping_moment, [0.0; 3]);
    assert_eq!(rig.state().support_moment, [0.0; 3]);
    let after = rig.state().targets;
    let knee_range = rig.pose.rig.definition.joints[11].angular_limits[0];
    let maximum_change = (knee_range[1] - knee_range[0]) * TICK_SECONDS / config.motor_release_s;
    assert!(
        (after[11][0] - before[11][0]).abs() <= maximum_change + 0.000001,
        "support loss snapped knee from {} to {}",
        before[11][0],
        after[11][0]
    );
    assert!(after[11][0].abs() < before[11][0].abs());
    assert!(targets[11].angle() >= before[11][0].abs() - maximum_change - 0.000001);
    for _ in 0..(config.motor_release_s / TICK_SECONDS).ceil() as u32 + 1 {
        rig.controller.advance(&mut rig.pose.rig, true);
        assert_eq!(rig.pose.rig.body_states(), bodies);
        assert_eq!(rig.state().acceleration, [0.0; 3]);
        assert_eq!(rig.state().stopping_moment, [0.0; 3]);
        assert_eq!(rig.state().support_moment, [0.0; 3]);
    }
    assert_eq!(rig.state().motor_strength, 0.0);
    assert_eq!(rig.state().targets, [[0.0; 3]; 14]);
    assert_eq!(rig.pose_targets(), [[0.0, 0.0, 0.0, 1.0]; 14]);
}

#[test]
fn transient_air_pose_clears_on_reset_disable_and_ineligible_posture() {
    for exit in 0..3 {
        let mut rig = running_pose();
        for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
            if collider.parent().is_none() {
                collider.set_enabled(false);
            }
        }
        rig.controller.advance(&mut rig.pose.rig, true);
        assert!(rig.state().targets[11][0].abs() > 0.4);
        match exit {
            0 => rig.reset(),
            1 => rig.set_enabled(false),
            _ => {
                let root = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
                rig.pose.rig.world.bodies[root].set_rotation(
                    Rotation::from_axis_angle(&Vector::x_axis(), std::f32::consts::PI),
                    true,
                );
                let before = rig.pose.rig.body_states();
                assert_eq!(
                    rig.controller.advance(&mut rig.pose.rig, true),
                    [Rotation::identity(); 14]
                );
                assert_eq!(rig.pose.rig.body_states(), before);
            }
        }
        assert_eq!(rig.state().targets, [[0.0; 3]; 14]);
        assert_eq!(rig.state().acceleration, [0.0; 3]);
        assert_eq!(rig.state().turn_torque, 0.0);
        assert_eq!(rig.state().stopping_moment, [0.0; 3]);
        assert_eq!(rig.state().support_moment, [0.0; 3]);
    }
}
