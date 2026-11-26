use crate::{CharacterError, FighterId};
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

#[derive(Clone)]
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
    fn match_won(&self) -> bool {
        self.winner
            .is_some_and(|id| self.scores[id as usize] >= WINS)
    }
    pub fn next_round_due(&self) -> bool {
        self.phase == MeleePhase::RoundResults && self.remaining_ticks == 1 && !self.match_won()
    }
    pub(crate) fn next_lineup(
        &self,
        previous: &[Option<FighterId>],
    ) -> Result<Self, CharacterError> {
        if !self.next_round_due() || !(2..=8).contains(&previous.len()) {
            return Err(CharacterError::Invalid(
                "roster requires a due round and 2 through 8 players",
            ));
        }
        let mut used = 0u8;
        let mut scores = vec![0; previous.len()];
        for (slot, id) in previous.iter().enumerate() {
            if let Some(id) = id {
                let index = *id as usize;
                if index >= self.scores.len() || used & (1 << index) != 0 {
                    return Err(CharacterError::Invalid(
                        "returning fighter must exist exactly once",
                    ));
                }
                scores[slot] = self.scores[index];
                used |= 1 << index;
            }
        }
        let mut next = self.clone();
        next.between_ticks();
        next.scores = scores;
        Ok(next)
    }
    pub(crate) fn between_ticks(&mut self) {
        if !matches!(self.phase, MeleePhase::Countdown | MeleePhase::RoundResults) {
            return;
        }
        self.remaining_ticks -= 1;
        if self.remaining_ticks > 0 {
            return;
        }
        match self.phase {
            MeleePhase::Countdown => self.phase = MeleePhase::Playing,
            MeleePhase::RoundResults if self.match_won() => self.phase = MeleePhase::MatchResults,
            MeleePhase::RoundResults => {
                self.round_id += 1;
                self.round_number += 1;
                self.remaining_ticks = COUNTDOWN;
                self.winner = None;
                self.phase = MeleePhase::Countdown;
            }
            _ => unreachable!(),
        }
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
