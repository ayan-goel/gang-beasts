use super::CharacterError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryConfig {
    pub force_cap_n: f32,
    pub moment_cap_nm: f32,
    pub response_s: f32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            force_cap_n: 600.0,
            moment_cap_nm: 360.0,
            response_s: 0.25,
        }
    }
}

impl RecoveryConfig {
    pub(super) fn validate(&self) -> Result<(), CharacterError> {
        // These limits bound the extra contact-assisted actuator independently of gait and joint motors.
        if !(0.0..=600.0).contains(&self.force_cap_n)
            || !(0.0..=360.0).contains(&self.moment_cap_nm)
            || !(0.2..=1.0).contains(&self.response_s)
        {
            return Err(CharacterError::Invalid(
                "invalid recovery force, moment or response",
            ));
        }
        Ok(())
    }
}
