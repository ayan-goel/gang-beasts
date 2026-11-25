use super::{BodyPart, MotorConfig, rig_frame::WriteRig};
use crate::{Hand, HandPhase, TICK_SECONDS, arena::Skyline, combat::Hands};
use rapier3d::prelude::*;

pub(super) fn aim(
    rig: &mut WriteRig<'_>,
    map: &Skyline,
    hands: &Hands,
    attached: [bool; 2],
    motors: &mut MotorConfig,
) {
    let chest = rig.parts.bodies[BodyPart::Chest.index()];
    let definition = rig.parts.definition;
    for hand in [Hand::Left, Hand::Right] {
        if attached[hand as usize] || hands.state(hand).phase != HandPhase::Reach {
            continue;
        }
        let parts = hand.parts();
        let shoulder = definition
            .joints
            .iter()
            .find(|j| j.child == parts[0])
            .unwrap();
        let anchor = Vector::from(shoulder.anchor);
        let rest = |p: BodyPart| Vector::from(definition.bodies[p.index()].position);
        let span = (rest(hand.body()) - anchor).norm();
        let shoulder =
            rig.world.bodies[chest].position() * Point::from(anchor - rest(BodyPart::Chest));
        let fist = rig.parts.bodies[hand.body().index()];
        let body = &rig.world.bodies[fist];
        let palm = *body.center_of_mass();
        let radius = rig.world.colliders[body.colliders()[0]]
            .shape()
            .compute_local_aabb()
            .half_extents()
            .min();
        let Some((lip, velocity)) = map.reachable_lip(rig.world, shoulder, palm, span, radius)
        else {
            continue;
        };
        let torque = hands.tuning.max_torque_nm;
        let force = ((lip - palm) * (hands.tuning.stiffness_nm_per_rad / span.powi(2))
            + (velocity - body.linvel()) * (hands.tuning.damping_nm_s_per_rad / span.powi(2)))
        .cap_magnitude(2. * torque / span);
        // Substitute endpoint control for the two arm-pose motors. The equal opposite impulse
        // at the same world point prevents reaching in mid-air from accelerating the whole fighter.
        for motor in motors
            .joints
            .iter_mut()
            .filter(|m| parts.contains(&m.child))
        {
            motor.max_torque_nm = 0.;
        }
        let impulse = force * TICK_SECONDS;
        rig.world.bodies[fist].apply_impulse(impulse, true);
        rig.world.bodies[chest].apply_impulse_at_point(-impulse, palm, true);
    }
}
