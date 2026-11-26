use super::*;
use crate::{BodyPart, MeleePhase};
use std::collections::HashSet;

#[test]
fn every_playing_skyline_fighter_owns_its_bodies_and_uses_its_anatomical_motors() {
    for count in [2, 8] {
        let mut game = Duel::skyline_match_with_players(PhysicsConfig::default(), count).unwrap();
        let mut body_handles = HashSet::new();
        let mut joint_handles = HashSet::new();
        for fighter in &game.fighters {
            let rig = fighter.parts(&game.definition, &game.physics, game.tick);
            let mut total_mass = 0.;
            for (handle, expected) in rig.bodies.into_iter().zip([
                4., 10., 6., 1.5, 1.5, 1., 1., 0.5, 0.5, 3.5, 3.5, 2.5, 2.5, 1., 1.,
            ]) {
                assert!(body_handles.insert(handle));
                let body = &game.world.bodies[handle];
                assert!(body.is_dynamic() && body.is_enabled());
                assert!((body.mass() - expected).abs() < 1.0e-4);
                total_mass += body.mass();
            }
            assert!((total_mass - 40.).abs() < 1.0e-4);
            for (handle, spec) in rig.joints.into_iter().zip(&game.definition.joints) {
                assert!(joint_handles.insert(handle));
                let joint = game.world.impulse_joints.get(handle).unwrap();
                assert_eq!(joint.body1, rig.bodies[spec.parent.index()]);
                assert_eq!(joint.body2, rig.bodies[spec.child.index()]);
            }
        }
        assert_eq!(body_handles.len(), count * 15);
        assert_eq!(joint_handles.len(), count * 14);

        for _ in 0..180 {
            game.step();
        }
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
        let mut solved = vec![[false; 14]; count];
        for tick in 0..120 {
            if tick == 60 {
                for &id in game.fighter_ids() {
                    game.set_input(id, [1., 0.], false).unwrap();
                }
            }
            game.step();
            for (slot, fighter) in game.fighters.iter().enumerate() {
                let rig = fighter.parts(&game.definition, &game.physics, game.tick);
                for (index, (handle, spec)) in rig
                    .joints
                    .into_iter()
                    .zip(&game.definition.joints)
                    .enumerate()
                {
                    let data = &game.world.impulse_joints.get(handle).unwrap().data;
                    let wrist = matches!(spec.child, BodyPart::LeftHand | BodyPart::RightHand);
                    assert_eq!(
                        data.motor_axes.is_empty(),
                        wrist,
                        "slot {slot} {:?}",
                        spec.child
                    );
                    assert!(!data.motor_axes.intersects(data.locked_axes));
                    for axis in [JointAxis::AngX, JointAxis::AngY, JointAxis::AngZ] {
                        if let Some(motor) = data.motor(axis) {
                            assert!(motor.impulse.is_finite() && motor.max_force.is_finite());
                            solved[slot][index] |= motor.impulse.abs() > 1.0e-7;
                        }
                    }
                }
            }
        }
        for activity in solved {
            for (exerted, spec) in activity.into_iter().zip(&game.definition.joints) {
                let driven = !matches!(spec.child, BodyPart::LeftHand | BodyPart::RightHand);
                assert_eq!(exerted, driven, "no solver effort from {:?}", spec.child);
            }
        }
    }
}
