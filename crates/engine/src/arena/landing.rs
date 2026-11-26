use super::Skyline;
use crate::physics::PhysicsWorld;
use rapier3d::{parry::query::intersection_test, prelude::*};

pub(crate) struct Landing {
    pub edge: Point<Real>,
    pub inward: Vector<Real>,
    pub body: RigidBodyHandle,
}

impl Skyline {
    pub(crate) fn landing(
        &self,
        world: &PhysicsWorld,
        object: u16,
        anchor: Point<Real>,
        next_tick: u64,
    ) -> Option<Landing> {
        let index = usize::from(object - 1);
        let body = self.bodies[index];
        let surface = &world.bodies[body];
        // Controllers execute before the hazard transition in the same tick.
        if !self.is_lip(object)
            || !(surface.is_fixed()
                || (surface.is_kinematic() && (surface.rotation() * Vector::y()).y > 0.72))
            || !surface.is_enabled()
            || (self.definition.tour.is_none()
                && self.panel_deadlines().contains(&(object, next_tick)))
        {
            return None;
        }
        let half = self.definition.objects[index].half;
        let axis = if half[0] - anchor.x.abs() <= half[2] - anchor.z.abs() {
            0
        } else {
            2
        };
        let mut edge = anchor;
        edge[axis] = anchor[axis].signum() * half[axis];
        edge.y = half[1];
        let mut inward = Vector::zeros();
        inward[axis] = -anchor[axis].signum();
        Some(Landing {
            edge: surface.position() * edge,
            inward: surface.rotation() * inward,
            body,
        })
    }
}

impl Landing {
    pub(crate) fn clear(
        &self,
        world: &PhysicsWorld,
        own: &[RigidBodyHandle],
        hip: Point<Real>,
    ) -> bool {
        let space = |depth: f32, low: f32, high: f32, half_depth: f32| {
            let center = self.edge + self.inward * depth + Vector::y() * ((high + low) * 0.5);
            let outward = self.inward.abs();
            let half = vector![
                outward.z * 0.34 + outward.x * half_depth,
                (high - low) * 0.5,
                outward.x * 0.34 + outward.z * half_depth
            ];
            (Isometry::from(center.coords), Cuboid::new(half))
        };
        // Cover the upright landing and the torso's approach outside the wall. An occupied
        // route withholds the assisted impulse while the existing hand joint supports the fighter.
        let clearance = [
            space(0.1, 0.02, 1.72, 0.72),
            space(-0.35, (hip.y - self.edge.y).min(-0.02), 0.02, 0.21),
        ];
        world.colliders.iter().all(|(_, c)| {
            !c.is_enabled()
                || c.is_sensor()
                || c.parent().is_some_and(|b| {
                    own.contains(&b) || b == self.body || !world.bodies[b].is_enabled()
                })
                || clearance.iter().all(|(pose, shape)| {
                    !intersection_test(pose, shape, c.position(), c.shape()).unwrap_or(true)
                })
        })
    }
}
