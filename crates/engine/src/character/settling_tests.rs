use super::*;
use rapier3d::prelude::*;

#[test]
fn standing_motor_release_requires_each_trunk_segment_to_be_upright() {
    let config = LocomotionConfig::bundled().unwrap();
    let release_step = crate::TICK_SECONDS / config.motor_release_s;
    let mut strengths = Vec::new();
    for folded in [None, Some(BodyPart::Chest), Some(BodyPart::Head)] {
        let mut rig =
            LocomotionRagdoll::new(PhysicsConfig::default(), config.clone(), [0.0; 3]).unwrap();
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([0.0, -1.0], false).unwrap();
        rig.step();
        rig.set_input([0.0; 2], false).unwrap();
        for _ in 0..config.braking_ticks {
            rig.step();
        }
        assert_eq!(rig.state().motor_strength, 1.0);

        // Hold the incoming motion and actual foot contacts identical across all three trials.
        for handle in rig.pose.rig.bodies {
            let body = &mut rig.pose.rig.world.bodies[handle];
            body.set_linvel(Vector::zeros(), true);
            body.set_angvel(Vector::zeros(), true);
        }
        for segment in [BodyPart::Chest, BodyPart::Head] {
            let handle = rig.pose.rig.bodies[segment.index()];
            let pitch = if Some(segment) == folded { 0.3 } else { 0.0 };
            rig.pose.rig.world.bodies[handle]
                .set_rotation(Rotation::from_euler_angles(pitch, 0.0, 0.0), true);
        }
        rig.step();
        assert_eq!(rig.state().feet, [true, true]);
        strengths.push(rig.state().motor_strength);
    }
    assert_eq!(strengths, [1.0 - release_step, 1.0, 1.0]);
}

#[test]
fn a_stumble_requires_whole_trunk_recovery_before_drive_resumes() {
    for part in [BodyPart::Pelvis, BodyPart::Chest, BodyPart::Head] {
        let mut rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.0; 3],
        )
        .unwrap();
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([1.0, 0.0], true).unwrap();
        let h = rig.pose.rig.bodies[part.index()];
        for (up, assisted) in [(0.8_f32, true), (0.4, false), (0.8, false), (1.0, true)] {
            rig.pose.rig.world.bodies[h].set_rotation(
                Rotation::from_axis_angle(&Vector::x_axis(), up.acos()),
                true,
            );
            rig.controller.advance(&mut rig.pose.rig, true);
            assert!(rig.state().feet.iter().any(|f| *f));
            assert_eq!(
                rig.state().acceleration != [0.0; 3],
                assisted,
                "{part:?} up={up}"
            );
        }
    }
}
