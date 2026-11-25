//! Shared fixed-tick physics and gameplay; independent of transport and presentation.

mod character;
mod combat;
mod config;
pub use combat::{Hand, HandPhase, HandState};
mod debug;
mod fixture;
mod physics;
mod recording;
mod session;
mod simulation;

pub use character::RecoveryConfig;
pub use character::{
    BalanceConfig, BalanceState, BodyDefinition, BodyPart, BodyShape, CharacterDefinition,
    CharacterError, JointDefinition, JointMotorConfig, JumpConfig, LocomotionConfig,
    LocomotionRagdoll, LocomotionState, MotorConfig, PassiveRagdoll, PoseRagdoll, RagdollBodyState,
    RagdollJointState,
};
pub use config::{ConfigError, PhysicsConfig};
pub use fixture::{CommandError, FixtureState};
pub use recording::{ENGINE_HASH, FixtureRecording, RecordingError};
pub use session::FixtureSession;
pub use simulation::Simulation;

pub const TICK_SECONDS: f32 = 1.0 / 60.0;
