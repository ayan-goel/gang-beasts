use super::{Duel, FighterId};
use crate::{CharacterError, Hand};

impl Duel {
    pub fn set_input(
        &mut self,
        who: FighterId,
        direction: [f32; 2],
        running: bool,
    ) -> Result<(), CharacterError> {
        if self.vitals(who).phase != crate::Consciousness::Active {
            if !direction.iter().all(|v| v.is_finite()) {
                return Err(CharacterError::Invalid("movement intent must be finite"));
            }
            return Ok(());
        }
        self.fighters[who as usize]
            .walking
            .set_input(direction, running)
    }
    pub fn set_lift(&mut self, who: FighterId, held: bool) {
        self.lifting[who as usize] = held && self.vitals(who).phase == crate::Consciousness::Active;
    }
    pub fn set_jump(&mut self, who: FighterId, held: bool) {
        if self.vitals(who).phase == crate::Consciousness::Active {
            self.fighters[who as usize].walking.set_jump(held);
        }
    }
    pub fn set_hand(&mut self, who: FighterId, hand: Hand, held: bool) {
        // A fall can switch the arm to its get-up pose while the grip persists.
        // A repeated held input cannot start a punch with that attached hand.
        if held
            && (self.grip(who, hand).is_some()
                || self.vitals(who).phase != crate::Consciousness::Active)
        {
            return;
        }
        if !held {
            self.grips.release(&mut self.world, who, hand);
        }
        let fighter = &mut self.fighters[who as usize];
        if fighter.walking.state.recovery_strength == 0.0 {
            fighter.hands.set(hand, held);
        }
    }
    pub fn clear_hands(&mut self, who: FighterId) {
        self.fighters[who as usize].hands.release_inputs();
        for hand in [Hand::Left, Hand::Right] {
            self.grips.release(&mut self.world, who, hand);
        }
    }
    pub fn clear_jump(&mut self, who: FighterId) {
        self.fighters[who as usize].walking.clear_jump_input();
    }
}
