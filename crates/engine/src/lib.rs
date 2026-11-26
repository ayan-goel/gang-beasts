//! Shared fixed-tick physics and gameplay; independent of transport and presentation.

mod arena;
pub use arena::{ArenaBodyState, ArenaKind, FallState, MapDefinition, SkylineEvent};
mod character;
mod combat;
mod config;
pub use combat::{Consciousness, Hand, HandPhase, HandState, Vitals};
mod debug;
mod fixture;
mod melee;
mod physics;
pub use melee::{MeleePhase, MeleeState};
mod recording;
mod session;
mod simulation;

pub use character::{
    BalanceConfig, BalanceState, BodyDefinition, BodyPart, BodyShape, CharacterDefinition,
    CharacterError, JointDefinition, JointMotorConfig, JumpConfig, LocomotionConfig,
    LocomotionRagdoll, LocomotionState, MotorConfig, PassiveRagdoll, PoseRagdoll, RagdollBodyState,
    RagdollJointState,
};
pub use character::{Duel, FighterId, GripState, LowStepConfig, RecoveryConfig, StrikeHit};
pub use config::{ConfigError, PhysicsConfig};
pub use fixture::{CommandError, FixtureState};
pub use recording::{ENGINE_HASH, FixtureRecording, RecordingError};
pub use session::FixtureSession;
pub use simulation::Simulation;

pub const TICK_SECONDS: f32 = 1.0 / 60.0;
