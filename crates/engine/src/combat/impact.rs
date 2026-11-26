use crate::{BodyPart, CharacterError};
use rapier3d::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactTuning {
    schema_version: u8,
    minimum_closing_speed_m_s: f32,
    threshold_j: f32,
    points_per_joule: f32,
    maximum_damage: f32,
}

impl ImpactTuning {
    pub fn bundled() -> Result<Self, CharacterError> {
        let config: Self = serde_json::from_str(include_str!("../../../../config/impact.json"))?;
        if config.schema_version != 1
            || !(0.01..=20.).contains(&config.minimum_closing_speed_m_s)
            || !(0.0..=100.).contains(&config.threshold_j)
            || !(0.01..=100.).contains(&config.points_per_joule)
            || !(0.01..=100.).contains(&config.maximum_damage)
        {
            return Err(CharacterError::Invalid(
                "impact tuning outside supported range",
            ));
        }
        Ok(config)
    }
    pub fn evaluate(
        &self,
        closing: f32,
        masses: [f32; 2],
        region: BodyPart,
        scale: f32,
    ) -> Option<(f32, f32)> {
        if closing < self.minimum_closing_speed_m_s {
            return None;
        }
        let [a, b] = masses;
        let energy = 0.5 * (a * b / (a + b)) * closing.powi(2);
        let multiplier = match region {
            BodyPart::Head => 1.5,
            BodyPart::Chest => 1.0,
            BodyPart::Pelvis => 0.8,
            BodyPart::LeftUpperArm
            | BodyPart::RightUpperArm
            | BodyPart::LeftForearm
            | BodyPart::RightForearm
            | BodyPart::LeftHand
            | BodyPart::RightHand => 0.4,
            _ => 0.5,
        };
        let damage = ((energy - self.threshold_j) * self.points_per_joule * multiplier * scale)
            .min(self.maximum_damage);
        (damage > 0.0).then_some((energy, damage))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BodyMotion {
    pose: Isometry<Real>,
    center: Point<Real>,
    linear: Vector<Real>,
    angular: Vector<Real>,
    pub mass: f32,
}
impl BodyMotion {
    pub fn sample(body: &RigidBody) -> Self {
        Self {
            pose: *body.position(),
            center: *body.center_of_mass(),
            linear: *body.linvel(),
            angular: *body.angvel(),
            mass: body.mass(),
        }
    }
    pub fn forward(&self) -> Vector<Real> {
        self.pose.rotation * -Vector::z()
    }
    pub fn point_velocity(&self, local: Point<Real>) -> Vector<Real> {
        self.linear + self.angular.cross(&(self.pose * local - self.center))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn angular_point_motion_normal_speed_region_weight_and_cap() {
        let motion = BodyMotion {
            pose: Isometry::translation(1., 0., 0.),
            center: point![1., 0., 0.],
            linear: vector![0., 4., 0.],
            angular: vector![0., 2., 0.],
            mass: 1.,
        };
        let velocity = motion.point_velocity(point![0.5, 0., 0.]);
        let closing = velocity.dot(&-Vector::z());
        assert_eq!(closing, 1.);
        let tuning = ImpactTuning::bundled().unwrap();
        let (energy, damage) = tuning
            .evaluate(closing, [1., 3.], BodyPart::Chest, 1.)
            .unwrap();
        assert_eq!(energy, 0.375);
        assert!((damage - 0.875).abs() < 1e-5);
        assert_eq!(
            tuning
                .evaluate(closing, [1., 3.], BodyPart::Head, 1.)
                .unwrap()
                .1,
            1.5 * damage
        );
        assert!(
            tuning
                .evaluate(-closing, [1., 3.], BodyPart::Chest, 1.)
                .is_none()
        );
        assert!(
            tuning
                .evaluate(0.5, [1., 3.], BodyPart::Chest, 1.)
                .is_none()
        );
        assert_eq!(
            tuning
                .evaluate(100., [1., 3.], BodyPart::Head, 1.)
                .unwrap()
                .1,
            35.
        );
    }
}
