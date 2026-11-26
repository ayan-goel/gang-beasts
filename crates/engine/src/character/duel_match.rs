use super::{Duel, FighterId};
use crate::{CharacterError, Consciousness, Hand, MeleePhase, MeleeState, PhysicsConfig};

impl Duel {
    pub fn skyline_match(physics: PhysicsConfig) -> Result<Self, CharacterError> {
        Self::skyline_match_with_players(physics, 2)
    }
    pub fn skyline_match_with_players(
        physics: PhysicsConfig,
        count: usize,
    ) -> Result<Self, CharacterError> {
        let mut game = Self::create(physics, true, count)?;
        game.melee = Some(MeleeState::new(count, 1));
        Ok(game)
    }
    pub fn match_state(&self) -> Option<&MeleeState> {
        self.melee.as_ref()
    }

    pub(super) fn accepts_input(&self, id: FighterId) -> bool {
        self.vitals(id).phase == Consciousness::Active
            && self
                .melee
                .as_ref()
                .is_none_or(|m| m.phase == MeleePhase::Playing)
    }
    pub(super) fn playing_tick(&mut self) -> bool {
        let Some(m) = self.melee.as_mut() else {
            return true;
        };
        if m.phase == MeleePhase::Playing {
            return true;
        }
        if m.between_ticks() {
            let m = self.melee.take();
            *self = Self::create(self.physics.clone(), true, self.fighters.len())
                .expect("validated Skyline round");
            self.melee = m;
        }
        false
    }
    pub(super) fn score_round(&mut self) {
        if self.melee.is_none() {
            return;
        }
        let mut alive = 0;
        for &id in self.fighter_ids() {
            if self.fall_state(id).is_some_and(|f| f.below_plane) {
                self.eliminate(id);
            }
            if self.vitals(id).phase != Consciousness::Eliminated {
                alive |= 1 << id as u8;
            }
        }
        let expired = self.tick
            >= self
                .arena
                .as_ref()
                .expect("Skyline match")
                .definition
                .schedule
                .end_tick
                .into();
        self.melee.as_mut().unwrap().settle(alive, expired);
    }
    pub(super) fn eliminate(&mut self, id: FighterId) {
        if self.vitals(id).phase == Consciousness::Eliminated {
            return;
        }
        self.clear_hands(id);
        for &owner in self.fighter_ids() {
            for hand in [Hand::Left, Hand::Right] {
                if self.grip(owner, hand).is_some_and(|g| g.target == id) {
                    self.grips.release(&mut self.world, owner, hand);
                }
            }
        }
        let i = id as usize;
        self.vitals[i].phase = Consciousness::Eliminated;
        self.vitals[i].since_tick = self.tick;
        self.fighters[i].hands.clear();
        self.fighters[i].headbutt.cancel();
        self.fighters[i].walking.reset();
        self.lifting[i] = false;
        for handle in self.fighters[i].bodies {
            self.world.bodies[handle].set_enabled(false);
        }
        self.changes.retain(|(actor, _)| *actor != id);
        self.changes.push((id, Consciousness::Eliminated));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rapier3d::prelude::*;

    #[test]
    fn two_pelvis_crossings_in_one_solve_are_a_draw_and_remove_all_gameplay_bodies() {
        let mut game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
        for _ in 0..180 {
            game.step();
        }
        for fighter in &game.fighters {
            for &h in &fighter.bodies {
                let p = *game.world.bodies[h].translation() - Vector::y() * 15.;
                game.world.bodies[h].set_translation(p, true);
            }
        }
        game.step();
        assert_eq!(game.match_state().unwrap().scores(), &[0, 0]);
        assert_eq!(game.match_state().unwrap().winner, None);
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::RoundResults);
        assert_eq!(game.consciousness_changes().len(), 2);
        for id in [FighterId::First, FighterId::Second] {
            assert_eq!(game.vitals(id).phase, Consciousness::Eliminated);
            for h in game.fighters[id as usize].bodies {
                assert!(!game.world.bodies[h].is_enabled());
            }
        }
    }

    #[test]
    fn the_final_deadline_ends_an_unresolved_round_once() {
        let mut game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
        for _ in 0..180 {
            game.step();
        }
        game.tick = u64::from(game.map_definition().unwrap().schedule.end_tick) - 1;
        game.step();
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::RoundResults);
        assert_eq!(game.match_state().unwrap().winner, None);
        assert_eq!(game.match_state().unwrap().scores(), &[0, 0]);
    }
}
