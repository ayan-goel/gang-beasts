use rapier3d::prelude::*;

#[derive(Default)]
pub(super) struct ReversalBrake {
    departure: Vector<Real>,
    history_ticks: u32,
    remaining: u32,
}

impl ReversalBrake {
    pub fn advance(
        &mut self,
        requested: Vector<Real>,
        relative_velocity: Vector<Real>,
        quantum: f32,
        limit: u32,
    ) -> f32 {
        let Some(direction) = requested.try_normalize(0.0) else {
            // Neutral cancels pending movement but briefly remembers departure.
            // Separate key releases/presses can straddle a physics tick.
            self.remaining = 0;
            self.history_ticks = self.history_ticks.saturating_sub(1);
            return 0.0;
        };
        let opposition = (-self.departure.dot(&direction)).clamp(0.0, 1.0);
        if self.remaining == 0 && self.history_ticks > 0 && opposition > 0.0 {
            self.remaining = limit;
        }
        self.history_ticks = limit;
        let opposing_speed = -relative_velocity.dot(&direction);
        // One acceleration tick is the resolution of this controller. Resume
        // within that speed band; the timer bounds waiting under external loads.
        if self.remaining == 0 || opposition == 0.0 || opposing_speed <= quantum || quantum == 0.0 {
            self.remaining = 0;
            self.departure = direction;
            return 0.0;
        }
        self.remaining -= 1;
        // An expired interval accepts the latest direction once, preventing a
        // held request from restarting the timer indefinitely under external loads.
        if self.remaining == 0 {
            self.departure = direction;
        }
        // Squaring the negative cosine fades braking smoothly across a right-angle turn.
        opposition * opposition
    }
}

#[cfg(test)]
#[path = "reversal_tests.rs"]
mod tests;
