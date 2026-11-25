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
