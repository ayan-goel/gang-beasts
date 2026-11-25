use super::{BodyPart, rig_frame::WriteRig};
use crate::{Hand, TICK_SECONDS, combat::HandTuning};
use rapier3d::prelude::*;

pub(super) fn apply(
    rig: &mut WriteRig<'_>,
    tuning: &HandTuning,
    lifting: [bool; 2],
    balance: &super::BalanceConfig,
    targets: &mut [Rotation<Real>; 14],
) {
    let chest = rig.parts.bodies[BodyPart::Chest.index()];
    // Bracing on another unsupported fighter could propel the pair in midair.
    let grounded = [BodyPart::LeftFoot, BodyPart::RightFoot]
        .into_iter()
        .any(|foot| {
            super::support::foot_support(
                rig.world,
                rig.parts.bodies[foot.index()],
                &rig.parts.bodies,
                balance,
            )
            .is_some_and(|s| {
                s.body
                    .is_none_or(|body| !rig.world.bodies[body].is_dynamic())
            })
        });
    for hand in [Hand::Left, Hand::Right] {
        if !lifting[hand as usize] {
            continue;
        }
        for (part, pitch) in hand.parts().into_iter().zip(tuning.lift_pose_rad) {
            let index = rig
                .parts
                .definition
                .joints
                .iter()
                .position(|j| j.child == part)
                .unwrap();
            let [min, max] = rig.parts.definition.joints[index].angular_limits[0];
            targets[index] = Rotation::from_euler_angles(pitch.clamp(min, max), 0., 0.);
        }
        if !grounded {
            continue;
        }
        let fist = rig.parts.bodies[hand.body().index()];
        let point = *rig.world.bodies[fist].center_of_mass();
        let torso = &rig.world.bodies[chest];
        let gap = torso.center_of_mass().y + tuning.lift_chest_clearance_m - point.y;
        let speed = rig.world.bodies[fist].linvel().y - torso.velocity_at_point(&point).y;
        let fraction = ((gap - speed * 0.06) / 0.1).clamp(0., 1.);
        let impulse = Vector::y() * (tuning.lift_force_per_hand_n * fraction * TICK_SECONDS);
        // Grounded assistance is divided between the raised hand and bracing torso.
        // The opponent receives force through the existing grip and physical contacts.
        rig.world.bodies[fist].apply_impulse(impulse * 0.75, true);
        rig.world.bodies[chest].apply_impulse(impulse * 0.25, true);
    }
}
