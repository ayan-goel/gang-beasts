use super::{BodyPart, CharacterError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointMotorConfig {
    pub child: BodyPart,
    pub stiffness_nm_per_rad: f32,
    pub damping_nm_s_per_rad: f32,
    pub max_torque_nm: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MotorConfig {
    pub schema_version: u32,
    pub model: String,
    pub joints: [JointMotorConfig; 14],
}

impl MotorConfig {
    pub fn bundled() -> Result<Self, CharacterError> {
        Self::from_json(include_str!("../../../../config/motors.json"))
    }

    pub fn standing() -> Result<Self, CharacterError> {
        Self::from_json(include_str!("../../../../config/standing-motors.json"))
    }

    pub fn from_json(json: &str) -> Result<Self, CharacterError> {
        // Limit allocation before deserialization; tuning cannot create more actuators.
        if json.len() > 8192 {
            return Err(CharacterError::Invalid("motor configuration exceeds 8 KiB"));
        }
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), CharacterError> {
        if self.schema_version != 1
            || !matches!(self.model.as_str(), "explicit_pd" | "implicit_velocity_pd")
        {
            return Err(CharacterError::Invalid("unsupported motor schema or model"));
        }
        let mut seen = [false; 15];
        for motor in &self.joints {
            if motor.child.parent().is_none() || seen[motor.child.index()] {
                return Err(CharacterError::Invalid("invalid or duplicate motor child"));
            }
            seen[motor.child.index()] = true;
            // These are effort/input ceilings, not a stability guarantee for arbitrary gains.
            // Removing them would let imported tuning bypass the bounded-actuation contract.
            for (value, max) in [
                (motor.stiffness_nm_per_rad, 200.0),
                (motor.damping_nm_s_per_rad, 20.0),
                (motor.max_torque_nm, 50.0),
            ] {
                if !value.is_finite() || !(0.0..=max).contains(&value) {
                    return Err(CharacterError::Invalid("motor gain outside allowed range"));
                }
            }
            if matches!(motor.child, BodyPart::LeftHand | BodyPart::RightHand)
                && (motor.stiffness_nm_per_rad != 0.0
                    || motor.damping_nm_s_per_rad != 0.0
                    || motor.max_torque_nm != 0.0)
            {
                return Err(CharacterError::Invalid("fixed wrists cannot be actuated"));
            }
        }
        Ok(())
    }
}
