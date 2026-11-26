use super::{Duel, FighterId};
use crate::{BodyPart, Hand, arena::Landing};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Default)]
pub(super) struct Pull {
    pub held: bool,
    support: Option<RigidBodyHandle>,
    crossing: bool,
    driven_ticks: u16,
}

impl Duel {
    fn landing(&self, id: FighterId) -> Option<Landing> {
        let map = self.arena.as_ref()?;
        [Hand::Left, Hand::Right].into_iter().find_map(|hand| {
            let (object, _, anchor) = self.grips.lip_anchor(id, hand)?;
            map.landing(&self.world, object, anchor, self.tick + 1)
        })
    }

    pub(super) fn pull_from_ledge(&mut self, id: FighterId) -> bool {
        let i = id as usize;
        if !self.pulls[i].held {
            return false;
        }
        let Some(landing) = self.landing(id) else {
            self.pulls[i] = Pull {
                held: true,
                ..Default::default()
            };
            return false;
        };
        let fighter = &self.fighters[i];
        let hip = *self.world.bodies[fighter.bodies[BodyPart::Pelvis.index()]].center_of_mass();
        let height = hip.y - landing.edge.y;
        let depth = (hip - landing.edge).dot(&landing.inward);
        let pull = &mut self.pulls[i];
        if pull.support != Some(landing.body) {
            if height > 0.12 || depth > 0.12 {
                return false;
            }
            *pull = Pull {
                held: true,
                support: Some(landing.body),
                ..Default::default()
            };
        }
        if !landing.clear(&self.world, &fighter.bodies, hip) {
            return false;
        }
        let hands = [Hand::Left, Hand::Right].map(|h| {
            self.grips
                .lip_anchor(id, h)
                .is_some_and(|(_, b, _)| b == landing.body)
        });
        pull.crossing |= height > 0.2;
        // Shallow torso contact can tip back off the lip as the hands release.
        // Keep the grip until the pelvis has room for the grounded get-up.
        if pull.crossing
            && height > 0.12
            && depth > 0.38
            && fighter.weight_on(&self.world, landing.body)
        {
            for h in [Hand::Left, Hand::Right] {
                if hands[h as usize] {
                    self.grips.release(&mut self.world, id, h);
                    self.fighters[i].hands.set(h, false);
                }
            }
            *pull = Pull::default();
            return false;
        }
        let tuning = self.grips.mantle_tuning();
        if pull.driven_ticks >= tuning.maximum_ticks || !self.grips.spend_pull(id) {
            return false;
        }
        pull.driven_ticks += 1;
        let cap = if hands == [true; 2] {
            tuning.two_hand_force_n
        } else {
            tuning.one_hand_force_n
        };
        super::super::mantle_drive::drive(
            &mut self.world,
            &fighter.bodies,
            &landing,
            pull.crossing,
            cap,
            tuning,
            self.physics.gravity_y,
        );
        true
    }
}
