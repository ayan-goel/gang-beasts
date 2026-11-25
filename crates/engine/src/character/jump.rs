use super::{CharacterError, PassiveRagdoll};
use crate::TICK_SECONDS;
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JumpConfig {
    pub takeoff_speed_m_s: f32,
    pub buffer_ticks: u8,
    pub coyote_ticks: u8,
    pub air_acceleration_fraction: f32,
    pub air_velocity_budget_m_s: f32,
}

impl Default for JumpConfig {
    fn default() -> Self {
        Self {
            takeoff_speed_m_s: 4.5,
            buffer_ticks: 6,
            coyote_ticks: 6,
            air_acceleration_fraction: 0.2,
            air_velocity_budget_m_s: 1.0,
        }
    }
}

impl JumpConfig {
    pub(super) fn validate(&self) -> Result<(), CharacterError> {
        if !(0.01..=4.5).contains(&self.takeoff_speed_m_s)
            || !(1..=6).contains(&self.buffer_ticks)
            || self.coyote_ticks > 6
            || !(0.0..=0.5).contains(&self.air_acceleration_fraction)
            || !(0.0..=1.0).contains(&self.air_velocity_budget_m_s)
        {
            return Err(CharacterError::Invalid(
                "jump tuning outside impulse or timing bounds",
            ));
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct Jump {
    held: bool,
    buffered: u8,
    coyote: u8,
    was_supported: bool,
    spent: bool,
    air_remaining: f32,
}

impl Jump {
    pub fn input(&mut self, held: bool, config: &JumpConfig) {
        if held && !self.held {
            self.buffered = config.buffer_ticks;
        }
        self.held = held;
    }

    pub fn clear_input(&mut self) {
        self.held = false;
        self.buffered = 0;
    }

    pub fn takeoff(&mut self, supported: bool, upright: bool, config: &JumpConfig) -> bool {
        if !upright {
            *self = Self {
                held: self.held,
                ..Self::default()
            };
            return false;
        }
        if supported {
            if !self.was_supported {
                self.spent = false;
                self.air_remaining = 0.0;
            }
            self.coyote = config.coyote_ticks;
        }
        self.was_supported = supported;
        // One support episode funds one launch. Contact can persist for a tick
        // after takeoff, and a press/release pair can arrive before the next tick.
        let launch = self.buffered > 0 && !self.spent && (supported || self.coyote > 0);
        if launch {
            self.spent = true;
            self.buffered = 0;
            self.air_remaining = config.air_velocity_budget_m_s;
        }
        self.buffered = self.buffered.saturating_sub(1);
        if !supported {
            self.coyote = self.coyote.saturating_sub(1);
        }
        launch
    }

    pub fn steer(&mut self, intent: Vector<Real>, acceleration: f32) -> Vector<Real> {
        // Spending absolute delta-v bounds drift even in zero gravity or on reversal.
        let delta = (intent * acceleration * TICK_SECONDS).cap_magnitude(self.air_remaining);
        self.air_remaining = (self.air_remaining - delta.norm()).max(0.0);
        delta
    }
}

pub(super) fn apply_velocity_change(rig: &mut PassiveRagdoll, delta: Vector<Real>) {
    // Mass weighting preserves limb-relative velocities and existing platform carry.
    for handle in rig.bodies {
        let body = &mut rig.world.bodies[handle];
        body.apply_impulse(delta * body.mass(), true);
    }
}
