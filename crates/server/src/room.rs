use crate::{
    SubmitError, controls,
    inbox::{Inbox, STALE_TICKS},
};
use brawler_engine::{CharacterError, Duel, FighterId, MeleePhase, MeleeState, PhysicsConfig};

const FIGHTERS: [FighterId; 2] = [FighterId::First, FighterId::Second];

pub struct Room {
    game: Duel,
    tick: u64,
    inboxes: [Inbox; 2],
    attached: [bool; 2],
}

impl Room {
    pub fn skyline() -> Result<Self, CharacterError> {
        Ok(Self {
            game: Duel::skyline_match(PhysicsConfig::default())?,
            tick: 0,
            inboxes: Default::default(),
            attached: [true; 2],
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
    pub fn applied(&self, id: FighterId) -> Option<u32> {
        self.inboxes[id as usize].applied
    }

    /// The transport supplies the authenticated fighter and synchronized round.
    pub fn submit(&mut self, id: FighterId, round: u64, bytes: &[u8]) -> Result<(), SubmitError> {
        if !self.attached[id as usize] {
            return Err(SubmitError::Detached);
        }
        if self.melee().round_id != round {
            return Err(SubmitError::Round);
        }
        if self.melee().phase != MeleePhase::Playing {
            return Err(SubmitError::Phase);
        }
        self.inboxes[id as usize].enqueue(bytes, self.tick)
    }

    pub fn detach(&mut self, id: FighterId) {
        self.attached[id as usize] = false;
        self.inboxes[id as usize].cancel();
        controls::neutralize(&mut self.game, id);
    }

    pub fn attach(&mut self, id: FighterId) {
        self.attached[id as usize] = true;
    }

    pub fn step(&mut self) {
        let round = self.melee().round_id;
        let playing = self.melee().phase == MeleePhase::Playing;
        if playing {
            for id in FIGHTERS {
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
        self.game.step();
        self.tick += 1;
        if playing && self.melee().phase != MeleePhase::Playing {
            for id in FIGHTERS {
                self.inboxes[id as usize].cancel();
                controls::neutralize(&mut self.game, id);
            }
        }
        // A rebuilt round has its own acknowledgement namespace. Only transport
        // sessions synchronized to this epoch may introduce a new first sequence.
        if self.melee().round_id != round {
            self.inboxes = Default::default();
        }
    }
}
