use super::BalanceConfig;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "support_tests.rs"]
mod tests;

fn current_pose(world: &PhysicsWorld, collider: &Collider) -> Isometry<Real> {
    match collider.parent() {
        Some(body) => world.bodies[body].position() * collider.position_wrt_parent().unwrap(),
        None => *collider.position(),
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FootSupport {
    pub velocity: Vector<Real>,
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
    let mut selected: Option<FootSupport> = None;
    for handle in world.bodies[foot].colliders() {
        let foot_collider = &world.colliders[*handle];
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
            let other = &world.colliders[other_handle];
            if other.is_sensor()
                || other
                    .parent()
                    .is_some_and(|body| own_bodies.contains(&body))
            {
                continue;
            }
            // Cached normals alone survive a sideways departure from a finite ledge.
            // Re-query current shapes; unsupported shape queries conservatively give no assistance.
            let Ok(Some(contact)) = rapier3d::parry::query::contact(
                &current_pose(world, foot_collider),
                foot_collider.shape(),
                &current_pose(world, other),
                other.shape(),
                config.maximum_contact_distance_m,
            ) else {
                continue;
            };
            if contact.normal2.y < config.minimum_normal_y {
                continue;
            }
            let surface_velocity = other.parent().map_or(Vector::zeros(), |body| {
                world.bodies[body].velocity_at_point(&contact.point2)
            });
            let relative_velocity =
                world.bodies[foot].velocity_at_point(&contact.point1) - surface_velocity;
            if relative_velocity.dot(&contact.normal2) <= config.maximum_separation_speed_m_s {
                let candidate = FootSupport {
                    friction: pair
                        .manifolds
                        .iter()
                        .filter(|m| m.data.solver_flags.contains(SolverFlags::COMPUTE_IMPULSES))
                        .flat_map(|m| &m.data.solver_contacts)
                        .map(|contact| contact.friction)
                        .min_by(f32::total_cmp)
                        .unwrap_or(0.0),
                    velocity: surface_velocity,
                    normal: *contact.normal2,
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
