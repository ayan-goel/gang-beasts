use super::*;

#[test]
fn slope_departure_pose_rate_does_not_depend_on_remaining_motor_strength() {
    let config = LocomotionConfig::bundled().unwrap();
    for strength_limit in [0.0, 0.34, 0.07] {
        let (mut rig, _, _) = slope(15.0, 0.0);
        for _ in 0..120 {
            rig.step();
        }
        if strength_limit > 0.0 {
            rig.set_input([0.0, -1.0], true).unwrap();
            for _ in 0..45 {
                rig.step();
            }
            rig.set_input([0.0; 2], false).unwrap();
            for _ in 0..600 {
                rig.step();
                if rig.state().motor_strength <= strength_limit {
                    break;
                }
            }
            assert!(rig.state().motor_strength > 0.0);
        }
        assert!(rig.state().motor_strength <= strength_limit);
        assert!(rig.state().targets[12][0].abs() > 0.1);
        for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
            if collider.parent().is_none() {
                collider.set_enabled(false);
            }
        }
        let bodies = rig.pose.rig.body_states();
        for _ in 0..(config.motor_release_s / crate::TICK_SECONDS).ceil() as u32 + 1 {
            let before = rig.state().targets;
            rig.controller.advance(&mut rig.pose.rig, true);
            assert_eq!(rig.pose.rig.body_states(), bodies);
            assert_eq!(rig.state().acceleration, [0.0; 3]);
            assert_eq!(rig.state().turn_torque, 0.0);
            assert_eq!(rig.state().support_moment, [0.0; 3]);
            assert_eq!(rig.state().stopping_moment, [0.0; 3]);
            for (index, joint) in rig.pose.rig.definition.joints.iter().enumerate() {
                for (axis, [lo, hi]) in joint.angular_limits.iter().enumerate() {
                    let change = (rig.state().targets[index][axis] - before[index][axis]).abs();
                    let bound = (hi - lo) * crate::TICK_SECONDS / config.motor_release_s;
                    assert!(
                        change <= bound + 0.000001,
                        "strength limit={strength_limit}, joint={index}, axis={axis}, change={change}, bound={bound}"
                    );
                }
            }
        }
        assert_eq!(rig.state().targets, [[0.0; 3]; 14]);
    }
}
