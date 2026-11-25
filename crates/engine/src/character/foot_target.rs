use super::BodyPart;
use super::rig_frame::ReadRig;
use rapier3d::prelude::*;

pub(super) fn separated(rig: ReadRig<'_>, targets: &[[f32; 3]; 14]) -> bool {
    let feet = [BodyPart::LeftFoot, BodyPart::RightFoot];
    let poses = [
        [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot],
        [
            BodyPart::RightThigh,
            BodyPart::RightShin,
            BodyPart::RightFoot,
        ],
    ]
    .map(|parts| {
        let definition = rig.parts.definition;
        let bind = |part: BodyPart| Vector::from(definition.bodies[part.index()].position);
        let mut pose = Isometry::translation(0.0, 0.0, 0.0);
        for child in parts {
            let (index, joint) = definition
                .joints
                .iter()
                .enumerate()
                .find(|(_, joint)| joint.child == child)
                .unwrap();
            let anchor = Vector::from(joint.anchor);
            let [x, y, z] = targets[index];
            let rotation = pose.rotation * Rotation::from_euler_angles(x, y, z);
            let center = pose.translation.vector + pose.rotation * (anchor - bind(joint.parent))
                - rotation * (anchor - bind(child));
            pose = Isometry::from_parts(Translation::from(center), rotation);
        }
        pose
    });
    let shapes = feet.map(|part| {
        let body = &rig.world.bodies[rig.parts.bodies[part.index()]];
        rig.world.colliders[body.colliders()[0]].shape()
    });
    // Check the actual rounded shapes: inward placement must not bypass the
    // nominal gait's clearance envelope. Unsupported shape pairs deny the change.
    rapier3d::parry::query::contact(&poses[0], shapes[0], &poses[1], shapes[1], 0.0)
        .is_ok_and(|contact| contact.is_none_or(|c| c.dist >= -0.00001))
}
