use super::{BodyPart, CharacterDefinition, LocomotionConfig};
use rapier3d::na::ComplexField;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "gait_geometry_tests.rs"]
pub(super) mod geometry_tests;

pub(super) fn targets(
    definition: &CharacterDefinition,
    config: &LocomotionConfig,
    local: Vector<Real>,
    phase: f32,
    blend: f32,
) -> [[f32; 3]; 14] {
    let mut targets = [[0.0; 3]; 14];
    let lateral = local.x * lateral_scale(definition, config.hip_amplitude_rad);
    // The stance sweep and raised return use different paths. Half the knee
    // flexion is offset at the hip/ankle to lift the foot beneath the body.
    let legs = [0.0, 0.5].map(|offset| {
        let cycle = (phase / std::f32::consts::TAU + offset).fract();
        let hip = if cycle < 0.5 {
            1.0 - 4.0 * cycle
        } else {
            4.0 * cycle - 3.0
        };
        let knee = if cycle < 0.5 {
            0.0
        } else {
            // Match Rapier's deterministic math path for native/WASM pose targets.
            -<f32 as ComplexField>::sin((cycle - 0.5) * std::f32::consts::TAU)
        };
        [
            hip * config.hip_amplitude_rad * blend,
            knee * config.knee_amplitude_rad * blend,
        ]
    });
    for (index, joint) in definition.joints.iter().enumerate() {
        let side = match joint.child {
            BodyPart::LeftThigh
            | BodyPart::LeftShin
            | BodyPart::LeftFoot
            | BodyPart::LeftUpperArm => 0,
            _ => 1,
        };
        let [swing, knee] = legs[side];
        let angles = match joint.child {
            BodyPart::LeftThigh | BodyPart::RightThigh => {
                [-local.z * swing - knee * 0.5, 0.0, lateral * swing]
            }
            BodyPart::LeftShin | BodyPart::RightShin => [knee, 0.0, 0.0],
            BodyPart::LeftFoot | BodyPart::RightFoot => {
                [local.z * swing - knee * 0.5, 0.0, -lateral * swing]
            }
            BodyPart::LeftUpperArm | BodyPart::RightUpperArm => [local.z * swing, 0.0, 0.0],
            _ => [0.0; 3],
        };
        targets[index] = std::array::from_fn(|axis| {
            angles[axis].clamp(joint.angular_limits[axis][0], joint.angular_limits[axis][1])
        });
    }
    targets
}

fn lateral_scale(definition: &CharacterDefinition, amplitude: f32) -> f32 {
    let legs = [
        [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot],
        [
            BodyPart::RightThigh,
            BodyPart::RightShin,
            BodyPart::RightFoot,
        ],
    ];
    let mut scale = 1.0_f32;
    for parts in legs {
        let anchors = parts.map(|part| {
            Vector::from(
                definition
                    .joints
                    .iter()
                    .find(|j| j.child == part)
                    .unwrap()
                    .anchor,
            )
        });
        let body = &definition.bodies[parts[2].index()];
        let bounds = body.shape.collider().build().compute_aabb();
        let margin = (body.position[0].abs() - bounds.half_extents().x).max(0.0);
        let reach = (anchors[0] - anchors[1]).norm() + (anchors[1] - anchors[2]).norm();
        let sole = (Vector::from(body.position) - anchors[2]).norm() + body.shape.bounding_radius();
        // Bound displacement from both hip roll and counter-rotating ankle roll.
        // A whole-cycle envelope avoids discontinuous corrections at foot overlap.
        scale = scale.min(margin / (amplitude * (reach + 2.0 * sole)).max(0.000001));
    }
    scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_returns_above_the_stance_path_and_extends_before_touchdown() {
        let definition = CharacterDefinition::bundled().unwrap();
        let config = LocomotionConfig::bundled().unwrap();
        for (phase, foot) in [
            (std::f32::consts::FRAC_PI_2, 1),
            (std::f32::consts::PI * 1.5, 0),
        ] {
            let pose = targets(&definition, &config, -Vector::z(), phase, 1.0);
            let hip = pose[8 + foot][0];
            let knee = pose[10 + foot][0];
            let vertical_reach = 0.32 * hip.cos() + 0.30 * (hip + knee).cos();
            assert!(0.62 - vertical_reach > 0.04);
            assert_eq!(pose[10 + (1 - foot)][0], 0.0);
        }
        let touchdown = targets(&definition, &config, -Vector::z(), 0.0, 1.0);
        assert_eq!(touchdown[10][0], 0.0);
        assert!(touchdown[8][0] > 0.0);
    }
}
