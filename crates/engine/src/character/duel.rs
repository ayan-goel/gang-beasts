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
}

impl FighterId {
    pub fn opponent(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

pub struct Duel {
    world: PhysicsWorld,
    fighters: [Fighter; 2],
    definition: CharacterDefinition,
    presets: MotorPresets,
    impact: crate::combat::ImpactTuning,
    lifting: [bool; 2],
    vitals: [crate::Vitals; 2],
    health_rules: crate::combat::HealthRules,
    changes: Vec<(FighterId, crate::Consciousness)>,
    grips: super::duel_grips::Grips,
    hits: Vec<super::StrikeHit>,
    physics: PhysicsConfig,
    tick: u64,
}

impl Duel {
    pub fn new(physics: PhysicsConfig) -> Result<Self, CharacterError> {
        physics.validate()?;
        let definition = CharacterDefinition::bundled()?;
        let mut world = PhysicsWorld::new(&physics);
        world
            .colliders
            .insert(ColliderBuilder::cuboid(6.0, 0.5, 6.0).translation(vector![0., -0.5, 0.]));
        let first = Fighter::spawn(
            &mut world,
            &definition,
            [0.15, 0., 0.28],
            Rotation::identity(),
        )?;
        let opposite = Rotation::from_quaternion(nalgebra::Quaternion::new(0., 0., 1., 0.));
        let second = Fighter::spawn(&mut world, &definition, [-0.15, 0., -0.28], opposite)?;
        let health_rules = crate::combat::HealthRules::bundled()?;
        Ok(Self {
            world,
            fighters: [first, second],
            definition,
            presets: MotorPresets::bundled()?,
            impact: crate::combat::ImpactTuning::bundled()?,
            lifting: [false; 2],
            vitals: [crate::Vitals::new(&health_rules); 2],
            health_rules,
            changes: Vec::with_capacity(2),
            hits: Vec::with_capacity(4),
            grips: super::duel_grips::Grips::new()?,
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
        // Both controllers publish their bounded commands before the one room-wide
        // solve. Stepping per fighter would double gravity and age contacts unevenly.
        for id in [FighterId::First, FighterId::Second] {
            let lifting = [Hand::Left, Hand::Right]
                .map(|h| self.lifting[id as usize] && self.grip(id, h).is_some());
            let vitals = self.vitals(id);
            let control = match vitals.phase {
                crate::Consciousness::Active => Control::Active { lifting },
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
        self.world.step();
        self.tick += 1;
        self.hits.clear();
        self.grips
            .acquire(&mut self.world, &self.fighters, self.tick);
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
    }
    pub fn hand_state(&self, who: FighterId, hand: Hand) -> HandState {
        let mut state = self.fighters[who as usize].hands.state(hand);
        if self.grip(who, hand).is_some() {
            state.phase = crate::HandPhase::Gripping;
        }
        state
    }
    pub fn stamina(&self, who: FighterId) -> f32 {
        self.grips.stamina(who)
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
        let opponent = &self.fighters[who.opponent() as usize];
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
                        .is_some_and(|body| opponent.bodies.contains(&body))
            })
            .map(|pair| pair.total_impulse_magnitude())
            .sum()
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.physics.clone()).expect("validated duel configuration");
    }
}

#[cfg(test)]
#[path = "duel_lift_tests.rs"]
mod lift_tests;

#[path = "duel_health.rs"]
mod health;

#[path = "duel_input.rs"]
mod input;
