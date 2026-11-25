use super::BalanceConfig;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "support_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "support_normal_tests.rs"]
mod normal_tests;

fn current_pose(world: &PhysicsWorld, collider: &Collider) -> Isometry<Real> {
    match collider.parent() {
        Some(body) => world.bodies[body].position() * collider.position_wrt_parent().unwrap(),
        None => *collider.position(),
    }
}

fn contact_normal(
    surface: &Collider,
    pose: &Isometry<Real>,
    point: Point<Real>,
    query_normal: Vector<Real>,
) -> Vector<Real> {
    let Some(cuboid) = surface.shape().as_cuboid() else {
        return query_normal;
    };
    let local = pose.inverse_transform_point(&point);
    let gaps = cuboid.half_extents - local.coords.abs();
    // Near touching, the generic rounded-shape query can return a tilted normal
    // inside a flat face. Use that face's current orientation, not a cached manifold.
    let coordinate_scale = cuboid
        .half_extents
        .max()
        .max(pose.translation.vector.abs().max())
        .max(point.coords.abs().max())
        .max(1.0);
    let tolerance = 32.0 * f32::EPSILON * coordinate_scale;
    for axis in 0..3 {
        // Include world-coordinate subtraction error in this margin. At edges
        // and corners the contact normal may combine several face directions.
        if gaps[axis].abs() <= tolerance
            && (0..3).all(|other| other == axis || gaps[other] > tolerance)
        {
            let mut normal = Vector::zeros();
            normal[axis] = if local[axis] >= 0.0 { 1.0 } else { -1.0 };
            return pose.rotation * normal;
        }
    }
    query_normal
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FootSupport {
    pub body: Option<RigidBodyHandle>,
    pub velocity: Vector<Real>,
    pub angular_velocity: Vector<Real>,
    pub linear_velocity: Vector<Real>,
    pub contact_point: Point<Real>,
    pub normal: Vector<Real>,
    pub friction: f32,
    collider: ColliderHandle,
    distance: f32,
}

pub(super) fn foot_supported(
    world: &PhysicsWorld,
    foot: RigidBodyHandle,
    own_bodies: &[RigidBodyHandle],
    config: &BalanceConfig,
) -> bool {
    foot_support(world, foot, own_bodies, config).is_some()
}

pub(super) fn foot_support(
    world: &PhysicsWorld,
    foot: RigidBodyHandle,
    own_bodies: &[RigidBodyHandle],
    config: &BalanceConfig,
) -> Option<FootSupport> {
    if !world.bodies[foot].is_enabled() {
        return None;
    }
    let mut selected: Option<FootSupport> = None;
    for handle in world.bodies[foot].colliders() {
        let foot_collider = &world.colliders[*handle];
        if !foot_collider.is_enabled() || foot_collider.is_sensor() {
            continue;
        }
        for pair in world.narrow_phase.contact_pairs_with(*handle) {
            if !pair.has_any_active_contact
                || !pair.manifolds.iter().any(|manifold| {
                    manifold
                        .data
                        .solver_flags
                        .contains(SolverFlags::COMPUTE_IMPULSES)
                })
            {
                continue;
            }
            let other_handle = if pair.collider1 == *handle {
                pair.collider2
            } else {
                pair.collider1
            };
            let Some(other) = world.colliders.get(other_handle) else {
                continue;
            };
            // Enabled flags change before Rapier rebuilds cached contact pairs.
            // Removed support must not supply one final tick of balance or drive.
            if !other.is_enabled()
                || other.is_sensor()
                || other.parent().is_some_and(|body| {
                    own_bodies.contains(&body) || !world.bodies[body].is_enabled()
                })
            {
                continue;
            }
            // Cached normals alone survive a sideways departure from a finite ledge.
            // Re-query current shapes; unsupported shape queries conservatively give no assistance.
            let surface_pose = current_pose(world, other);
            let Ok(Some(contact)) = rapier3d::parry::query::contact(
                &current_pose(world, foot_collider),
                foot_collider.shape(),
                &surface_pose,
                other.shape(),
                config.maximum_contact_distance_m,
            ) else {
                continue;
            };
            let normal = contact_normal(other, &surface_pose, contact.point2, *contact.normal2);
            if normal.y < config.minimum_normal_y {
                continue;
            }
            let surface_velocity = other.parent().map_or(Vector::zeros(), |body| {
                world.bodies[body].velocity_at_point(&contact.point2)
            });
            let relative_velocity =
                world.bodies[foot].velocity_at_point(&contact.point1) - surface_velocity;
            if relative_velocity.dot(&normal) <= config.maximum_separation_speed_m_s {
                let candidate = FootSupport {
                    body: other.parent(),
                    friction: pair
                        .manifolds
                        .iter()
                        .filter(|m| m.data.solver_flags.contains(SolverFlags::COMPUTE_IMPULSES))
                        .flat_map(|m| &m.data.solver_contacts)
                        .map(|contact| contact.friction)
                        .min_by(f32::total_cmp)
                        .unwrap_or(0.0),
                    velocity: surface_velocity,
                    linear_velocity: other
                        .parent()
                        .map_or(Vector::zeros(), |body| *world.bodies[body].linvel()),
                    contact_point: contact.point2,
                    angular_velocity: other
                        .parent()
                        .map_or(Vector::zeros(), |body| *world.bodies[body].angvel()),
                    normal,
                    collider: other_handle,
                    distance: contact.dist,
                };
                // Prefer the most upward, closest contact; handle order breaks exact
                // ties so narrow-phase traversal cannot change moving-surface velocity.
                if selected.is_none_or(|old| {
                    candidate
                        .normal
                        .y
                        .total_cmp(&old.normal.y)
                        .then_with(|| old.distance.total_cmp(&candidate.distance))
                        .then_with(|| {
                            old.collider
                                .into_raw_parts()
                                .cmp(&candidate.collider.into_raw_parts())
                        })
                        .is_gt()
                }) {
                    selected = Some(candidate);
                }
            }
        }
    }
    selected
}
