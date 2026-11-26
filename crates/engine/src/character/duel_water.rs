use super::{Duel, FighterId};
use crate::{BodyPart, Consciousness, Hand, TICK_SECONDS};
use rapier3d::prelude::*;

const BREATH_TICKS: u16 = 360;
const FLOAT_TICKS: u16 = 720;
const FATIGUE_TICKS: u16 = 180;

#[derive(Clone, Copy, Default)]
pub(super) struct Swimmer {
    immersion: u16,
    submerged: u16,
    bite: Option<(RigidBodyHandle, u64)>,
    next_bite: u64,
}

impl Swimmer {
    pub(super) fn drowned(&self) -> bool {
        self.submerged >= BREATH_TICKS
    }
    fn float_strength(&self) -> f32 {
        1.0 - 0.56
            * (f32::from(self.immersion.saturating_sub(FLOAT_TICKS)) / f32::from(FATIGUE_TICKS))
                .min(1.)
    }
}

fn immerse(body: &mut RigidBody, level: f32, gravity: f32, lift: f32, stroke: Vector<Real>) {
    // Partial immersion distributes buoyancy among the ragdoll's limbs. Velocity
    // damping is mass-scaled so a small hand and a heavy torso settle together.
    let fraction = ((level - body.center_of_mass().y + 0.14) / 0.28).clamp(0., 1.);
    if fraction == 0. {
        return;
    }
    let accel = Vector::y() * (gravity * lift) + stroke - body.linvel() * 2.4;
    body.apply_impulse(accel * (fraction * body.mass() * TICK_SECONDS), true);
    body.set_angvel(*body.angvel() * (1. - fraction * 1.6 * TICK_SECONDS), true);
}

impl Duel {
    pub(super) fn apply_water(&mut self) {
        let Some(arena) = &self.arena else { return };
        let Some(level) = arena.definition.tour.as_ref().and_then(|s| s.water) else {
            return;
        };
        let gravity = -self.physics.gravity_y;
        for body in arena.crate_bodies() {
            if self.world.bodies[body].is_enabled() && self.world.bodies[body].is_dynamic() {
                immerse(
                    &mut self.world.bodies[body],
                    level,
                    gravity,
                    1.32,
                    Vector::zeros(),
                );
            }
        }
        for &id in self.fighter_ids() {
            let slot = id as usize;
            if self.vitals[slot].phase == Consciousness::Eliminated {
                continue;
            }
            let fighter = &self.fighters[slot];
            let swimmer = &mut self.swimmers[slot];
            let hip = fighter.bodies[BodyPart::Pelvis.index()];
            let head = fighter.bodies[BodyPart::Head.index()];
            let hip_pos = *self.world.bodies[hip].center_of_mass();
            if hip_pos.y < level + 0.20 {
                swimmer.immersion = swimmer.immersion.saturating_add(1);
            } else {
                swimmer.immersion = swimmer.immersion.saturating_sub(4);
            }
            if self.world.bodies[head].center_of_mass().y < level - 0.1 {
                swimmer.submerged = swimmer.submerged.saturating_add(1);
            } else {
                swimmer.submerged = swimmer.submerged.saturating_sub(3);
            }
            let active = self.vitals[slot].phase == Consciousness::Active;
            let pushing = active && self.pulls[slot].held;
            let reserve = swimmer.float_strength();
            let stroke = if active {
                (fighter.walking.movement_intent() * 7.0
                    + Vector::y() * if pushing { 5.6 } else { 0. })
                    * reserve
            } else {
                Vector::zeros()
            };
            for &h in &fighter.bodies {
                immerse(
                    &mut self.world.bodies[h],
                    level,
                    gravity,
                    1.24 * reserve,
                    stroke,
                );
            }
            if swimmer.bite.is_none() && self.tick >= swimmer.next_bite {
                for (obj, shark, collider) in arena.grip_surfaces(&self.world) {
                    if arena.definition.objects[usize::from(obj) - 1]
                        .art
                        .as_deref()
                        != Some("shark")
                        || self.world.bodies[shark].linvel().norm_squared() < 1.0
                    {
                        continue;
                    }
                    let caught = self
                        .world
                        .narrow_phase
                        .contact_pairs_with(collider)
                        .any(|p| {
                            let limb = if p.collider1 == collider {
                                p.collider2
                            } else {
                                p.collider1
                            };
                            p.has_any_active_contact
                                && self.world.colliders[limb]
                                    .parent()
                                    .is_some_and(|h| fighter.bodies.contains(&h))
                        });
                    if caught {
                        swimmer.bite = Some((shark, self.tick));
                        swimmer.next_bite = self.tick + 180;
                        break;
                    }
                }
            }
            if let Some((shark, start)) = swimmer.bite {
                let elapsed = self.tick - start;
                let anchored = [Hand::Left, Hand::Right]
                    .into_iter()
                    .any(|hand| self.grips.lip_anchor(id, hand).is_some());
                if elapsed >= 40 || (elapsed >= 8 && (pushing || anchored)) {
                    swimmer.bite = None;
                } else {
                    let jaw = &self.world.bodies[shark];
                    let chase = (jaw.center_of_mass() - hip_pos) * 5. + jaw.linvel()
                        - self.world.bodies[hip].linvel();
                    let tow = chase.cap_magnitude(10.) * (3. * TICK_SECONDS);
                    for &h in &fighter.bodies {
                        let b = &mut self.world.bodies[h];
                        b.apply_impulse(tow * b.mass(), true);
                    }
                }
            }
        }
    }

    pub fn breath_remaining(&self, id: FighterId) -> u16 {
        BREATH_TICKS.saturating_sub(self.swimmers[id as usize].submerged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MapDefinition, MeleePhase, MeleeState, PhysicsConfig};
    const OWNER: FighterId = FighterId::First;

    fn water_match() -> Duel {
        let map = MapDefinition::tour_catalog()
            .unwrap()
            .iter()
            .find(|m| m.id == "night-trawler")
            .unwrap()
            .clone();
        let mut game = Duel::on_map(PhysicsConfig::default(), 2, Some(map), None).unwrap();
        let mut state = MeleeState::new(2, 1);
        state.phase = MeleePhase::Playing;
        game.melee = Some(state);
        game
    }
    fn hip(game: &Duel) -> Vector<Real> {
        *game.world.bodies[game.fighters[0].bodies[BodyPart::Pelvis.index()]].translation()
    }
    fn place(game: &mut Duel, position: Vector<Real>) {
        let offset = position - hip(game);
        for body in game.fighters[0].bodies {
            let b = &mut game.world.bodies[body];
            b.set_translation(b.translation() + offset, true);
            b.set_linvel(Vector::zeros(), true);
            b.set_angvel(Vector::zeros(), true);
        }
        game.world
            .bodies
            .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
    }

    #[test]
    fn a_submerged_player_can_stroke_toward_shore_and_get_air_before_tiring() {
        let mut game = water_match();
        let level = game
            .map_definition()
            .unwrap()
            .tour
            .as_ref()
            .unwrap()
            .water
            .unwrap();
        place(&mut game, vector![18., level - 1.5, 0.]);
        game.set_input(OWNER, [-1., 0.], false).unwrap();
        game.set_jump(OWNER, true);
        for _ in 0..150 {
            game.step();
        }
        assert!(
            hip(&game).x < 16.,
            "swimmer failed to move, {:?}",
            hip(&game)
        );
        assert!(
            hip(&game).y > level - 0.8,
            "swimmer failed to surface, {:?}",
            hip(&game)
        );
        assert!(game.breath_remaining(OWNER) > BREATH_TICKS / 2);
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
    }

    #[test]
    fn a_stranded_swimmer_exhausts_buoyancy_then_drowns_before_the_fall_plane() {
        let mut game = water_match();
        place(&mut game, vector![18., -1.7, 0.]);
        for _ in 0..2100 {
            game.step();
            if game.vitals(OWNER).phase == Consciousness::Eliminated {
                break;
            }
        }
        assert!(
            game.swimmers[0].drowned(),
            "swimmer should lose air after tiring, hip {:?}, immersion {}",
            hip(&game),
            game.swimmers[0].immersion
        );
        assert_eq!(game.match_state().unwrap().scores(), &[0, 1]);
        assert!(hip(&game).y > game.map_definition().unwrap().fall_y);
    }

    #[test]
    fn drowning_ends_the_round_once_and_a_resurfaced_head_recovers_air() {
        let mut game = water_match();
        place(&mut game, vector![18., -3., 0.]);
        game.swimmers[0].submerged = 180;
        game.step();
        let available = game.breath_remaining(OWNER);
        place(&mut game, vector![18., 1., 0.]);
        game.step();
        assert!(game.breath_remaining(OWNER) > available);
        place(&mut game, vector![18., -3., 0.]);
        game.swimmers[0].submerged = BREATH_TICKS - 1;
        game.step();
        game.step();
        assert_eq!(game.match_state().unwrap().scores(), &[0, 1]);
        assert_eq!(game.vitals(OWNER).phase, Consciousness::Eliminated);
    }

    #[test]
    fn a_swimmer_catches_the_trawlers_rolling_bow_and_climbs_aboard() {
        let mut game = water_match();
        game.tick = 720;
        for hand in [Hand::Left, Hand::Right] {
            game.set_hand(OWNER, hand, true);
        }
        for _ in 0..40 {
            game.step();
        }
        // Approach the clear starboard bow. The port-side wheelhouse physically
        // occupies the landing area and correctly blocks a pull through its wall.
        let fighter = &game.fighters[0];
        let pelvis = &game.world.bodies[fighter.bodies[BodyPart::Pelvis.index()]];
        let turn = Rotation::from_axis_angle(&Vector::y_axis(), std::f32::consts::FRAC_PI_2)
            * pelvis.rotation().inverse();
        let origin = *pelvis.translation();
        for body in fighter.bodies {
            let b = &mut game.world.bodies[body];
            let pos = turn * (b.translation() - origin) + vector![1.65, -0.52, -7.02];
            b.set_position(Isometry::from_parts(pos.into(), turn * b.rotation()), true);
            b.set_linvel(vector![0., 0., 0.3], true);
            b.set_angvel(Vector::zeros(), true);
        }
        game.world
            .bodies
            .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
        game.set_input(OWNER, [0., 1.], false).unwrap();
        game.set_jump(OWNER, true);
        let mut caught = false;
        for _ in 0..360 {
            game.step();
            caught |= [Hand::Left, Hand::Right]
                .iter()
                .any(|&h| game.arena_grip(OWNER, h) == Some(1));
            if caught
                && hip(&game).z > -6.15
                && hip(&game).y > 0.38
                && game.fighters[0].walking.state.feet.contains(&true)
            {
                return;
            }
        }
        panic!(
            "failed bow rescue; caught={caught}, hip={:?}, grip={:?}, {:?}",
            hip(&game),
            game.arena_grip(OWNER, Hand::Left),
            game.arena_grip(OWNER, Hand::Right)
        );
    }

    #[test]
    fn breaching_shark_catches_a_contact_then_a_swim_stroke_releases_it() {
        let mut game = water_match();
        let start = game
            .map_definition()
            .unwrap()
            .tour
            .as_ref()
            .unwrap()
            .motions
            .iter()
            .find(|m| m.object == 4)
            .unwrap()
            .start;
        game.tick = u64::from(start) + 15;
        game.step();
        let shark = game.arena_states()[3].position;
        place(&mut game, Vector::from(shark) + vector![0., 0.6, 0.]);
        for _ in 0..15 {
            game.step();
            if game.swimmers[0].bite.is_some() {
                break;
            }
        }
        assert!(
            game.swimmers[0].bite.is_some(),
            "no physical contact with breaching shark at {shark:?}"
        );
        game.set_jump(OWNER, true);
        for _ in 0..10 {
            game.step();
        }
        assert!(game.swimmers[0].bite.is_none());
        assert!(game.swimmers[0].next_bite > game.tick);
    }
}
