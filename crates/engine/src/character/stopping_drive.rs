use super::{BodyPart, LocomotionConfig, PassiveRagdoll, support::FootSupport};
use rapier3d::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct StoppingDrive {
    pub net: Vector<Real>,
    pub foot: Vector<Real>,
    pub moment: Vector<Real>,
}

pub(super) fn allocate(
    rig: &PassiveRagdoll,
    requested: Vector<Real>,
    velocity: Vector<Real>,
    supports: [Option<FootSupport>; 2],
    strength: f32,
    config: &LocomotionConfig,
) -> StoppingDrive {
    let count = supports.iter().flatten().count() as f32;
    debug_assert!(
        count > 0.0,
        "stopping allocation requires current foot support"
    );
    let mut mass = 0.0;
    let mut center = Vector::zeros();
    for handle in rig.bodies {
        let body = &rig.world.bodies[handle];
        mass += body.mass();
        center += body.center_of_mass().coords * body.mass();
    }
    center /= mass;
    // A co-moving COM and contact point have different velocities on a rotating
    // deck. Compare at COM so deck rotation is not mistaken for foot drag.
    let velocity = velocity
        - supports
            .iter()
            .flatten()
            .fold(Vector::zeros(), |sum, support| {
                sum + support
                    .angular_velocity
                    .cross(&(center - support.contact_point.coords))
            })
            / count;
    let horizontal = vector![velocity.x, 0.0, velocity.z];
    let quantum = config.maximum_acceleration_m_s2 * crate::TICK_SECONDS;
    // Fade drag below one tick's velocity correction to avoid reversing a full
    // foot command when nearly stationary feedback changes sign.
    let fade = (horizontal.norm() / quantum.max(0.000001)).min(1.0);
    let root = &rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]];
    let up = root.rotation() * Vector::y();
    let rate = root.angvel().cross(&up);
    let gravity = rig.configuration().gravity_y.abs();
    let drag = supports
        .iter()
        .flatten()
        .fold(Vector::zeros(), |sum, support| {
            let normal = support.normal;
            let tangent = vector![
                horizontal.x,
                -(normal.x * horizontal.x + normal.z * horizontal.z) / normal.y,
                horizontal.z
            ];
            let direction = tangent / tangent.norm().max(quantum).max(0.000001);
            let load = (gravity * normal.y - requested.dot(&normal)).max(0.0);
            sum + (vector![direction.x, 0.0, direction.z] * support.friction
                - vector![normal.x, 0.0, normal.z] * fade)
                * (load / gravity)
        })
        / count;
    let mut foot = ((drag
        + vector![up.x, 0.0, up.z] * config.drive_tilt_gain
        + vector![rate.x, 0.0, rate.z] * config.drive_tilt_damping_s)
        * rig.configuration().gravity_y.abs())
    .cap_magnitude(config.maximum_acceleration_m_s2 / 2.0)
        * strength;
    let mut feet = Vector::zeros();
    for (index, part) in [BodyPart::LeftFoot, BodyPart::RightFoot]
        .into_iter()
        .enumerate()
    {
        if supports[index].is_some() {
            feet += rig.world.bodies[rig.bodies[part.index()]]
                .center_of_mass()
                .coords
                / count;
        }
    }
    // INVARIANT: the force pair has its own finite moment budget, including yaw.
    // Mass-proportional body forces have no moment about COM; equal foot shares do.
    let moment = mass * (feet - center).cross(&foot);
    let scale = (config.stopping_moment_cap_nm * strength / moment.norm().max(0.000001)).min(1.0);
    foot *= scale;
    // |net - foot| + |foot| <= |net| + 2|foot| preserves the absolute
    // linear impulse budget even when net deceleration reaches zero.
    let available = (config.maximum_acceleration_m_s2 - 2.0 * foot.norm()).max(0.0);
    StoppingDrive {
        net: requested.cap_magnitude(available),
        foot,
        moment: moment * scale,
    }
}
