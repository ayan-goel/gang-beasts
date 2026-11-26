use super::{
    CharacterDefinition, CharacterError, RagdollBodyState,
    duel_fighter::{Control, Fighter},
    fighter_control::MotorPresets,
    rig_frame::{RigFrame, body_states},
};
use crate::{Hand, HandState, PhysicsConfig, physics::PhysicsWorld};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FighterId {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

impl FighterId {
    pub const ALL: [Self; 8] = [
        Self::First,
        Self::Second,
        Self::Third,
        Self::Fourth,
        Self::Fifth,
        Self::Sixth,
        Self::Seventh,
        Self::Eighth,
    ];

    /// The other member of an adjacent sparring pair in fixture code.
    pub fn opponent(self) -> Self {
        Self::ALL[self as usize ^ 1]
    }
}

pub struct Duel {
    world: PhysicsWorld,
    arena: Option<crate::arena::Skyline>,
    melee: Option<crate::MeleeState>,
    fighters: Vec<Fighter>,
    definition: CharacterDefinition,
    presets: MotorPresets,
    impact: crate::combat::ImpactTuning,
    lifting: Vec<bool>,
    pulls: Vec<mantle::Pull>,
    vitals: Vec<crate::Vitals>,
    health_rules: crate::combat::HealthRules,
    changes: Vec<(FighterId, crate::Consciousness)>,
    grips: super::duel_grips::Grips,
    hits: Vec<super::StrikeHit>,
    physics: PhysicsConfig,
    tick: u64,
}

impl Duel {
    pub fn new(physics: PhysicsConfig) -> Result<Self, CharacterError> {
        Self::create(physics, false, 2)
    }
    pub fn skyline(physics: PhysicsConfig) -> Result<Self, CharacterError> {
        Self::create(physics, true, 2)
    }
    pub fn fighter_ids(&self) -> &'static [FighterId] {
        &FighterId::ALL[..self.fighters.len()]
    }
    fn create(physics: PhysicsConfig, skyline: bool, count: usize) -> Result<Self, CharacterError> {
        if !(2..=FighterId::ALL.len()).contains(&count) {
            return Err(CharacterError::Invalid("Skyline requires 2 to 8 fighters"));
        }
        physics.validate()?;
        let definition = CharacterDefinition::bundled()?;
        let mut world = PhysicsWorld::new(&physics);
        let arena = if skyline {
            Some(crate::arena::Skyline::insert(&mut world)?)
        } else {
            world
                .colliders
                .insert(ColliderBuilder::cuboid(6., 0.5, 6.).translation(vector![0., -0.5, 0.]));
            None
        };
        let spawn = |index: usize| {
            if let Some(a) = &arena {
                let slot = a.definition.spawns[index];
                (
                    slot.position,
                    Rotation::from_axis_angle(&Vector::y_axis(), slot.yaw),
                )
            } else if index == 0 {
                ([0.15, 0., 0.28], Rotation::identity())
            } else {
                (
                    [-0.15, 0., -0.28],
                    Rotation::from_quaternion(nalgebra::Quaternion::new(0., 0., 1., 0.)),
                )
            }
        };
        let mut fighters = Vec::with_capacity(count);
        for index in 0..count {
            let (at, heading) = spawn(index);
            fighters.push(Fighter::spawn(&mut world, &definition, at, heading)?);
        }
        let health_rules = crate::combat::HealthRules::bundled()?;
        Ok(Self {
            world,
            arena,
            melee: None,
            fighters,
            definition,
            presets: MotorPresets::bundled()?,
            impact: crate::combat::ImpactTuning::bundled()?,
            lifting: vec![false; count],
            pulls: vec![Default::default(); count],
            vitals: vec![crate::Vitals::new(&health_rules); count],
            health_rules,
            changes: Vec::with_capacity(count),
            hits: Vec::with_capacity(2 * count),
            grips: super::duel_grips::Grips::new(count)?,
            physics,
            tick: 0,
        })
    }
    pub fn hits(&self) -> &[super::StrikeHit] {
        &self.hits
    }
    pub fn definition(&self) -> &CharacterDefinition {
        &self.definition
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn step(&mut self) {
        self.step_with_forfeits(0);
    }
    pub fn step_with_forfeits(&mut self, forfeits: u8) {
        self.changes.clear();
        let playing = self.playing_tick();
        if forfeits != 0 {
            for &id in self.fighter_ids() {
                if forfeits & (1 << id as u8) != 0 {
                    self.eliminate(id);
                }
            }
        }
        if !playing {
            self.hits.clear();
            if let Some(a) = &mut self.arena {
                a.events.clear();
            }
            return;
        }
        // Every fighter publishes commands before the shared solve so gravity and contact age advance once.
        for &id in self.fighter_ids() {
            let vitals = self.vitals(id);
            let control = match vitals.phase {
                crate::Consciousness::Eliminated => continue,
                crate::Consciousness::Active => {
                    let pulling = self.pull_from_ledge(id);
                    let attached = [Hand::Left, Hand::Right].map(|h| self.grips.attached(id, h));
                    Control::Active {
                        pulling,
                        lifting: attached.map(|held| held && self.lifting[id as usize] && !pulling),
                        attached,
                        stamina: self.grips.stamina_mut(id),
                        arena: self.arena.as_ref(),
                    }
                }
                crate::Consciousness::KnockedOut => Control::Incapacitated { effort: 0. },
                crate::Consciousness::Recovering => Control::Incapacitated {
                    effort: vitals.waking_effort(self.tick + 1, &self.health_rules),
                },
            };
            self.fighters[id as usize].prepare(
                &mut self.world,
                &self.definition,
                &self.physics,
                self.tick,
                &self.presets,
                control,
            );
        }
        let contacts = super::duel_contacts::ContactFrame::capture(&self.world, &self.fighters);
        if let Some(a) = &mut self.arena {
            a.step(&mut self.world, self.tick + 1);
            self.grips.release_disabled(&mut self.world);
        }
        self.world.step();
        self.tick += 1;
        self.hits.clear();
        self.grips.acquire(
            &mut self.world,
            &self.fighters,
            self.arena.as_ref(),
            self.tick,
        );
        self.grips
            .settle(&mut self.world, self.physics.solver_iterations);
        if let Some(frame) = contacts {
            frame.resolve(
                &self.world,
                &mut self.fighters,
                &self.impact,
                self.tick,
                &mut self.hits,
            );
        }
        self.resolve_health();
        self.score_round();
    }
    pub fn hand_state(&self, who: FighterId, hand: Hand) -> HandState {
        let mut state = self.fighters[who as usize].hands.state(hand);
        if self.grips.attached(who, hand) {
            state.phase = crate::HandPhase::Gripping;
        }
        state
    }
    pub fn stamina(&self, who: FighterId) -> f32 {
        self.grips.stamina(who)
    }
    pub fn arena_grip(&self, who: FighterId, hand: Hand) -> Option<u16> {
        self.grips.arena_target(who, hand)
    }
    pub fn grip(&self, who: FighterId, hand: Hand) -> Option<super::GripState> {
        self.grips.state(who, hand)
    }
    pub fn body_states(&self, who: FighterId) -> Vec<RagdollBodyState> {
        body_states(RigFrame {
            world: &self.world,
            parts: self.fighters[who as usize].parts(&self.definition, &self.physics, self.tick),
        })
    }
    pub fn hand_contact_impulse(&self, who: FighterId, hand: Hand) -> f32 {
        let fist = self.fighters[who as usize].hand_collider(&self.world, hand);
        self.world
            .narrow_phase
            .contact_pairs_with(fist)
            .filter(|pair| {
                let other = if pair.collider1 == fist {
                    pair.collider2
                } else {
                    pair.collider1
                };
                pair.has_any_active_contact
                    && self.world.colliders[other]
                        .parent()
                        .and_then(|body| Fighter::other_limb(&self.fighters, who, body))
                        .is_some()
            })
            .map(|pair| pair.total_impulse_magnitude())
            .sum()
    }
    pub fn reset(&mut self) {
        let melee = self
            .melee
            .as_ref()
            .map(|m| crate::MeleeState::new(self.fighters.len(), m.round_id + 1));
        *self = Self::create(
            self.physics.clone(),
            self.arena.is_some(),
            self.fighters.len(),
        )
        .expect("validated duel configuration");
        self.melee = melee;
    }
}

#[cfg(test)]
#[path = "duel_lift_tests.rs"]
mod lift_tests;

#[path = "duel_health.rs"]
mod health;

#[path = "duel_input.rs"]
mod input;

#[path = "duel_arena.rs"]
mod map;

#[path = "duel_match.rs"]
mod rounds;

#[cfg(test)]
#[path = "ledge_tests.rs"]
mod ledge_tests;

#[cfg(test)]
#[path = "mantle_tests.rs"]
mod mantle_tests;

#[path = "duel_mantle.rs"]
mod mantle;

#[cfg(test)]
#[path = "mantle_constraints.rs"]
mod mantle_constraints;

#[cfg(test)]
#[path = "skyline_roster_tests.rs"]
mod roster_tests;

#[cfg(test)]
#[path = "headbutt_tests.rs"]
mod headbutt_tests;
