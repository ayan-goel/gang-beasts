use super::{BodyPart, LocomotionConfig, PassiveRagdoll, support::FootSupport};
use rapier3d::na::{ComplexField, RealField};
use rapier3d::prelude::*;

#[derive(Default)]
pub(super) struct SwingCompletion {
    pending: Option<(usize, f32)>,
}

impl SwingCompletion {
    pub fn arm(&mut self, phase: f32) {
        let phase = phase.rem_euclid(std::f32::consts::TAU);
        let side = usize::from(phase < std::f32::consts::PI);
        self.pending = Some((
            side,
            std::f32::consts::PI - phase.rem_euclid(std::f32::consts::PI),
        ));
    }

    pub fn finish(&mut self) {
        self.pending = None;
    }

    pub fn advance(
        &mut self,
        rig: &PassiveRagdoll,
        supports: [Option<FootSupport>; 2],
        relative_velocity: Vector<Real>,
        config: &LocomotionConfig,
        phase_delta: f32,
        targets: &mut [[f32; 3]; 14],
    ) {
        let Some((side, remaining)) = self.pending else {
            return;
        };
        let Some(support) = supports[1 - side] else {
            self.finish();
            return;
        };
        if supports[side].is_some() {
            self.finish();
            return;
        }
        // The release event can finish only its existing swing. A later impact
        // cannot rearm it. Allow accumulated f32 phase error at the terminal sample.
        self.pending = if remaining <= phase_delta + 64.0 * f32::EPSILON * std::f32::consts::TAU {
            None
        } else {
            Some((side, remaining - phase_delta))
        };

        let definition = &rig.definition;
        let root = &rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]];
        let forward = root.rotation() * -Vector::z();
        let heading = Rotation::from_axis_angle(
            &Vector::y_axis(),
            <f32 as RealField>::atan2(-forward.x, -forward.z),
        );
        let mass = definition.bodies.iter().map(|b| b.mass).sum::<f32>();
        let center = rig
            .bodies
            .iter()
            .map(|h| {
                let b = &rig.world.bodies[*h];
                b.center_of_mass().coords * b.mass()
            })
            .sum::<Vector<Real>>()
            / mass;
        let bind_center = definition
            .bodies
            .iter()
            .map(|b| Vector::from(b.position) * b.mass)
            .sum::<Vector<Real>>()
            / mass;
        let feet = [BodyPart::LeftFoot, BodyPart::RightFoot];
        let stance = &rig.world.bodies[rig.bodies[feet[1 - side].index()]];
        let nominal = heading
            * (bind_center - Vector::from(definition.bodies[feet[1 - side].index()].position));
        let gravity = rig.configuration().gravity_y.abs();
        if gravity <= f32::EPSILON {
            return;
        }
        let height = (center.y - support.contact_point.y).max(0.0);
        let horizon = <f32 as ComplexField>::sqrt(height / gravity);
        let mut error =
            center - stance.center_of_mass().coords - nominal + relative_velocity * horizon;
        error.y = 0.0;
        let Some(direction) = error.try_normalize(0.00001) else {
            return;
        };
        let sole = &rig.world.colliders[stance.colliders()[0]];
        let local_direction = stance.rotation().inverse() * direction;
        let footprint = sole
            .shape()
            .compute_local_aabb()
            .half_extents()
            .dot(&local_direction.abs());
        let excess = (error.norm() - footprint).max(0.0);
        if excess == 0.0 {
            return;
        }

        let parts = if side == 0 {
            [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot]
        } else {
            [
                BodyPart::RightThigh,
                BodyPart::RightShin,
                BodyPart::RightFoot,
            ]
        };
        let [hip, knee, ankle] = parts.map(|part| {
            definition
                .joints
                .iter()
                .position(|j| j.child == part)
                .unwrap()
        });
        let reach = (Vector::from(definition.joints[hip].anchor)
            - Vector::from(definition.joints[knee].anchor))
        .norm()
            + (Vector::from(definition.joints[knee].anchor)
                - Vector::from(definition.joints[ankle].anchor))
            .norm();
        let angle = <f32 as RealField>::atan2(excess, reach).min(config.hip_amplitude_rad);
        let axis =
            UnitVector::new_normalize((-Vector::y()).cross(&(heading.inverse() * direction)));
        let q = |v: [f32; 3]| Rotation::from_euler_angles(v[0], v[1], v[2]);
        let (h, k, a) = (q(targets[hip]), q(targets[knee]), q(targets[ankle]));
        let clamp = |index: usize, rotation: Rotation<Real>| {
            let (x, y, z) = rotation.euler_angles();
            core::array::from_fn(|i| {
                [x, y, z][i].clamp(
                    definition.joints[index].angular_limits[i][0],
                    definition.joints[index].angular_limits[i][1],
                )
            })
        };
        let original = *targets;
        // Bounded backtracking prevents an inward swing from intersecting the
        // other sole. If no candidate is clear, retain the existing gait targets.
        for scale in [1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125, 0.015625, 0.0078125] {
            let mut candidate = original;
            candidate[hip] = clamp(hip, Rotation::from_axis_angle(&axis, angle * scale) * h);
            candidate[ankle] = clamp(ankle, (q(candidate[hip]) * k).inverse() * h * k * a);
            if super::foot_target::separated(rig, &candidate) {
                *targets = candidate;
                return;
            }
        }
    }
}
