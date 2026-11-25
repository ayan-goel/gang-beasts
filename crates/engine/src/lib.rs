//! Shared fixed-tick physics and gameplay; independent of transport and presentation.

mod config;
mod physics;
mod simulation;

pub use config::{ConfigError, PhysicsConfig};
pub use simulation::Simulation;

pub const TICK_SECONDS: f32 = 1.0 / 60.0;
