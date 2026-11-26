use super::{Duel, FighterId};
use crate::{Consciousness, Vitals};

impl Duel {
    pub fn vitals(&self, id: FighterId) -> Vitals {
        self.vitals[id as usize]
    }
    pub fn consciousness_changes(&self) -> &[(FighterId, Consciousness)] {
        &self.changes
    }

    pub(super) fn resolve_health(&mut self) {
        // All attackers complete contact resolution before anyone can lose control.
        self.hits
            .retain(|hit| self.vitals[hit.defender as usize].phase == Consciousness::Active);
        let mut damage = [0.; FighterId::ALL.len()];
        for hit in &self.hits {
            damage[hit.defender as usize] += hit.damage;
        }
        for &id in self.fighter_ids() {
            let i = id as usize;
            if let Some(next) = self.vitals[i].advance(self.tick, damage[i], &self.health_rules) {
                if next == Consciousness::KnockedOut {
                    self.clear_hands(id);
                    self.fighters[i].hands.clear();
                    self.fighters[i].headbutt.cancel();
                    self.fighters[i].walking.reset();
                    self.lifting[i] = false;
                    self.pulls[i] = Default::default();
                }
                self.changes.push((id, next));
            }
        }
    }
}

#[cfg(test)]
#[path = "duel_health_tests.rs"]
mod tests;
