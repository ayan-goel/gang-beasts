use super::{BodyPart, PassiveRagdoll, support::FootSupport};
use rapier3d::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reference {
    World,
    Body(RigidBodyHandle),
}

#[derive(Default)]
pub(super) struct PoseMotion {
    previous: Option<(u64, Rotation<Real>)>,
    reference: Option<Reference>,
}

impl PoseMotion {
    pub fn sample_supported(
        &mut self,
        rig: &PassiveRagdoll,
        supports: [Option<FootSupport>; 2],
    ) -> Vector<Real> {
        let root = &rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]];
        let mut selected = supports.iter().flatten();
        let Some(first) = selected.next() else {
            *self = Self::default();
            return *root.angvel();
        };
        // Fixed parents share the inertial world frame, including across tile seams.
        // Treating each fixed handle as moving would discard valid completed motion.
        let moving_body = |support: &FootSupport| {
            support
                .body
                .filter(|body| !rig.world.bodies[*body].is_fixed())
        };
        let reference_body = moving_body(first);
        if selected.any(|support| moving_body(support) != reference_body) {
            // Different moving bodies have no single material frame. Discard
            // history and use current relative spin until a shared frame exists.
            *self = Self::default();
            let spin = supports
                .iter()
                .flatten()
                .map(|s| s.angular_velocity)
                .sum::<Vector<Real>>()
                / supports.iter().flatten().count() as f32;
            return root.angvel() - spin;
        }
        let frame = reference_body.map_or(Rotation::identity(), |body| {
            *rig.world.bodies[body].rotation()
        });
        self.sample_in_frame(
            rig.tick(),
            *root.rotation(),
            *root.angvel(),
            reference_body,
            frame,
            first.angular_velocity,
        )
    }

    fn sample_in_frame(
        &mut self,
        tick: u64,
        rotation: Rotation<Real>,
        instantaneous: Vector<Real>,
        body: Option<RigidBodyHandle>,
        frame: Rotation<Real>,
        spin: Vector<Real>,
    ) -> Vector<Real> {
        let reference = body.map_or(Reference::World, Reference::Body);
        if self.reference != Some(reference) {
            self.previous = None;
        }
        self.reference = Some(reference);
        // Difference the pelvis in the same material frame at both completed
        // ticks. Subtracting current deck spin from an older interval is not relative motion.
        frame
            * self.sample(
                tick,
                frame.inverse() * rotation,
                frame.inverse() * (instantaneous - spin),
            )
    }

    fn sample(
        &mut self,
        tick: u64,
        rotation: Rotation<Real>,
        instantaneous: Vector<Real>,
    ) -> Vector<Real> {
        let previous = self.previous.replace((tick, rotation));
        match previous {
            Some((last_tick, last_rotation)) if last_tick.checked_add(1) == Some(tick) => {
                // A solver endpoint velocity can oscillate without comparable pose
                // motion. Measure the completed tick for the settling decision.
                super::motor::rotation_error(last_rotation, rotation) / crate::TICK_SECONDS
            }
            // Never interpret missing or repeated physics samples as zero motion.
            _ => instantaneous,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TICK_SECONDS;

    #[test]
    fn completed_rotation_is_measured_in_world_axes_instead_of_endpoint_velocity() {
        let mut motion = PoseMotion::default();
        let start = Rotation::from_euler_angles(0.3, 0.2, -0.4);
        let endpoint_velocity = Vector::x() * 2.0;
        assert_eq!(
            motion.sample(7, start, endpoint_velocity),
            endpoint_velocity
        );
        let end = Rotation::from_axis_angle(&Vector::y_axis(), 0.4 * TICK_SECONDS) * start;
        let equivalent = Rotation::new_unchecked(-end.into_inner());
        assert!(
            (motion.sample(8, equivalent, endpoint_velocity) - Vector::y() * 0.4).norm() < 0.00001
        );
        assert_eq!(motion.sample(8, end, endpoint_velocity), endpoint_velocity);
        assert_eq!(motion.sample(10, end, endpoint_velocity), endpoint_velocity);
        let next = Rotation::from_axis_angle(&Vector::x_axis(), -0.1 * TICK_SECONDS) * end;
        assert!((motion.sample(11, next, endpoint_velocity) + Vector::x() * 0.1).norm() < 0.00001);
    }

    #[test]
    fn stationary_pose_does_not_inherit_solver_endpoint_spin() {
        let mut motion = PoseMotion::default();
        motion.sample(0, Rotation::identity(), Vector::x() * 0.4);
        assert_eq!(
            motion.sample(1, Rotation::identity(), Vector::x() * 0.4),
            Vector::zeros()
        );
    }
}

#[cfg(test)]
#[path = "relative_pose_motion_tests.rs"]
mod relative_tests;
