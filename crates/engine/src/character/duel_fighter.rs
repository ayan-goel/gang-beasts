use super::{
    BalanceConfig, BodyPart, CharacterDefinition, CharacterError, FighterId, LocomotionConfig,
    MotorConfig,
    balance::{BalanceController, BalanceState},
    fighter_control::MotorPresets,
    implicit_motor,
    rig::insert_oriented_rig,
    rig_frame::{RigFrame, RigParts},
    walking_controller::WalkingController,
};
use crate::{Hand, PhysicsConfig, combat::Hands, physics::PhysicsWorld};
use rapier3d::prelude::*;

pub(super) enum Control<'a> {
    Active {
        pulling: bool,
        stamina: &'a mut f32,
        lifting: [bool; 2],
        attached: [bool; 2],
        arena: Option<&'a crate::arena::Skyline>,
    },
    Incapacitated {
        effort: f32,
    },
}

pub(super) struct Fighter {
    pub bodies: [RigidBodyHandle; 15],
    joints: [ImpulseJointHandle; 14],
    pub hands: Hands,
    pub aim: [Option<super::punch_aim::Aim>; 2],
    pub last_hit: [[u64; FighterId::ALL.len()]; 3],
    pub headbutt: super::headbutt::Headbutt,
    pub walking: WalkingController,
    motors: MotorConfig,
    balance: BalanceController,
}

impl Fighter {
    pub fn other_limb(
        fighters: &[Self],
        owner: FighterId,
        body: RigidBodyHandle,
    ) -> Option<(FighterId, BodyPart)> {
        FighterId::ALL
            .iter()
            .copied()
            .zip(fighters)
            .filter(|(id, _)| *id != owner)
            .find_map(|(id, fighter)| {
                fighter
                    .bodies
                    .iter()
                    .position(|&handle| handle == body)
                    .map(|index| (id, BodyPart::ALL[index]))
            })
    }

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
            aim: [None; 2],
            headbutt: super::headbutt::Headbutt::new()?,
            last_hit: Default::default(),
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
        control: Control<'_>,
    ) {
        let mut rig = RigFrame {
            parts: self.parts(definition, physics, tick),
            world,
        };
        let (pulling, lifting, attached, arena, stamina) = match control {
            Control::Active {
                pulling,
                lifting,
                attached,
                arena,
                stamina,
            } => (pulling, lifting, attached, arena, stamina),
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
        for hand in [Hand::Left, Hand::Right] {
            let weight = self.hands.aim_weight(hand);
            if weight == 0. {
                self.aim[hand as usize] = None;
            } else if let Some(aim) = self.aim[hand as usize] {
                aim.steer(
                    definition,
                    hand,
                    weight,
                    &mut self.walking.state.targets,
                    &mut targets,
                );
            }
        }
        if self.walking.state.recovery_strength == 0. && !pulling {
            self.headbutt.drive(
                definition,
                &mut self.motors,
                &mut targets,
                stamina,
                &mut self.hands,
            );
        } else {
            self.headbutt.cancel();
        }
        if let Some(map) = arena {
            super::ledge_reach::aim(&mut rig, map, &self.hands, attached, &mut self.motors);
            super::prop_reach::aim(rig.read(), map, &self.hands, &mut targets);
        }
        if lifting.contains(&true) {
            super::lifting::apply(
                &mut rig,
                &self.hands.tuning,
                lifting,
                &self.balance.config,
                &mut targets,
            );
        }
        if pulling {
            let bracing = [Hand::Left, Hand::Right].map(|h| {
                attached[h as usize]
                    || matches!(
                        self.hands.state(h).phase,
                        crate::HandPhase::Reach | crate::HandPhase::Idle
                    )
            });
            super::mantle_drive::tuck(definition, &mut targets, bracing);
        }
        implicit_motor::drive(
            &mut rig,
            &self.motors,
            &targets,
            Some(&mut self.balance),
            true,
        );
    }
    pub fn weight_on(&self, world: &PhysicsWorld, surface: RigidBodyHandle) -> bool {
        use super::BodyPart::*;
        [
            Pelvis, Chest, LeftThigh, RightThigh, LeftShin, RightShin, LeftFoot, RightFoot,
        ]
        .into_iter()
        .any(|part| {
            super::support::foot_support(
                world,
                self.bodies[part.index()],
                &self.bodies,
                &self.balance.config,
            )
            .is_some_and(|s| s.body == Some(surface))
        })
    }
    pub fn hand_collider(&self, world: &PhysicsWorld, side: Hand) -> ColliderHandle {
        world.bodies[self.bodies[side.body().index()]].colliders()[0]
    }
}
