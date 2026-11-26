use super::CharacterDefinition;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

pub(crate) fn insert_rig(
    world: &mut PhysicsWorld,
    definition: &CharacterDefinition,
    spawn: [f32; 3],
) -> ([RigidBodyHandle; 15], [ImpulseJointHandle; 14]) {
    insert_oriented_rig(world, definition, spawn, Rotation::identity())
}

pub(super) fn insert_oriented_rig(
    world: &mut PhysicsWorld,
    definition: &CharacterDefinition,
    spawn: [f32; 3],
    heading: Rotation<Real>,
) -> ([RigidBodyHandle; 15], [ImpulseJointHandle; 14]) {
    let bodies = std::array::from_fn(|index| {
        let body = &definition.bodies[index];
        let position = heading * Vector::from(body.position) + Vector::from(spawn);
        let handle = world.bodies.insert(
            RigidBodyBuilder::dynamic()
                .position(Isometry::from_parts(Translation::from(position), heading)),
        );
        world.colliders.insert_with_parent(
            body.shape
                .collider()
                .mass(body.mass)
                .friction(0.7)
                .restitution(0.0),
            handle,
            &mut world.bodies,
        );
        if matches!(
            body.part,
            super::BodyPart::LeftHand | super::BodyPart::RightHand
        ) {
            world.track_fast_body(handle, body.shape.bounding_radius());
        }
        handle
    });
    let joints = std::array::from_fn(|index| {
        let joint = &definition.joints[index];
        let mut locked = JointAxesMask::LOCKED_SPHERICAL_AXES;
        for (axis, limits) in [
            JointAxesMask::ANG_X,
            JointAxesMask::ANG_Y,
            JointAxesMask::ANG_Z,
        ]
        .into_iter()
        .zip(joint.angular_limits)
        {
            if limits == [0.0, 0.0] {
                locked |= axis;
            }
        }
        // One authored world-space anchor becomes two local anchors; a shared
        // spawn heading rotates both connected bodies and preserves local anchors.
        let parent = &definition.bodies[joint.parent.index()];
        let child = &definition.bodies[joint.child.index()];
        let mut builder = GenericJointBuilder::new(locked)
            .local_anchor1(Point::from(
                Vector::from(joint.anchor) - Vector::from(parent.position),
            ))
            .local_anchor2(Point::from(
                Vector::from(joint.anchor) - Vector::from(child.position),
            ))
            .contacts_enabled(false);
        for (axis, limits) in [JointAxis::AngX, JointAxis::AngY, JointAxis::AngZ]
            .into_iter()
            .zip(joint.angular_limits)
        {
            if limits != [0.0, 0.0] {
                builder = builder.limits(axis, limits);
            }
        }
        // Disable only the adjacent pair; global character masks would also
        // remove necessary self and opponent contacts in later fights.
        world.impulse_joints.insert(
            bodies[joint.parent.index()],
            bodies[joint.child.index()],
            builder,
            true,
        )
    });
    (bodies, joints)
}

#[cfg(test)]
#[path = "fast_hand_tests.rs"]
mod fast_hand_tests;
