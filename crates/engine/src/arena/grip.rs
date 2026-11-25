use super::{ArenaKind, Skyline};
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

impl Skyline {
    pub(crate) fn grip_surfaces<'a>(
        &'a self,
        world: &'a PhysicsWorld,
    ) -> impl Iterator<Item = (u16, RigidBodyHandle, ColliderHandle)> + 'a {
        self.definition
            .objects
            .iter()
            .zip(&self.bodies)
            .filter_map(|(obj, &body)| {
                let target = &world.bodies[body];
                // Only the authored playable box has handholds. The secondary façade collider is solid scenery.
                target
                    .is_enabled()
                    .then(|| (obj.id, body, target.colliders()[0]))
            })
    }

    pub(crate) fn allows_grip(
        &self,
        world: &PhysicsWorld,
        id: u16,
        surface: Point<Real>,
        palm: Real,
    ) -> bool {
        let idx = usize::from(id - 1);
        let obj = &self.definition.objects[idx];
        if matches!(obj.kind, ArenaKind::Vent | ArenaKind::Crate) {
            return true;
        }
        let body = &world.bodies[self.bodies[idx]];
        let local = body.position().inverse_transform_point(&surface);
        if local.y < obj.half[1] - palm {
            return false;
        }
        // Probe just outside each reachable lip. An adjoining walkway closes that edge until it moves away.
        [0, 2].into_iter().any(|axis| {
            if obj.half[axis] - local[axis].abs() > palm {
                return false;
            }
            let mut outside = local;
            outside[axis] = local[axis].signum() * (obj.half[axis] + 0.002);
            outside.y = obj.half[1] - 0.002;
            let probe = body.position().transform_point(&outside);
            !self.grip_surfaces(world).any(|(other, _, c)| {
                let collider = &world.colliders[c];
                other != id && collider.shape().contains_point(collider.position(), &probe)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_roof_and_walkway_only_offer_exposed_upper_edges() {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let mut arena = Skyline::insert(&mut world).unwrap();
        let palm = 0.07;
        for (id, point, expected) in [
            (1, point![0., 0., 0.], false),
            (1, point![0., -0.3, 10.], false),
            (1, point![0., -0.05, 10.], true),
            (1, point![9.98, 0., 8.], true),
            (1, point![10., 0., 2.], false),
            (9, point![10., 0., 2.], false),
            (9, point![12., -0.03, 2.], true),
        ] {
            assert_eq!(
                arena.allows_grip(&world, id, point, palm),
                expected,
                "object {id}, point {point:?}"
            );
        }
        arena.step(&mut world, 9000);
        for _ in 0..30 {
            world.step();
        }
        assert!(arena.allows_grip(&world, 1, point![10., 0., 2.], palm));
    }
}
