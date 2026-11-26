use super::{Duel, FighterId};
use crate::{BodyPart, physics::PhysicsWorld};

#[derive(Clone, Copy, Default)]
pub(super) struct Footsteps {
    loaded: [bool; 2],
    last: Option<u64>,
}

impl Duel {
    pub fn last_footstep(&self, id: FighterId) -> Option<u64> {
        self.footsteps[id as usize].last
    }

    pub(super) fn record_footsteps(&mut self) {
        for &id in self.fighter_ids() {
            let slot = id as usize;
            let fighter = &self.fighters[slot];
            let hip = &self.world.bodies[fighter.bodies[BodyPart::Pelvis.index()]];
            let chest = &self.world.bodies[fighter.bodies[BodyPart::Chest.index()]];
            let feet = &mut self.footsteps[slot];
            if self.vitals[slot].phase != crate::Consciousness::Active
                || !fighter.walking.has_movement_input()
                || fighter.walking.state.recovery_strength > 0.
                || chest.translation().y <= hip.translation().y
                || hip.linvel().xz().norm_squared() < 0.45 * 0.45
                || self.landings[slot].is_some_and(|(tick, _)| self.tick - tick < 15)
            {
                feet.loaded = [true; 2];
                continue;
            }

            let loaded = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
                let sole = &self.world.bodies[fighter.bodies[part.index()]];
                floor_impulse(&self.world, sole.colliders()[0]) >= 0.25
            });
            // Require a new load on a sole. The cooldown coalesces a two-foot plant
            // and suppresses rapid contact chatter on a dragging foot.
            if (0..2).any(|side| loaded[side] && !feet.loaded[side])
                && feet.last.is_none_or(|tick| self.tick - tick >= 9)
            {
                feet.last = Some(self.tick);
            }
            feet.loaded = loaded;
        }
    }

    /// Most recent qualifying fixed-surface landing, as round tick and upward impulse in N·s.
    pub fn last_landing(&self, id: FighterId) -> Option<(u64, u8)> {
        self.landings[id as usize]
    }

    pub(super) fn record_landings(&mut self, approach: &[f32; 8]) {
        for &id in self.fighter_ids() {
            let slot = id as usize;
            let last = self.landings[slot];
            // Resting weight and rolling contacts can span many solves. Require downward
            // motion before impact and give each fall a quarter second to settle.
            if approach[slot] > -1.5 || last.is_some_and(|(tick, _)| self.tick - tick < 15) {
                continue;
            }
            let rig = &self.fighters[slot].bodies;
            if !self.world.bodies[rig[BodyPart::Pelvis.index()]].is_enabled() {
                continue;
            }
            let impulse: f32 = rig
                .iter()
                .flat_map(|&h| self.world.bodies[h].colliders())
                .map(|&handle| floor_impulse(&self.world, handle))
                .sum();
            if impulse >= 12. {
                self.landings[slot] = Some((self.tick, impulse.round().min(255.) as u8));
            }
        }
    }
}

fn floor_impulse(world: &PhysicsWorld, limb: rapier3d::prelude::ColliderHandle) -> f32 {
    let mut up = 0.;
    for pair in world
        .narrow_phase
        .contact_pairs_with(limb)
        .filter(|p| p.has_any_active_contact)
    {
        let first = pair.collider1 == limb;
        let other = if first {
            pair.collider2
        } else {
            pair.collider1
        };
        let surface = &world.colliders[other];
        if surface.is_sensor()
            || !surface.is_enabled()
            || surface
                .parent()
                .is_some_and(|h| !world.bodies[h].is_fixed())
        {
            continue;
        }
        for manifold in &pair.manifolds {
            let normal_y = if first {
                -manifold.data.normal.y
            } else {
                manifold.data.normal.y
            };
            if normal_y >= 0.7 {
                up += normal_y * manifold.points.iter().map(|p| p.data.impulse).sum::<f32>();
            }
        }
    }
    up
}
