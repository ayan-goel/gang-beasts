use super::*;

fn foot_pose(
    definition: &CharacterDefinition,
    targets: [[f32; 3]; 14],
    parts: [BodyPart; 3],
) -> Isometry<Real> {
    let mut pose = Isometry::from_parts(
        Translation::from(Vector::from(
            definition.bodies[BodyPart::Pelvis.index()].position,
        )),
        Rotation::identity(),
    );
    for child in parts {
        let (index, joint) = definition
            .joints
            .iter()
            .enumerate()
            .find(|(_, joint)| joint.child == child)
            .unwrap();
        let anchor = Vector::from(joint.anchor);
        let parent_offset = anchor - Vector::from(definition.bodies[joint.parent.index()].position);
        let child_offset = anchor - Vector::from(definition.bodies[child.index()].position);
        let [x, y, z] = targets[index];
        let rotation = pose.rotation * Rotation::from_euler_angles(x, y, z);
        let center =
            pose.translation.vector + pose.rotation * parent_offset - rotation * child_offset;
        pose = Isometry::from_parts(Translation::from(center), rotation);
    }
    pose
}

pub(crate) fn foot_separation(definition: &CharacterDefinition, targets: [[f32; 3]; 14]) -> f32 {
    let feet = [BodyPart::LeftFoot, BodyPart::RightFoot]
        .map(|part| definition.bodies[part.index()].shape.collider().build());
    let poses = [
        [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot],
        [
            BodyPart::RightThigh,
            BodyPart::RightShin,
            BodyPart::RightFoot,
        ],
    ]
    .map(|parts| foot_pose(definition, targets, parts));
    // Exact rounded colliders distinguish real overlap from intersecting
    // bounding boxes while one foot passes above the other.
    rapier3d::parry::query::contact(&poses[0], feet[0].shape(), &poses[1], feet[1].shape(), 0.0)
        .unwrap()
        .map_or(0.0, |contact| contact.dist)
}

#[test]
fn directional_gait_does_not_command_intersecting_feet() {
    let definition = CharacterDefinition::bundled().unwrap();
    let config = LocomotionConfig::bundled().unwrap();
    for direction_index in 0..16 {
        let angle = direction_index as f32 * std::f32::consts::TAU / 16.0;
        let direction = vector![angle.sin(), 0.0, -angle.cos()];
        for sample in 0..120 {
            let phase = sample as f32 * std::f32::consts::TAU / 120.0;
            let targets = targets(&definition, &config, direction, phase, 1.0);
            let separation = foot_separation(&definition, targets);
            assert!(
                separation >= -0.00001,
                "direction {direction_index}, phase sample {sample}, separation {separation}"
            );
        }
    }
}
