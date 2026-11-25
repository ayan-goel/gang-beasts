use super::rig_frame::ReadRig;
use super::{BodyPart, LocomotionConfig, support::FootSupport};
use rapier3d::na::{ComplexField, RealField};
use rapier3d::prelude::*;

const FLAT_NORMAL_TOLERANCE: f32 = 0.0001;

pub(super) fn adapt(
    rig: ReadRig<'_>,
    supports: [Option<FootSupport>; 2],
    config: &LocomotionConfig,
    blend: f32,
    targets: &mut [[f32; 3]; 14],
) {
    let Some(fallback) = supports.into_iter().flatten().next() else {
        return;
    };
    let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
    let forward = root.rotation() * -Vector::z();
    // Use the desired upright frame, retaining heading only. Following actual
    // shin/root tilt would cancel the ankle's restoring action during a fall.
    let heading = Rotation::from_axis_angle(
        &Vector::y_axis(),
        <f32 as RealField>::atan2(-forward.x, -forward.z),
    );
    for (side, parts) in [
        [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot],
        [
            BodyPart::RightThigh,
            BodyPart::RightShin,
            BodyPart::RightFoot,
        ],
    ]
    .into_iter()
    .enumerate()
    {
        let normal = supports[side].unwrap_or(fallback).normal;
        // Ignore nearly flat query-normal noise so existing flat gait stays intact.
        if 1.0 - normal.y < FLAT_NORMAL_TOLERANCE {
            continue;
        }
        let [hip, knee, ankle] = parts.map(|part| {
            rig.parts
                .definition
                .joints
                .iter()
                .position(|joint| joint.child == part)
                .unwrap()
        });
        let joints = &rig.parts.definition.joints;
        let upper = (Vector::from(joints[hip].anchor) - Vector::from(joints[knee].anchor)).norm();
        let lower = (Vector::from(joints[knee].anchor) - Vector::from(joints[ankle].anchor)).norm();
        let nominal = rotation(targets[hip])
            * (-Vector::y() * upper + rotation(targets[knee]) * -Vector::y() * lower);
        let hip_offset = Vector::from(joints[hip].anchor)
            - Vector::from(rig.parts.definition.bodies[BodyPart::Pelvis.index()].position);
        let mut desired = heading * nominal;
        let horizontal = heading * hip_offset + desired;
        // The linear angle envelope bounds sin(amplitude), reserving extension
        // for the downward stride. This virtual plane changes targets, not transforms.
        let reserve = blend
            * (upper + lower)
            * config.hip_amplitude_rad
            * vector![normal.x, 0.0, normal.z].norm()
            / normal.y;
        desired.y += reserve - (normal.x * horizontal.x + normal.z * horizontal.z) / normal.y;
        let (hip_angles, bend) = leg_angles(heading.inverse() * desired, upper, lower);
        targets[hip] = hip_angles;
        targets[knee] = [bend, 0.0, 0.0];
        let shin = heading * rotation(targets[hip]) * rotation(targets[knee]);
        // Upward support normals exclude the antipodal rotation case. Preserve
        // the IK placement when the ankle saturates; limits can leave sole error.
        let sole = Rotation::rotation_between(&Vector::y(), &normal).unwrap() * heading;
        let (x, y, z) = (shin.inverse() * sole).euler_angles();
        targets[ankle] = clamp_angles([x, y, z], joints[ankle].angular_limits);
        targets[hip] = clamp_angles(targets[hip], joints[hip].angular_limits);
        targets[knee] = clamp_angles(targets[knee], joints[knee].angular_limits);
    }
}

fn rotation(angles: [f32; 3]) -> Rotation<Real> {
    Rotation::from_euler_angles(angles[0], angles[1], angles[2])
}

fn clamp_angles(angles: [f32; 3], limits: [[f32; 2]; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| angles[axis].clamp(limits[axis][0], limits[axis][1]))
}

fn leg_angles(target: Vector<Real>, upper: f32, lower: f32) -> ([f32; 3], f32) {
    // A 1 mm margin excludes the fully folded singularity. Unreachable points
    // project onto the leg's reach; anatomical limits still bound every actuator.
    let distance = target
        .norm()
        .clamp((upper - lower).abs() + 0.001, upper + lower);
    let cosine = ((distance * distance - upper * upper - lower * lower) / (2.0 * upper * lower))
        .clamp(-1.0, 1.0);
    let bend = -<f32 as ComplexField>::acos(cosine);
    let roll = <f32 as RealField>::atan2(target.x, -target.y);
    let pitch = <f32 as RealField>::atan2(
        -target.z,
        <f32 as ComplexField>::sqrt(target.x * target.x + target.y * target.y),
    ) - <f32 as RealField>::atan2(
        lower * <f32 as ComplexField>::sin(bend),
        upper + lower * <f32 as ComplexField>::cos(bend),
    );
    ([pitch, 0.0, roll], bend)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_link_solution_reaches_nonplanar_targets() {
        for target in [
            vector![0.0, -0.6, 0.0],
            vector![0.06, -0.5, -0.2],
            vector![-0.08, -0.4, 0.3],
        ] {
            let (hip, knee) = leg_angles(target, 0.32, 0.30);
            let actual = rotation(hip)
                * (-Vector::y() * 0.32 + rotation([knee, 0.0, 0.0]) * -Vector::y() * 0.30);
            assert!((actual - target).norm() < 0.00001);
            assert!(knee <= 0.0);
        }
    }

    #[test]
    fn unreachable_targets_remain_finite_and_within_leg_reach() {
        for target in [Vector::zeros(), vector![1.0, -2.0, 3.0]] {
            let (hip, knee) = leg_angles(target, 0.32, 0.30);
            assert!(hip.iter().chain([&knee]).all(|v| v.is_finite()));
            let actual = rotation(hip)
                * (-Vector::y() * 0.32 + rotation([knee, 0.0, 0.0]) * -Vector::y() * 0.30);
            assert!((0.0209..=0.62001).contains(&actual.norm()));
        }
    }
}
