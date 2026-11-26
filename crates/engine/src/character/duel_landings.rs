use super::{Duel, FighterId};
use crate::{BodyPart, physics::PhysicsWorld};

impl Duel {
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
