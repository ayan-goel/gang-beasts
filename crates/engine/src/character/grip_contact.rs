use super::{BodyPart, FighterId, duel_fighter::Fighter};
use crate::physics::PhysicsWorld;
use rapier3d::{parry::query::contact, prelude::*};

#[derive(Clone, Copy)]
pub(super) enum Target {
    Fighter(FighterId, BodyPart),
    Arena(u16),
}

pub(super) struct Candidate {
    pub body: RigidBodyHandle,
    pub target: Target,
    pub point: Point<Real>,
    distance: Real,
}

pub(super) fn nearest(
    world: &PhysicsWorld,
    hand: ColliderHandle,
    id: FighterId,
    opponent: &Fighter,
    arena: Option<&crate::arena::Skyline>,
    reach: f32,
) -> Option<Candidate> {
    let fist = &world.colliders[hand];
    let mut best: Option<Candidate> = None;
    let mut consider = |body, handle, target| {
        let collider: &Collider = &world.colliders[handle];
        if !collider.is_enabled() || collider.is_sensor() {
            return;
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
            return;
        };
        if best.as_ref().is_some_and(|b| b.distance <= c.dist)
            || occluded(world, [hand, handle], c.point1, c.point2, c.dist)
        {
            return;
        }
        if let Target::Arena(id) = target {
            let palm = fist.shape().compute_local_aabb().half_extents().min();
            if !arena
                .expect("arena surface")
                .allows_grip(world, id, c.point2, palm)
            {
                return;
            }
        }
        best = Some(Candidate {
            body,
            target,
            distance: c.dist,
            point: nalgebra::center(&c.point1, &c.point2),
        });
    };
    // Stable iteration resolves equal surface distances in favor of a fighter, then the lowest map ID.
    for region in BodyPart::ALL {
        let body = opponent.bodies[region.index()];
        if world.bodies[body].is_enabled() {
            for &handle in world.bodies[body].colliders() {
                consider(body, handle, Target::Fighter(id, region));
            }
        }
    }
    if let Some(map) = arena {
        for (id, body, handle) in map.grip_surfaces(world) {
            consider(body, handle, Target::Arena(id));
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
