use super::{BodyPart, PassiveRagdoll, RecoveryConfig};
use crate::TICK_SECONDS;
use rapier3d::prelude::*;

pub(super) fn apply(rig: &mut PassiveRagdoll, strength: f32, config: &RecoveryConfig) -> [f32; 2] {
    // Distribute each half of the force pair by measured mass to avoid concentrating
    // a full-body correction on one joint or foot.
    let mut mass = [0.; 2];
    let mut center = [Vector::zeros(); 2];
    let mut velocity = [Vector::zeros(); 2];
    for part in BodyPart::ALL {
        let group = usize::from(part.index() >= BodyPart::LeftThigh.index());
        let body = &rig.world.bodies[rig.bodies[part.index()]];
        mass[group] += body.mass();
        center[group] += body.center_of_mass().coords * body.mass();
        velocity[group] += body.linvel() * body.mass();
    }
    for i in 0..2 {
        center[i] /= mass[i];
        velocity[i] /= mass[i];
    }
    let arm = center[0] - center[1];
    let length = arm.norm();
    if length < 0.05 {
        return [0.0; 2];
    }
    let axis = arm / length;
    let angle = Rotation::rotation_between(&axis, &Vector::y())
        .unwrap_or_else(|| Rotation::from_axis_angle(&Vector::x_axis(), std::f32::consts::PI))
        .scaled_axis();
    let spin = arm.cross(&(velocity[0] - velocity[1])) / length.powi(2);
    let feet = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
        rig.world.bodies[rig.bodies[part.index()]]
            .center_of_mass()
            .coords
    });
    let total = mass[0] + mass[1];
    let from_feet = (mass[0] * center[0] + mass[1] * center[1]) / total - (feet[0] + feet[1]) * 0.5;
    let gravity = rig.configuration().gravity_y;
    let inertia = total * from_feet.norm_squared();
    // The feet provide a pivot estimate for gravity compensation. Project the requested
    // moment onto the realizable force pair and report the moment actually commanded.
    let moment = ((angle / config.response_s.powi(2) - spin * (2.0 / config.response_s)) * inertia
        - from_feet.cross(&(Vector::y() * gravity * total)))
    .cap_magnitude(config.moment_cap_nm * strength);
    let force = (moment.cross(&arm) / length.powi(2)).cap_magnitude(config.force_cap_n * strength);
    for part in BodyPart::ALL {
        let group = usize::from(part.index() >= BodyPart::LeftThigh.index());
        let body = &mut rig.world.bodies[rig.bodies[part.index()]];
        let signed = if group == 0 { 1. } else { -1. };
        body.apply_impulse(
            force * (signed * body.mass() / mass[group] * TICK_SECONDS),
            true,
        );
    }
    [force.norm(), arm.cross(&force).norm()]
}
