use crate::{
    SubmitError, controls,
    inbox::{Inbox, STALE_TICKS},
};
use brawler_engine::{CharacterError, Duel, FighterId, MeleePhase, MeleeState, PhysicsConfig};

pub struct Room {
    game: Duel,
    tick: u64,
    inboxes: Vec<Inbox>,
    attached: Vec<bool>,
}

impl Room {
    pub fn skyline() -> Result<Self, CharacterError> {
        Self::skyline_with_players(2)
    }
    pub fn skyline_with_players(count: usize) -> Result<Self, CharacterError> {
        Ok(Self {
            game: Duel::skyline_match_with_players(PhysicsConfig::default(), count)?,
            tick: 0,
            inboxes: (0..count).map(|_| Inbox::default()).collect(),
            attached: vec![true; count],
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
        self.inboxes.get(id as usize)?.applied
    }

    /// The transport supplies the authenticated fighter and synchronized round.
    pub fn submit(&mut self, id: FighterId, round: u64, bytes: &[u8]) -> Result<(), SubmitError> {
        if self.attached.get(id as usize) != Some(&true) {
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

    pub fn step(&mut self) {
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
        self.game.step();
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
