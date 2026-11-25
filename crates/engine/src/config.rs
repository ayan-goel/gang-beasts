use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicsConfig {
    pub schema_version: u32,
    pub gravity_y: f32,
    pub solver_iterations: usize,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid physics configuration: {0}")]
    Invalid(&'static str),
    #[error("invalid configuration JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            gravity_y: -9.81,
            solver_iterations: 8,
        }
    }
}

impl PhysicsConfig {
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != 1 {
            return Err(ConfigError::Invalid("unsupported schema version"));
        }
        // Bound configuration before allocating a world so malformed tuning cannot
        // introduce nonfinite state or unbounded solver work.
        if !self.gravity_y.is_finite() || !(-30.0..=-0.1).contains(&self.gravity_y) {
            return Err(ConfigError::Invalid(
                "gravity_y must be between -30 and -0.1 m/s²",
            ));
        }
        if !(1..=32).contains(&self.solver_iterations) {
            return Err(ConfigError::Invalid(
                "solver_iterations must be between 1 and 32",
            ));
        }
        Ok(())
    }
}
