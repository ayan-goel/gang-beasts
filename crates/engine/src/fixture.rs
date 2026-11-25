use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixtureState {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub mass: f32,
}

#[derive(Debug, Error, PartialEq)]
pub enum CommandError {
    #[error("impulse must be finite with magnitude at most 10 N·s")]
    InvalidImpulse,
    #[error("an impulse is already queued for the next tick")]
    ImpulsePending,
}

pub(crate) fn validate_impulse(impulse: [f32; 3]) -> Result<(), CommandError> {
    // INVARIANT: diagnostics cannot enqueue nonfinite or unbounded actuation.
    // The one-kilogram fixture permits at most 10 m/s of added speed per tick.
    let magnitude_squared: f32 = impulse.iter().map(|value| value * value).sum();
    if !magnitude_squared.is_finite() || magnitude_squared > 100.0 {
        return Err(CommandError::InvalidImpulse);
    }
    Ok(())
}
