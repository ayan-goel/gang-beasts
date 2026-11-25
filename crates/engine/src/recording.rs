use crate::{ConfigError, PhysicsConfig, fixture::validate_impulse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const MAX_RECORDING_BYTES: usize = 262_144;
pub const MAX_RECORDING_TICKS: u64 = 36_000;
pub const MAX_RECORDED_COMMANDS: usize = 1_024;
pub const ENGINE_HASH: &str = env!("BRAWLER_ENGINE_HASH");

#[derive(Debug, Error)]
pub enum RecordingError {
    #[error(transparent)]
    Command(#[from] crate::CommandError),
    #[error("invalid recording: {0}")]
    Invalid(&'static str),
    #[error("recording belongs to a different engine build")]
    BuildMismatch,
    #[error("recording limit reached; reset the fixture to start a new log")]
    LimitReached,
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error("invalid recording JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpulseRecord {
    pub tick: u32,
    pub impulse: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureRecording {
    pub schema_version: u32,
    pub engine_hash: String,
    pub producer_target: String,
    pub producer_profile: String,
    pub config_hash: String,
    pub config: PhysicsConfig,
    pub seed: u64,
    pub ticks: u32,
    pub commands: Vec<ImpulseRecord>,
}

fn config_hash(config: &PhysicsConfig) -> String {
    let json = serde_json::to_vec(config).expect("validated configuration is serializable");
    format!("{:x}", Sha256::digest(json))
}

impl FixtureRecording {
    pub(crate) fn new(config: PhysicsConfig) -> Self {
        Self {
            schema_version: 1,
            engine_hash: ENGINE_HASH.into(),
            producer_target: env!("BRAWLER_BUILD_TARGET").into(),
            producer_profile: env!("BRAWLER_BUILD_PROFILE").into(),
            config_hash: config_hash(&config),
            config,
            seed: 0,
            ticks: 0,
            commands: Vec::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, RecordingError> {
        // Bound parsing and later replay work before allocating a physics world.
        // Input-only replay is allowed only for the exact recorded engine sources.
        if json.len() > MAX_RECORDING_BYTES {
            return Err(RecordingError::Invalid("file exceeds 256 KiB"));
        }
        let recording: Self = serde_json::from_str(json)?;
        recording.validate()?;
        Ok(recording)
    }

    pub fn validate(&self) -> Result<(), RecordingError> {
        if self.schema_version != 1 || self.seed != 0 {
            return Err(RecordingError::Invalid(
                "unsupported schema or fixture seed",
            ));
        }
        if self.engine_hash != ENGINE_HASH {
            return Err(RecordingError::BuildMismatch);
        }
        self.config.validate()?;
        if self.config_hash != config_hash(&self.config) {
            return Err(RecordingError::Invalid("configuration hash mismatch"));
        }
        if self.producer_target.len() > 128
            || !["debug", "release"].contains(&self.producer_profile.as_str())
        {
            return Err(RecordingError::Invalid("invalid producer metadata"));
        }
        if u64::from(self.ticks) > MAX_RECORDING_TICKS
            || self.commands.len() > MAX_RECORDED_COMMANDS
        {
            return Err(RecordingError::LimitReached);
        }
        let mut previous_tick = 0;
        for command in &self.commands {
            if command.tick <= previous_tick || command.tick > self.ticks {
                return Err(RecordingError::Invalid(
                    "commands must have increasing completed ticks",
                ));
            }
            validate_impulse(command.impulse)
                .map_err(|_| RecordingError::Invalid("invalid impulse"))?;
            previous_tick = command.tick;
        }
        Ok(())
    }
}
