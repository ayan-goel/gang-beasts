use std::time::Duration;

const HZ: u128 = 60;
const SECOND: u128 = 1_000_000_000;
const MAX_PASS: u64 = 4;

#[derive(Default)]
pub(super) struct Cadence {
    completed: u64,
}

impl Cadence {
    pub fn debt(&self, elapsed: Duration) -> u64 {
        let target = elapsed.as_nanos() * HZ / SECOND;
        u64::try_from(target)
            .unwrap_or(u64::MAX)
            .saturating_sub(self.completed)
    }

    pub fn take(&mut self, elapsed: Duration) -> u64 {
        let count = self.debt(elapsed).min(MAX_PASS);
        self.completed += count;
        count
    }

    pub fn wait(&self, elapsed: Duration) -> Duration {
        // Derive deadlines from the origin to avoid accumulating nanosecond rounding at 60 Hz.
        let next = ((u128::from(self.completed) + 1) * SECOND).div_ceil(HZ);
        Duration::from_nanos(next.saturating_sub(elapsed.as_nanos()).min(SECOND) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractional_boundaries_and_a_long_stall_preserve_every_fixed_tick() {
        let mut clock = Cadence::default();
        assert_eq!(clock.take(Duration::from_nanos(16_666_666)), 0);
        assert_eq!(
            clock.wait(Duration::from_nanos(16_666_666)),
            Duration::from_nanos(1)
        );
        assert_eq!(clock.take(Duration::from_nanos(16_666_667)), 1);
        let second = Duration::from_secs(1);
        while clock.debt(second) > 4 {
            assert_eq!(clock.take(second), 4);
            assert!(clock.wait(second).is_zero());
        }
        assert_eq!(clock.take(second), 3);
        assert_eq!(clock.completed, 60);
        assert_eq!(clock.wait(second), Duration::from_nanos(16_666_667));
    }
}
