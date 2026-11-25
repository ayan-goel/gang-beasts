use super::{
    BalanceConfig, BodyPart, CharacterDefinition, CharacterError, LocomotionConfig, MotorConfig,
    balance::{BalanceController, BalanceState},
    fighter_control::MotorPresets,
    implicit_motor,
    rig::insert_oriented_rig,
    rig_frame::{RigFrame, RigParts},
    walking_controller::WalkingController,
};
use crate::{Hand, PhysicsConfig, combat::Hands, physics::PhysicsWorld};
use rapier3d::prelude::*;

pub(super) struct Fighter {
    pub bodies: [RigidBodyHandle; 15],
    joints: [ImpulseJointHandle; 14],
    pub hands: Hands,
    pub walking: WalkingController,
    motors: MotorConfig,
    balance: BalanceController,
}

impl Fighter {
    pub fn spawn(
        world: &mut PhysicsWorld,
        definition: &CharacterDefinition,
        position: [f32; 3],
        heading: Rotation<Real>,
    ) -> Result<Self, CharacterError> {
        let (bodies, joints) = insert_oriented_rig(world, definition, position, heading);
        Ok(Self {
            bodies,
            joints,
            hands: Hands::new()?,
            walking: WalkingController::new(LocomotionConfig::bundled()?),
            motors: MotorConfig::standing()?,
            balance: BalanceController {
                config: BalanceConfig::bundled()?,
                state: BalanceState::default(),
            },
        })
    }
    pub fn parts<'a>(
        &self,
        definition: &'a CharacterDefinition,
        physics: &'a PhysicsConfig,
        tick: u64,
    ) -> RigParts<'a> {
        RigParts {
            bodies: self.bodies,
            joints: self.joints,
            definition,
            physics,
            tick,
        }
    }
    pub fn prepare(
        &mut self,
        world: &mut PhysicsWorld,
        definition: &CharacterDefinition,
        physics: &PhysicsConfig,
        tick: u64,
        presets: &MotorPresets,
    ) {
        let mut rig = RigFrame {
            parts: self.parts(definition, physics, tick),
            world,
        };
        let targets = presets.prepare(
            &mut rig,
            &mut self.motors,
            &mut self.walking,
            &mut self.hands,
            true,
        );
        implicit_motor::drive(
            &mut rig,
            &self.motors,
            &targets,
            Some(&mut self.balance),
            true,
        );
    }
    pub fn hand_collider(&self, world: &PhysicsWorld, side: Hand) -> ColliderHandle {
        let part = match side {
            Hand::Left => BodyPart::LeftHand,
            Hand::Right => BodyPart::RightHand,
        };
        world.bodies[self.bodies[part.index()]].colliders()[0]
    }
}
