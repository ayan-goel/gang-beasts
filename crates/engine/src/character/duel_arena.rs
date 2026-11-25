use super::{Duel, FighterId};
use crate::{ArenaBodyState, BodyPart, MapDefinition, SkylineEvent};
impl Duel {
    pub fn map_definition(&self) -> Option<&MapDefinition> {
        self.arena.as_ref().map(|a| &a.definition)
    }
    pub fn arena_states(&self) -> Vec<ArenaBodyState> {
        self.arena
            .as_ref()
            .map_or_else(Vec::new, |a| a.states(&self.world))
    }
    pub fn arena_events(&self) -> &[SkylineEvent] {
        self.arena.as_ref().map_or(&[], |a| &a.events)
    }
    pub fn fall_state(&self, id: FighterId) -> Option<crate::FallState> {
        self.arena.as_ref().map(|a| {
            a.fall_state(
                &self.world,
                self.fighters[id as usize].bodies[BodyPart::Pelvis.index()],
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Hand, PhysicsConfig};
    use rapier3d::prelude::*;

    #[test]
    fn a_hand_at_the_walkway_lip_stays_attached_during_detachment_and_releases_on_retirement() {
        let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
        let owner = FighterId::First;
        let hand = Hand::Left;
        game.set_hand(owner, hand, true);
        for _ in 0..40 {
            game.step();
        }
        let fighter = &game.fighters[0];
        let shift = vector![-12.03, 0.035, -2.]
            - game.world.bodies[fighter.bodies[hand.body().index()]].translation();
        for h in fighter.bodies {
            let pose = game.world.bodies[h].translation() + shift;
            game.world.bodies[h].set_translation(pose, true);
        }
        game.world
            .bodies
            .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
        game.grips.acquire(
            &mut game.world,
            &game.fighters,
            game.arena.as_ref(),
            game.tick,
        );
        assert_eq!(game.arena_grip(owner, hand), Some(8));
        game.tick = 7199;
        game.step();
        assert_eq!(game.arena_grip(owner, hand), Some(8));
        assert_eq!(game.arena_events(), &[SkylineEvent::PanelDetached(8)]);
        let panel = game
            .arena
            .as_ref()
            .unwrap()
            .grip_surfaces(&game.world)
            .find(|(id, _, _)| *id == 8)
            .unwrap()
            .1;
        game.world.bodies[panel].set_translation(vector![-12., -29., -2.], true);
        game.step();
        assert_eq!(game.arena_grip(owner, hand), None);
        assert!(!game.world.bodies[panel].is_enabled());
    }
}
