use super::{BodyPart, LocomotionConfig, PassiveRagdoll, support::FootSupport};
use rapier3d::prelude::*;

pub(super) struct SupportDrive {
    pub net: Vector<Real>,
    pub foot: Vector<Real>,
    pub moment: Vector<Real>,
    pub scale: f32,
}

pub(super) fn allocate(
    rig: &PassiveRagdoll,
    supports: [Option<FootSupport>; 2],
    velocity: Vector<Real>,
    requested: Vector<Real>,
    base_foot: Vector<Real>,
    config: &LocomotionConfig,
) -> Option<SupportDrive> {
    let count = supports.iter().flatten().count() as f32;
    debug_assert!(count > 0.0, "support drive requires current foot support");
    // Sample the rotating velocity field along COM motion. Removing the
    // support body's translation keeps this estimate independent of reference frame.
    let mut extra = supports
        .iter()
        .flatten()
        .map(|support| {
            -support
                .angular_velocity
                .cross(&(velocity - support.linear_velocity))
        })
        .sum::<Vector<Real>>()
        / count;
    extra.y = 0.0;
    let magnitude = extra.norm();
    if magnitude == 0.0 || config.support_moment_cap_nm == 0.0 {
        return None;
    }
    let direction = -extra / magnitude;
    // Only contact-supplied carry has the foot lever arm. Subtract existing
    // distributed assistance, and never compensate more than the support load.
    let residual = (magnitude - (requested - base_foot).dot(&direction)).clamp(0.0, magnitude);
    extra = -direction * residual;
    if extra.norm_squared() == 0.0 {
        return None;
    }
    let mass = rig
        .bodies
        .iter()
        .map(|h| rig.world.bodies[*h].mass())
        .sum::<f32>();
    let center = rig
        .bodies
        .iter()
        .map(|h| {
            let body = &rig.world.bodies[*h];
            body.center_of_mass().coords * body.mass()
        })
        .sum::<Vector<Real>>()
        / mass;
    let feet = [BodyPart::LeftFoot, BodyPart::RightFoot]
        .into_iter()
        .zip(supports)
        .filter(|(_, support)| support.is_some())
        .map(|(part, _)| {
            rig.world.bodies[rig.bodies[part.index()]]
                .center_of_mass()
                .coords
        })
        .sum::<Vector<Real>>()
        / count;
    // INVARIANT: compensation has a separate COM-moment ceiling even at idle.
    // This permits deck carry without an unlimited world-space balance actuator.
    let moment = mass * (feet - center).cross(&extra);
    extra *= (config.support_moment_cap_nm / moment.norm().max(0.000001)).min(1.0);
    let foot = base_foot + extra;
    // Rebudget the combined command; a correction cannot add an uncharged impulse.
    let cost = (requested - foot).norm() + foot.norm();
    let scale = (config.maximum_acceleration_m_s2 / cost.max(0.000001)).min(1.0);
    Some(SupportDrive {
        net: requested * scale,
        foot: foot * scale,
        moment: mass * (feet - center).cross(&extra) * scale,
        scale,
    })
}
