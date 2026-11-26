use std::time::{Duration, Instant};

const PROBE_INTERVAL: Duration = Duration::from_secs(2);
const REPLY_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Probe {
    Wait,
    Ping(u32),
    Expired,
}

pub(super) struct Heartbeat {
    sequence: u32,
    pending: bool,
    deadline: Instant,
}

impl Heartbeat {
    pub fn new(now: Instant) -> Self {
        Self {
            sequence: 0,
            pending: false,
            deadline: now + PROBE_INTERVAL,
        }
    }

    pub fn poll(&mut self, now: Instant) -> Probe {
        if now < self.deadline {
            return Probe::Wait;
        }
        if self.pending {
            return Probe::Expired;
        }
        self.sequence = self.sequence.wrapping_add(1);
        self.pending = true;
        self.deadline = now + REPLY_TIMEOUT;
        Probe::Ping(self.sequence)
    }

    pub fn pong(&mut self, payload: &[u8], now: Instant) {
        if self.pending && now < self.deadline && payload == self.sequence.to_le_bytes() {
            self.pending = false;
            self.deadline = now + PROBE_INTERVAL;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_outstanding_ping_and_only_its_reply_extend_the_connection() {
        let now = Instant::now();
        let mut heartbeat = Heartbeat::new(now);
        let ping = now + Duration::from_secs(2);
        assert_eq!(heartbeat.poll(ping - Duration::from_nanos(1)), Probe::Wait);
        assert_eq!(heartbeat.poll(ping), Probe::Ping(1));
        heartbeat.pong(&2_u32.to_le_bytes(), ping);
        let deadline = ping + Duration::from_secs(6);
        assert_eq!(
            heartbeat.poll(deadline - Duration::from_nanos(1)),
            Probe::Wait
        );
        heartbeat.pong(&1_u32.to_le_bytes(), deadline);
        assert_eq!(heartbeat.poll(deadline), Probe::Expired);

        let mut live = Heartbeat::new(now);
        assert_eq!(live.poll(ping), Probe::Ping(1));
        live.pong(&1_u32.to_le_bytes(), ping + Duration::from_secs(1));
        assert_eq!(live.poll(ping + Duration::from_secs(2)), Probe::Wait);
        assert_eq!(live.poll(ping + Duration::from_secs(3)), Probe::Ping(2));
        live.pong(&1_u32.to_le_bytes(), ping + Duration::from_secs(4));
        assert_eq!(live.poll(ping + Duration::from_secs(9)), Probe::Expired);
    }
}
