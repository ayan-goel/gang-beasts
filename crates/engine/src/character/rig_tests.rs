use super::{BodyPart, CharacterDefinition, rig::insert_rig};
use crate::{PhysicsConfig, physics::PhysicsWorld};
use rapier3d::prelude::*;

#[test]
fn anatomical_constraints_have_authored_limits_and_no_motors() {
    let definition = CharacterDefinition::bundled().unwrap();
    let mut world = PhysicsWorld::new(&PhysicsConfig::default());
    let bodies = insert_rig(&mut world, &definition, [0.0, 0.0, 0.0]);
    for (_, joint) in world.impulse_joints.iter() {
        let child = BodyPart::ALL[bodies
            .iter()
            .position(|handle| *handle == joint.body2)
            .unwrap()];
        let authored = definition
            .joints
            .iter()
            .find(|joint| joint.child == child)
            .unwrap();
        assert_eq!(joint.body1, bodies[authored.parent.index()]);
        assert!(joint.data.motor_axes.is_empty());
        assert!(
            joint
                .data
                .locked_axes
                .contains(JointAxesMask::LOCKED_SPHERICAL_AXES)
        );
        for ((axis, mask), limits) in [
            (JointAxis::AngX, JointAxesMask::ANG_X),
            (JointAxis::AngY, JointAxesMask::ANG_Y),
            (JointAxis::AngZ, JointAxesMask::ANG_Z),
        ]
        .into_iter()
        .zip(authored.angular_limits)
        {
            if limits == [0.0, 0.0] {
                assert!(joint.data.locked_axes.contains(mask));
            } else {
                assert!(!joint.data.locked_axes.contains(mask));
                let actual = joint.data.limits(axis).unwrap();
                assert_eq!([actual.min, actual.max], limits);
            }
        }
    }
}

#[test]
fn adjacent_contacts_are_suppressed_but_self_and_opponent_contacts_survive() {
    let definition = CharacterDefinition::bundled().unwrap();
    let mut world = PhysicsWorld::new(&PhysicsConfig::default());
    world
        .colliders
        .insert(ColliderBuilder::cuboid(6.0, 0.5, 6.0).translation(vector![0.0, -0.5, 0.0]));
    let first = insert_rig(&mut world, &definition, [-0.2, 1.0, 0.0]);
    let second = insert_rig(&mut world, &definition, [0.2, 1.3, 0.0]);
    let mut own_contact = false;
    let mut opponent_contact = false;
    for _ in 0..600 {
        world.step();
        for contact in world
            .narrow_phase
            .contact_pairs()
            .filter(|pair| pair.has_any_active_contact)
        {
            let owners = [contact.collider1, contact.collider2].map(|collider| {
                let body = world.colliders[collider].parent()?;
                first
                    .iter()
                    .position(|handle| *handle == body)
                    .map(|part| (0, BodyPart::ALL[part]))
                    .or_else(|| {
                        second
                            .iter()
                            .position(|handle| *handle == body)
                            .map(|part| (1, BodyPart::ALL[part]))
                    })
            });
            if let [Some((a, part_a)), Some((b, part_b))] = owners {
                if a == b {
                    assert_ne!(part_a.parent(), Some(part_b));
                    assert_ne!(part_b.parent(), Some(part_a));
                    own_contact = true;
                } else {
                    opponent_contact = true;
                }
            }
        }
    }
    assert!(own_contact, "nonadjacent limbs never contacted");
    assert!(opponent_contact, "different characters never contacted");
}
