use super::CharacterError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BalanceConfig {
    pub schema_version: u32,
    pub stiffness_nm_per_rad: f32,
    pub damping_nm_s_per_rad: f32,
    pub max_torque_nm: f32,
    pub minimum_normal_y: f32,
    pub maximum_contact_distance_m: f32,
    pub maximum_separation_speed_m_s: f32,
}

impl BalanceConfig {
    pub fn bundled() -> Result<Self, CharacterError> {
        Self::from_json(include_str!("../../../../config/balance.json"))
    }

    pub fn from_json(json: &str) -> Result<Self, CharacterError> {
        // Bound imported tuning before allocation; no controller state is replaced on failure.
        if json.len() > 4096 {
            return Err(CharacterError::Invalid(
                "balance configuration exceeds 4 KiB",
            ));
        }
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), CharacterError> {
        if self.schema_version != 1 {
            return Err(CharacterError::Invalid("unsupported balance schema"));
        }
        // Effort ceilings prevent tuning from turning balance into an unlimited anchor.
        // Contact bounds also prevent walls and distant surfaces from counting as support.
        for (value, min, max) in [
            (self.stiffness_nm_per_rad, 0.0, 1000.0),
            (self.damping_nm_s_per_rad, 0.0, 20.0),
            (self.max_torque_nm, 0.0, 50.0),
            (self.minimum_normal_y, 0.5, 1.0),
            (self.maximum_contact_distance_m, 0.0, 0.03),
            (self.maximum_separation_speed_m_s, 0.0, 1.0),
        ] {
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(CharacterError::Invalid(
                    "balance tuning outside allowed range",
                ));
            }
        }
        Ok(())
    }
}
