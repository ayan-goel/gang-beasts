use super::Skyline;
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

impl Skyline {
    pub(crate) fn reachable_lip(
        &self,
        world: &PhysicsWorld,
        shoulder: Point<Real>,
        palm: Point<Real>,
        span: Real,
        radius: Real,
    ) -> Option<(Point<Real>, Vector<Real>)> {
        let mut best = None;
        let mut distance = Real::INFINITY;
        for (id, body, _) in self.grip_surfaces(world) {
            let obj = &self.definition.objects[usize::from(id - 1)];
            if !self.is_lip(id) {
                continue;
            }
            let target = &world.bodies[body];
            let a = target.position().inverse_transform_point(&shoulder);
            if (a.y - obj.half[1]).abs() > span {
                continue;
            }
            let hand = target.position().inverse_transform_point(&palm);
            for axis in [0, 2] {
                if a[axis].abs() < obj.half[axis] - radius {
                    continue;
                }
                let along = 2 - axis;
                let mut edge = hand;
                edge[axis] = a[axis].signum() * obj.half[axis];
                edge[along] = edge[along].clamp(-obj.half[along], obj.half[along]);
                edge.y = obj.half[1];
                let lip = target.position() * edge;
                if (lip - shoulder).norm() > span + radius
                    || !self.allows_grip(world, id, lip, radius)
                {
                    continue;
                }
                let gap = (lip - palm).norm_squared();
                if gap < distance {
                    distance = gap;
                    best = Some((lip, target.velocity_at_point(&lip)));
                }
            }
        }
        best
    }
}
