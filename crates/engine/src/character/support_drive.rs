use super::rig_frame::ReadRig;
use super::{BodyPart, LocomotionConfig, support::FootSupport};
use rapier3d::prelude::*;

pub(super) struct SupportDrive {
    pub net: Vector<Real>,
    pub foot: Vector<Real>,
    pub moment: Vector<Real>,
    pub scale: f32,
}

pub(super) fn allocate(
    rig: ReadRig<'_>,
    supports: [Option<FootSupport>; 2],
    velocity: Vector<Real>,
    requested: Vector<Real>,
    base_foot: Vector<Real>,
    steering: bool,
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
    if config.support_moment_cap_nm == 0.0 || (magnitude == 0.0 && !steering) {
        return None;
    }
    if magnitude > 0.0 {
        let direction = -extra / magnitude;
        let residual = (magnitude - (requested - base_foot).dot(&direction)).clamp(0.0, magnitude);
        extra = -direction * residual;
    }
    let mass = rig
        .parts
        .bodies
        .iter()
        .map(|h| rig.world.bodies[*h].mass())
        .sum::<f32>();
    let center = rig
        .parts
        .bodies
        .iter()
        .map(|h| {
            let body = &rig.world.bodies[*h];
            body.center_of_mass().coords * body.mass()
        })
        .sum::<Vector<Real>>()
        / mass;
    if steering {
        let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
        let up = root.rotation() * Vector::y();
        let rate = root.angvel().cross(&up);
        let surface = supports
            .iter()
            .flatten()
            // Compare both velocities at COM. A rotating deck has different
            // contact-point velocities even when the character moves with it.
            .map(|s| s.velocity + s.angular_velocity.cross(&(center - s.contact_point.coords)))
            .sum::<Vector<Real>>()
            / count;
        let relative = velocity - surface;
        let horizontal = vector![relative.x, 0.0, relative.z];
        let motion = horizontal
            / horizontal
                .norm()
                .max(config.maximum_acceleration_m_s2 * crate::TICK_SECONDS)
                .max(0.000001);
        let g = rig.parts.physics.gravity_y.abs();
        let drag = supports
            .iter()
            .flatten()
            .map(|s| {
                let n = s.normal;
                let tangent = vector![
                    horizontal.x,
                    -(n.x * horizontal.x + n.z * horizontal.z) / n.y,
                    horizontal.z
                ];
                let direction = tangent
                    / tangent
                        .norm()
                        .max(config.maximum_acceleration_m_s2 * crate::TICK_SECONDS)
                        .max(0.000001);
                let normal_load = (g * n.y - requested.dot(&n)).max(0.0);
                vector![
                    direction.x * s.friction - n.x,
                    0.0,
                    direction.z * s.friction - n.z
                ] * normal_load
            })
            .sum::<Vector<Real>>()
            / count;
        let target = drag
            + (vector![up.x, 0.0, up.z] * config.drive_tilt_gain
                + vector![rate.x, 0.0, rate.z] * config.drive_tilt_damping_s)
                * g;
        let mut correction = target - base_foot;
        if let Some(direction) = requested.try_normalize(0.001) {
            let alignment = direction.dot(&motion);
            let brake = alignment.min(0.0).powi(2);
            correction -= direction * correction.dot(&direction).min(0.0) * (1.0 - brake);
        }
        extra += correction;
    }
    if extra.norm_squared() == 0.0 {
        return None;
    }
    let feet = [BodyPart::LeftFoot, BodyPart::RightFoot]
        .into_iter()
        .zip(supports)
        .filter(|(_, support)| support.is_some())
        .map(|(part, _)| {
            rig.world.bodies[rig.parts.bodies[part.index()]]
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
