use super::{
    BalanceConfig, CharacterDefinition, CharacterError, LocomotionConfig, MotorConfig,
    balance::{BalanceController, BalanceState},
    fighter_control::MotorPresets,
    implicit_motor,
    rig::insert_oriented_rig,
    rig_frame::{RigFrame, RigParts},
    walking_controller::WalkingController,
};
use crate::{Hand, PhysicsConfig, combat::Hands, physics::PhysicsWorld};
use rapier3d::prelude::*;

pub(super) enum Control {
    Active { lifting: [bool; 2] },
    Incapacitated { effort: f32 },
}

pub(super) struct Fighter {
    pub bodies: [RigidBodyHandle; 15],
    joints: [ImpulseJointHandle; 14],
    pub hands: Hands,
    pub last_hit: [u64; 2],
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
            last_hit: [0; 2],
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
        control: Control,
    ) {
        let mut rig = RigFrame {
            parts: self.parts(definition, physics, tick),
            world,
        };
        let lifting = match control {
            Control::Active { lifting } => lifting,
            Control::Incapacitated { effort } => {
                // Only internal pose motors wake during the ramp. Ground assistance
                // and player actions resume after the timed transition completes.
                presets.blend(&mut self.motors, 0.);
                for motor in &mut self.motors.joints {
                    motor.max_torque_nm *= effort;
                }
                let targets = super::recovery::tuck_targets(rig.read(), effort)
                    .map(|[x, y, z]| Rotation::from_euler_angles(x, y, z));
                implicit_motor::configure(&mut rig, &self.motors, &targets, effort > 0.);
                return;
            }
        };
        let mut targets = presets.prepare(
            &mut rig,
            &mut self.motors,
            &mut self.walking,
            &mut self.hands,
            true,
        );
        if lifting.contains(&true) {
            super::lifting::apply(
                &mut rig,
                &self.hands.tuning,
                lifting,
                &self.balance.config,
                &mut targets,
            );
        }
        implicit_motor::drive(
            &mut rig,
            &self.motors,
            &targets,
            Some(&mut self.balance),
            true,
        );
    }
    pub fn hand_collider(&self, world: &PhysicsWorld, side: Hand) -> ColliderHandle {
        world.bodies[self.bodies[side.body().index()]].colliders()[0]
    }
}
