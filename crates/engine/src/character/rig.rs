use super::CharacterDefinition;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

pub(crate) fn insert_rig(
    world: &mut PhysicsWorld,
    definition: &CharacterDefinition,
    spawn: [f32; 3],
) -> [RigidBodyHandle; 15] {
    let bodies = std::array::from_fn(|index| {
        let body = &definition.bodies[index];
        let position = Vector::from(body.position) + Vector::from(spawn);
        let handle = world
            .bodies
            .insert(RigidBodyBuilder::dynamic().translation(position));
        world.colliders.insert_with_parent(
            body.shape
                .collider()
                .mass(body.mass)
                .friction(0.7)
                .restitution(0.0),
            handle,
            &mut world.bodies,
        );
        handle
    });
    for joint in &definition.joints {
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
        // One authored world-space anchor becomes two local anchors; identity
        // bind rotations keep every joint frame aligned before the first solve.
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
        );
    }
    bodies
}
