use crate::{
    SubmitError, controls,
    inbox::{Inbox, STALE_TICKS},
};
use brawler_engine::{CharacterError, Duel, FighterId, MeleePhase, MeleeState, PhysicsConfig};

pub struct Room {
    game: Duel,
    tick: u64,
    match_serial: u32,
    inboxes: Vec<Inbox>,
    attached: Vec<bool>,
    forfeits: u8,
    finished: bool,
}

impl Room {
    pub fn skyline() -> Result<Self, CharacterError> {
        Self::skyline_with_players(2)
    }
    pub fn skyline_with_players(count: usize) -> Result<Self, CharacterError> {
        Self::skyline_in_match(count, 0)
    }
    pub(crate) fn skyline_in_match(
        count: usize,
        match_serial: u32,
    ) -> Result<Self, CharacterError> {
        Ok(Self {
            game: Duel::skyline_match_with_players(PhysicsConfig::default(), count)?,
            tick: 0,
            match_serial,
            inboxes: (0..count).map(|_| Inbox::default()).collect(),
            attached: vec![true; count],
            forfeits: 0,
            finished: false,
        })
    }

    pub fn game(&self) -> &Duel {
        &self.game
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    fn melee(&self) -> &MeleeState {
        self.game.match_state().expect("Skyline room owns a match")
    }
    pub(crate) fn epoch(&self) -> u64 {
        // A rematch reuses displayed round numbers, so its input acknowledgements need a fresh namespace.
        (u64::from(self.match_serial) << 32) | u64::from(self.melee().round_number)
    }
    pub fn applied(&self, id: FighterId) -> Option<u32> {
        self.inboxes.get(id as usize)?.applied
    }

    /// The transport supplies the authenticated fighter and synchronized round.
    pub fn submit(&mut self, id: FighterId, round: u64, bytes: &[u8]) -> Result<(), SubmitError> {
        if self.attached.get(id as usize) != Some(&true) || self.forfeits & (1 << id as usize) != 0
        {
            return Err(SubmitError::Detached);
        }
        if self.epoch() != round {
            return Err(SubmitError::Round);
        }
        if self.melee().phase != MeleePhase::Playing {
            return Err(SubmitError::Phase);
        }
        self.inboxes[id as usize].enqueue(bytes, self.tick)
    }

    pub fn detach(&mut self, id: FighterId) {
        let Some(attached) = self.attached.get_mut(id as usize) else {
            return;
        };
        *attached = false;
        self.expire(id);
    }

    pub(crate) fn expire(&mut self, id: FighterId) {
        self.inboxes[id as usize].cancel();
        controls::neutralize(&mut self.game, id);
    }

    pub(crate) fn replace_connection(&mut self, id: FighterId, connected: bool) {
        self.detach(id);
        // Each socket starts an independent sequence namespace after old input and grips are cleared.
        self.inboxes[id as usize] = Inbox::default();
        self.attached[id as usize] = connected;
    }

    pub fn attach(&mut self, id: FighterId) {
        if let Some(attached) = self.attached.get_mut(id as usize) {
            *attached = true;
        }
    }

    pub fn finished_for_lack_of_players(&self) -> bool {
        self.finished
    }

    pub fn start_next_round(
        &mut self,
        previous: &[Option<FighterId>],
    ) -> Result<(), CharacterError> {
        if self.finished
            || previous
                .iter()
                .flatten()
                .any(|id| self.forfeits & (1 << *id as usize) != 0)
        {
            return Err(CharacterError::Invalid(
                "a closed room or forfeited player cannot resume",
            ));
        }
        self.game.start_next_round(previous)?;
        // Connection state follows a returning player across compact fighter indices.
        // A newcomer must establish their own binding before submitting any input.
        self.attached = previous
            .iter()
            .map(|id| id.is_some_and(|id| self.attached[id as usize]))
            .collect();
        self.inboxes = (0..previous.len()).map(|_| Inbox::default()).collect();
        self.forfeits = 0;
        self.tick += 1;
        Ok(())
    }
    pub fn step(&mut self) {
        self.step_with_forfeits(0);
    }
    pub fn step_with_forfeits(&mut self, mask: u8) {
        if self.finished {
            return;
        }
        for &id in self.game.fighter_ids() {
            let bit = 1 << id as usize;
            if mask & bit != 0 && self.forfeits & bit == 0 {
                self.detach(id);
                self.forfeits |= bit;
            }
        }
        // Give players the full results interval before releasing an uncontested room.
        let results_over = self.melee().phase == MeleePhase::MatchResults
            || (self.melee().phase == MeleePhase::RoundResults
                && self.melee().remaining_ticks == 1);
        if results_over && self.attached.len() - (self.forfeits.count_ones() as usize) < 2 {
            self.finished = true;
            return;
        }
        let round = self.melee().round_id;
        let playing = self.melee().phase == MeleePhase::Playing;
        if playing {
            for &id in self.game.fighter_ids() {
                let inbox = &mut self.inboxes[id as usize];
                if inbox
                    .last_received
                    .is_some_and(|at| self.tick - at >= STALE_TICKS)
                {
                    inbox.cancel();
                    controls::neutralize(&mut self.game, id);
                }
                for frame in inbox.pending.drain(..) {
                    controls::apply(&mut self.game, id, frame);
                    inbox.applied = Some(frame.sequence);
                }
            }
        }
        self.game.step_with_forfeits(self.forfeits);
        self.tick += 1;
        if playing && self.melee().phase != MeleePhase::Playing {
            for &id in self.game.fighter_ids() {
                self.inboxes[id as usize].cancel();
                controls::neutralize(&mut self.game, id);
            }
        }
        // A rebuilt round has its own acknowledgement namespace. Only transport
        // sessions synchronized to this epoch may introduce a new first sequence.
        if self.melee().round_id != round {
            self.inboxes.fill_with(Inbox::default);
        }
    }
}

#[cfg(test)]
mod epoch_tests {
    use super::*;
    use brawler_protocol::InputFrame;

    #[test]
    fn a_fresh_match_starts_round_one_and_rejects_the_previous_matches_round_one_input() {
        let mut room = Room::skyline_in_match(2, 1).unwrap();
        assert_eq!(room.melee().round_number, 1);
        let epoch = (1u64 << 32) + 1;
        assert_eq!(room.capture().unwrap().round, epoch);
        for _ in 0..180 {
            room.step();
        }
        let bytes = InputFrame::default().encode().unwrap();
        assert_eq!(
            room.submit(FighterId::First, 1, &bytes),
            Err(SubmitError::Round)
        );
        room.submit(FighterId::First, epoch, &bytes).unwrap();
        room.step();
        assert_eq!(room.applied(FighterId::First), Some(0));
    }
}
