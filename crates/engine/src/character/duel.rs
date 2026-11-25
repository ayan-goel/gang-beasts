use super::{
    CharacterDefinition, CharacterError, RagdollBodyState,
    duel_fighter::Fighter,
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
        Ok(Self {
            world,
            fighters: [first, second],
            definition,
            presets: MotorPresets::bundled()?,
            impact: crate::combat::ImpactTuning::bundled()?,
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
    pub fn clear_hands(&mut self, who: FighterId) {
        self.fighters[who as usize].hands.release_inputs();
        for hand in [Hand::Left, Hand::Right] {
            self.grips.release(&mut self.world, who, hand);
        }
    }
    pub fn clear_jump(&mut self, who: FighterId) {
        self.fighters[who as usize].walking.clear_jump_input();
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn step(&mut self) {
        // Both controllers publish their bounded commands before the one room-wide
        // solve. Stepping per fighter would double gravity and age contacts unevenly.
        for fighter in &mut self.fighters {
            fighter.prepare(
                &mut self.world,
                &self.definition,
                &self.physics,
                self.tick,
                &self.presets,
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
    }
    pub fn set_input(
        &mut self,
        who: FighterId,
        direction: [f32; 2],
        running: bool,
    ) -> Result<(), CharacterError> {
        self.fighters[who as usize]
            .walking
            .set_input(direction, running)
    }
    pub fn set_jump(&mut self, who: FighterId, held: bool) {
        self.fighters[who as usize].walking.set_jump(held);
    }
    pub fn set_hand(&mut self, who: FighterId, hand: Hand, held: bool) {
        // A fall can switch the arm to its get-up pose while the grip persists.
        // A repeated held input cannot start a punch with that attached hand.
        if held && self.grip(who, hand).is_some() {
            return;
        }
        if !held {
            self.grips.release(&mut self.world, who, hand);
        }
        let fighter = &mut self.fighters[who as usize];
        if fighter.walking.state.recovery_strength == 0.0 {
            fighter.hands.set(hand, held);
        }
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
