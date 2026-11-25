use serde::Serialize;

const COUNTDOWN: u16 = 180;
const RESULTS: u16 = 300;
const WINS: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleePhase {
    Countdown,
    Playing,
    RoundResults,
    MatchResults,
}

pub struct MeleeState {
    pub phase: MeleePhase,
    pub round_id: u64,
    pub round_number: u32,
    pub remaining_ticks: u16,
    pub winner: Option<u8>,
    scores: Vec<u8>,
}

impl MeleeState {
    pub(crate) fn new(count: usize, round_id: u64) -> Self {
        assert!((2..=8).contains(&count));
        Self {
            phase: MeleePhase::Countdown,
            round_id,
            round_number: 1,
            remaining_ticks: COUNTDOWN,
            winner: None,
            scores: vec![0; count],
        }
    }
    pub fn scores(&self) -> &[u8] {
        &self.scores
    }
    pub(crate) fn between_ticks(&mut self) -> bool {
        if !matches!(self.phase, MeleePhase::Countdown | MeleePhase::RoundResults) {
            return false;
        }
        self.remaining_ticks -= 1;
        if self.remaining_ticks > 0 {
            return false;
        }
        match self.phase {
            MeleePhase::Countdown => self.phase = MeleePhase::Playing,
            MeleePhase::RoundResults
                if self
                    .winner
                    .is_some_and(|id| self.scores[id as usize] >= WINS) =>
            {
                self.phase = MeleePhase::MatchResults
            }
            MeleePhase::RoundResults => {
                self.round_id += 1;
                self.round_number += 1;
                self.remaining_ticks = COUNTDOWN;
                self.winner = None;
                self.phase = MeleePhase::Countdown;
                return true;
            }
            _ => unreachable!(),
        }
        false
    }
    pub(crate) fn settle(&mut self, alive: u8, expired: bool) {
        if self.phase != MeleePhase::Playing {
            return;
        }
        let survivors = alive.count_ones();
        if survivors > 1 && !expired {
            return;
        }
        // All pelvis crossings have been applied before scoring. Leaving Playing
        // makes duplicate results for this round unable to increment the score.
        self.winner = (survivors == 1).then(|| alive.trailing_zeros() as u8);
        if let Some(id) = self.winner {
            self.scores[id as usize] += 1;
        }
        self.phase = MeleePhase::RoundResults;
        self.remaining_ticks = RESULTS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn final_tick_eliminations_and_timeout_draws_are_atomic() {
        for (alive, expired, winner) in [
            (0, false, None),
            (0b11000, true, None),
            (0b10000, true, Some(4)),
        ] {
            let mut m = MeleeState::new(8, 45);
            for _ in 0..COUNTDOWN {
                m.between_ticks();
            }
            m.settle(alive, expired);
            m.settle(1, true);
            assert_eq!(m.winner, winner);
            assert_eq!(m.scores().iter().sum::<u8>(), u8::from(winner.is_some()));
            for _ in 0..RESULTS {
                m.between_ticks();
            }
            assert_eq!(m.round_id, 46);
            assert_eq!(m.phase, MeleePhase::Countdown);
        }
    }
}
