use super::{BodyPart, duel_fighter::Fighter};
use crate::physics::PhysicsWorld;
use rapier3d::{parry::query::contact, prelude::*};

pub(super) struct Candidate {
    pub body: RigidBodyHandle,
    pub region: BodyPart,
    pub point: Point<Real>,
    distance: Real,
}

pub(super) fn nearest(
    world: &PhysicsWorld,
    hand: ColliderHandle,
    opponent: &Fighter,
    reach: f32,
) -> Option<Candidate> {
    let fist = &world.colliders[hand];
    let mut best: Option<Candidate> = None;
    for region in BodyPart::ALL {
        let body = opponent.bodies[region.index()];
        let target = &world.bodies[body];
        if !target.is_enabled() {
            continue;
        }
        for &handle in target.colliders() {
            let collider = &world.colliders[handle];
            if collider.is_sensor() || !collider.is_enabled() {
                continue;
            }
            let Some(c) = contact(
                fist.position(),
                fist.shape(),
                collider.position(),
                collider.shape(),
                reach,
            )
            .ok()
            .flatten() else {
                continue;
            };
            if best.as_ref().is_some_and(|b| b.distance <= c.dist) {
                continue;
            }
            if occluded(world, [hand, handle], c.point1, c.point2, c.dist) {
                continue;
            }
            best = Some(Candidate {
                body,
                region,
                point: nalgebra::center(&c.point1, &c.point2),
                distance: c.dist,
            });
        }
    }
    best
}

fn occluded(
    world: &PhysicsWorld,
    ends: [ColliderHandle; 2],
    start: Point<Real>,
    end: Point<Real>,
    gap: f32,
) -> bool {
    if gap <= 0.0 {
        return false;
    }
    // Only the short segment between the two actual surfaces may bridge a gap.
    let ray = Ray::new(start, end - start);
    world.colliders.iter().any(|(id, c)| {
        !ends.contains(&id)
            && c.is_enabled()
            && !c.is_sensor()
            && c.shape().cast_ray(c.position(), &ray, 1.0, true).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervening_solid_blocks_the_short_surface_gap_but_sensor_does_not() {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let start = point![0., 0., 0.];
        let end = point![0.024, 0., 0.];
        let ends = [start, end].map(|point| {
            world
                .colliders
                .insert(ColliderBuilder::ball(0.001).translation(point.coords))
        });
        let barrier = world
            .colliders
            .insert(ColliderBuilder::cuboid(0.0005, 1., 1.).translation(vector![0.012, 0., 0.]));
        assert!(occluded(&world, ends, start, end, 0.024));
        world.colliders[barrier].set_sensor(true);
        assert!(!occluded(&world, ends, start, end, 0.024));
        world.colliders[barrier].set_sensor(false);
        world.colliders[barrier].set_enabled(false);
        assert!(!occluded(&world, ends, start, end, 0.024));
    }
}
