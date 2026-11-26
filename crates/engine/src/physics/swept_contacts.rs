use super::*;
use rapier3d::parry::query::{ShapeCastOptions, cast_shapes};

impl PhysicsWorld {
    pub fn track_fast_body(&mut self, body: RigidBodyHandle, half_extent: Real) {
        self.fast_bodies.push((body, half_extent));
    }

    pub(super) fn prepare_sweeps(&mut self) {
        for &(handle, half_extent) in &self.fast_bodies {
            let body = &self.bodies[handle];
            let travel = body.linvel().norm() * self.parameters.dt;
            let predict = body.is_enabled() && travel >= half_extent && self.crosses_solid(handle);
            self.bodies[handle].set_soft_ccd_prediction(if predict { travel } else { 0. });
        }
    }

    fn crosses_solid(&self, handle: RigidBodyHandle) -> bool {
        let body = &self.bodies[handle];
        let shape = self.colliders[body.colliders()[0]].shape();
        let pose = body.position();
        let bounds = shape.compute_aabb(pose);
        self.colliders.iter().any(|(_, other)| {
            if !other.is_enabled() || other.is_sensor() || other.parent() == Some(handle) {
                return false;
            }
            let (other_pose, other_vel) = match other.parent() {
                Some(id) => {
                    let target = &self.bodies[id];
                    if !target.is_enabled() {
                        return false;
                    }
                    (
                        target.position() * other.position_wrt_parent().unwrap(),
                        *target.linvel(),
                    )
                }
                None => (*other.position(), Vector::zeros()),
            };
            let relative = (body.linvel() - other_vel) * self.parameters.dt;
            if !bounds.intersects_moving_aabb(&other.shape().compute_aabb(&other_pose), -relative) {
                return false;
            }
            if other.parent().is_some_and(|id| {
                self.impulse_joints
                    .joints_between(handle, id)
                    .any(|(_, joint)| !joint.data.contacts_enabled)
            }) {
                return false;
            }
            let options = ShapeCastOptions {
                stop_at_penetration: false,
                ..ShapeCastOptions::with_max_time_of_impact(1.)
            };
            let Some(hit) = cast_shapes(
                pose,
                &relative,
                shape,
                &other_pose,
                &Vector::zeros(),
                other.shape(),
                options,
            )
            .ok()
            .flatten() else {
                return false;
            };
            // Only a crossing to the far side needs an earlier constraint. Ordinary contact
            // on the near side stays in the regular solve, including contact on a supporting floor.
            let normal = other_pose.rotation * hit.normal2;
            let end = pose.translation.vector - other_pose.translation.vector + relative;
            relative.dot(&normal) < 0. && end.dot(&normal) < 0.
        })
    }
}
