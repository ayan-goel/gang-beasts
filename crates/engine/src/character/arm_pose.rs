use super::{BodyPart, MotorConfig, rig_frame::ReadRig};
use crate::{Hand, combat::Hands};
use rapier3d::prelude::*;

// The base motor blend is rebuilt each tick so a released arm regains its gait caps.
pub(super) fn apply(
    hands: &mut Hands,
    rig: ReadRig<'_>,
    motors: &mut MotorConfig,
    feet: [bool; 2],
    angles: &mut [[f32; 3]; 14],
    targets: &mut [Rotation<Real>; 14],
) {
    let mut bracing = false;
    for hand in [Hand::Left, Hand::Right] {
        if let Some(pitch) = hands.advance(hand) {
            bracing = true;
            for (part, angle) in hand.parts().into_iter().zip(pitch) {
                let index = rig
                    .parts
                    .definition
                    .joints
                    .iter()
                    .position(|j| j.child == part)
                    .expect("anatomical arm joint");
                let [min, max] = rig.parts.definition.joints[index].angular_limits[0];
                let angle = angle.clamp(min, max);
                angles[index] = [angle, 0., 0.];
                targets[index] = Rotation::from_euler_angles(angle, 0., 0.);
                let motor = motors
                    .joints
                    .iter_mut()
                    .find(|m| m.child == part)
                    .expect("arm motor preset");
                motor.stiffness_nm_per_rad = hands.tuning.stiffness_nm_per_rad;
                motor.damping_nm_s_per_rad = hands.tuning.damping_nm_s_per_rad;
                motor.max_torque_nm = hands.tuning.max_torque_nm;
            }
        }
    }
    if bracing && feet.contains(&true) {
        counterbalance(rig, motors, feet, angles, targets);
    }
}

fn counterbalance(
    rig: ReadRig<'_>,
    motors: &MotorConfig,
    feet: [bool; 2],
    angles: &mut [[f32; 3]; 14],
    targets: &mut [Rotation<Real>; 14],
) {
    // Forward arm weight creates a pitching moment even with both feet planted.
    // Bias existing motor targets by torque / stiffness, within their ordinary effort caps.
    let chest = &rig.world.bodies[rig.parts.bodies[BodyPart::Chest.index()]];
    let right = chest.rotation() * Vector::x();
    let gravity = Vector::y() * rig.parts.physics.gravity_y;
    let mut moment = 0.0;
    for part in [
        BodyPart::LeftUpperArm,
        BodyPart::RightUpperArm,
        BodyPart::LeftForearm,
        BodyPart::RightForearm,
        BodyPart::LeftHand,
        BodyPart::RightHand,
    ] {
        let body = &rig.world.bodies[rig.parts.bodies[part.index()]];
        let arm = body.center_of_mass().coords - chest.center_of_mass().coords;
        moment += arm.cross(&(gravity * body.mass())).dot(&right);
    }
    let count = feet.into_iter().filter(|f| *f).count() as f32;
    for (index, joint) in rig.parts.definition.joints.iter().enumerate() {
        let torque = match joint.child {
            BodyPart::Chest => -moment,
            BodyPart::LeftFoot if feet[0] => moment / count,
            BodyPart::RightFoot if feet[1] => moment / count,
            _ => continue,
        };
        let motor = motors
            .joints
            .iter()
            .find(|m| m.child == joint.child)
            .unwrap();
        if motor.stiffness_nm_per_rad <= 0.0 {
            continue;
        }
        let offset = (torque / motor.stiffness_nm_per_rad).clamp(-0.20, 0.20);
        let [min, max] = joint.angular_limits[0];
        angles[index][0] = (angles[index][0] + offset).clamp(min, max);
        let [x, y, z] = angles[index];
        targets[index] = Rotation::from_euler_angles(x, y, z);
    }
}
