use super::{
    BalanceConfig, BodyPart, CharacterError, rig_frame::ReadRig, step_probe, support::FootSupport,
};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LowStepConfig {
    pub hip_pitch_rad: f32,
    pub toe_pitch_rad: f32,
    pub forward_lean_gain: f32,
    pub hold_ticks: u8,
    pub release_ticks: u8,
}

impl Default for LowStepConfig {
    fn default() -> Self {
        Self {
            hip_pitch_rad: 0.4,
            toe_pitch_rad: 0.5,
            forward_lean_gain: 0.5,
            hold_ticks: 20,
            release_ticks: 4,
        }
    }
}

impl LowStepConfig {
    pub(super) fn validate(&self) -> Result<(), CharacterError> {
        if !(0.0..=0.7).contains(&self.hip_pitch_rad)
            || !(0.0..=0.5).contains(&self.toe_pitch_rad)
            || !(0.0..=1.0).contains(&self.forward_lean_gain)
            || self.hold_ticks > 40
            || !(1..=30).contains(&self.release_ticks)
        {
            return Err(CharacterError::Invalid("invalid low step pose or duration"));
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct StepApproach {
    hold: u8,
    blend: f32,
}

impl StepApproach {
    pub fn sample(
        &mut self,
        rig: ReadRig<'_>,
        intent: Vector<Real>,
        feet: [Option<FootSupport>; 2],
        support: &BalanceConfig,
        config: &LowStepConfig,
    ) {
        let Some(direction) = intent.try_normalize(0.001) else {
            *self = Self::default();
            return;
        };
        let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
        let forward = root.rotation() * -Vector::z();
        if forward.dot(&direction) < std::f32::consts::FRAC_1_SQRT_2 {
            *self = Self::default();
        } else if step_probe::approaching(rig, direction, feet, support) {
            self.hold = config.hold_ticks;
            self.blend = 1.;
        } else if self.hold > 0 {
            self.hold -= 1;
        } else {
            self.blend = (self.blend - 1. / f32::from(config.release_ticks)).max(0.);
        }
    }

    pub fn pose(
        &self,
        rig: ReadRig<'_>,
        intent: Vector<Real>,
        config: &LowStepConfig,
        targets: &mut [[f32; 3]; 14],
    ) {
        let weight = self.blend * intent.norm();
        if weight == 0. {
            return;
        }
        let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
        let lean = (root.rotation() * Vector::y()).dot(&intent);
        // As the pelvis pitches over the toes, release some hip flexion to let the planted leg extend.
        let hip = config.hip_pitch_rad - config.forward_lean_gain * lean;
        for (angles, joint) in targets.iter_mut().zip(&rig.parts.definition.joints) {
            let bias = match joint.child {
                BodyPart::LeftThigh | BodyPart::RightThigh => hip,
                BodyPart::LeftFoot | BodyPart::RightFoot => config.toe_pitch_rad,
                _ => continue,
            };
            angles[0] = (angles[0] + bias * weight)
                .clamp(joint.angular_limits[0][0], joint.angular_limits[0][1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::LocomotionConfig;

    #[test]
    fn existing_locomotion_files_use_the_bundled_step_pose_and_invalid_timing_is_rejected() {
        let mut json = serde_json::to_value(LocomotionConfig::bundled().unwrap()).unwrap();
        let defaults = json.as_object_mut().unwrap().remove("low_step").unwrap();
        let previous = LocomotionConfig::from_json(&json.to_string()).unwrap();
        assert_eq!(serde_json::to_value(previous.low_step).unwrap(), defaults);

        for (field, value) in [
            ("release_ticks", serde_json::json!(0)),
            ("hold_ticks", serde_json::json!(41)),
            ("hip_pitch_rad", serde_json::json!(-0.1)),
            ("toe_pitch_rad", serde_json::json!(1.)),
            ("forward_lean_gain", serde_json::json!(1.1)),
        ] {
            json["low_step"] = defaults.clone();
            json["low_step"][field] = value;
            assert!(
                LocomotionConfig::from_json(&json.to_string()).is_err(),
                "{field}"
            );
        }
    }
}
