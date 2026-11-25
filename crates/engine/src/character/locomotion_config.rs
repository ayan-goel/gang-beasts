use super::CharacterError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocomotionConfig {
    pub schema_version: u32,
    pub walk_speed_m_s: f32,
    pub run_speed_m_s: f32,
    pub maximum_acceleration_m_s2: f32,
    pub velocity_response_s: f32,
    pub walk_frequency_hz: f32,
    pub run_frequency_hz: f32,
    pub hip_amplitude_rad: f32,
    pub knee_amplitude_rad: f32,
    pub turn_stiffness_nm_rad: f32,
    pub turn_damping_nm_s_rad: f32,
    pub turn_cap_nm: f32,
    pub stopping_moment_cap_nm: f32,
    pub braking_ticks: u32,
    pub motor_release_s: f32,
    pub settle_speed_m_s: f32,
    pub settle_spin_rad_s: f32,
    pub settle_minimum_up_y: f32,
    pub gait_blend_s: f32,
    pub minimum_upright_y: f32,
    pub idle_response_s: f32,
    pub idle_maximum_acceleration_m_s2: f32,
    pub drive_tilt_gain: f32,
    pub drive_tilt_damping_s: f32,
}

impl LocomotionConfig {
    pub fn bundled() -> Result<Self, CharacterError> {
        Self::from_json(include_str!("../../../../config/locomotion.json"))
    }
    pub fn from_json(json: &str) -> Result<Self, CharacterError> {
        if json.len() > 4096 {
            return Err(CharacterError::Invalid(
                "locomotion configuration exceeds 4 KiB",
            ));
        }
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }
    pub fn validate(&self) -> Result<(), CharacterError> {
        if self.schema_version != 2
            || self.run_speed_m_s < self.walk_speed_m_s
            || self.braking_ticks > 120
        {
            return Err(CharacterError::Invalid(
                "invalid locomotion schema or speed ordering",
            ));
        }
        // Finite speed/effort ceilings keep movement from becoming an unlimited
        // tether; positive response times prevent division by zero. Settling bounds
        // exclude strict-comparison endpoints that could never release active motors.
        for (value, min, max) in [
            (self.walk_speed_m_s, 0.0, 6.0),
            (self.run_speed_m_s, 0.0, 8.0),
            (self.maximum_acceleration_m_s2, 0.0, 20.0),
            (self.velocity_response_s, 0.02, 1.0),
            (self.walk_frequency_hz, 0.0, 4.0),
            (self.run_frequency_hz, 0.0, 5.0),
            (self.hip_amplitude_rad, 0.0, 0.7),
            (self.knee_amplitude_rad, 0.0, 1.2),
            (self.turn_stiffness_nm_rad, 0.0, 100.0),
            (self.turn_damping_nm_s_rad, 0.0, 20.0),
            (self.turn_cap_nm, 0.0, 30.0),
            (self.stopping_moment_cap_nm, 0.0, 120.0),
            (self.drive_tilt_gain, 0.0, 4.0),
            (self.motor_release_s, 0.1, 2.0),
            (self.settle_speed_m_s, 0.01, 0.5),
            (self.settle_spin_rad_s, 0.01, 1.0),
            (self.settle_minimum_up_y, 0.9, 0.999),
            (self.gait_blend_s, 0.05, 1.0),
            (self.minimum_upright_y, 0.25, 0.9),
            (self.idle_response_s, 0.25, 2.0),
            (self.idle_maximum_acceleration_m_s2, 0.0, 2.0),
            (self.drive_tilt_damping_s, 0.0, 1.0),
        ] {
            if !value.is_finite() || !(min..=max).contains(&value) {
                return Err(CharacterError::Invalid(
                    "locomotion tuning outside allowed range",
                ));
            }
        }
        Ok(())
    }
}
