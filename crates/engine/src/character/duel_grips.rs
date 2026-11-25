use super::{BodyPart, CharacterError, FighterId, duel_fighter::Fighter, grip_contact::nearest};
use crate::{Hand, HandPhase, combat::GripTuning, physics::PhysicsWorld};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct GripState {
    pub target: FighterId,
    pub region: BodyPart,
    pub created_tick: u64,
    pub hand_anchor: [f32; 3],
    pub target_anchor: [f32; 3],
}

struct Attachment {
    joint: ImpulseJointHandle,
    state: GripState,
}
pub(super) struct Grips {
    slots: [[Option<Attachment>; 2]; 2],
    tuning: GripTuning,
}

impl Grips {
    pub fn new() -> Result<Self, CharacterError> {
        Ok(Self {
            slots: Default::default(),
            tuning: GripTuning::bundled()?,
        })
    }

    pub fn state(&self, owner: FighterId, hand: Hand) -> Option<GripState> {
        self.slots[owner as usize][hand as usize]
            .as_ref()
            .map(|a| a.state)
    }
    pub fn release(&mut self, world: &mut PhysicsWorld, owner: FighterId, hand: Hand) {
        if let Some(grip) = self.slots[owner as usize][hand as usize].take() {
            // Removing only the constraint preserves both bodies' integrated momentum.
            world.impulse_joints.remove(grip.joint, true);
        }
    }
    pub fn retract(&mut self, world: &mut PhysicsWorld, fighters: &[Fighter; 2]) {
        for owner in [FighterId::First, FighterId::Second] {
            for hand in [Hand::Left, Hand::Right] {
                if fighters[owner as usize].hands.state(hand).phase != HandPhase::Reach {
                    self.release(world, owner, hand);
                }
            }
        }
    }
    pub fn acquire(&mut self, world: &mut PhysicsWorld, fighters: &[Fighter; 2], tick: u64) {
        for owner in [FighterId::First, FighterId::Second] {
            let fighter = &fighters[owner as usize];
            let target = owner.opponent();
            for hand in [Hand::Left, Hand::Right] {
                let slot = &mut self.slots[owner as usize][hand as usize];
                if slot.is_some() || fighter.hands.state(hand).phase != HandPhase::Reach {
                    continue;
                }
                let collider = fighter.hand_collider(world, hand);
                let Some(c) = nearest(
                    world,
                    collider,
                    &fighters[target as usize],
                    self.tuning.reach_gap_m,
                ) else {
                    continue;
                };
                let fist = fighter.bodies[hand.body().index()];
                let a = world.bodies[fist]
                    .position()
                    .inverse_transform_point(&c.point);
                let b = world.bodies[c.body]
                    .position()
                    .inverse_transform_point(&c.point);
                let mut joint = GenericJointBuilder::new(JointAxesMask::empty())
                    .local_anchor1(a)
                    .local_anchor2(b);
                // Independent axis ceilings bound the vector norm. Free angular
                // axes let a held limb twist and keep reciprocal grips compliant.
                for axis in [JointAxis::LinX, JointAxis::LinY, JointAxis::LinZ] {
                    joint = joint
                        .motor_model(axis, MotorModel::ForceBased)
                        .motor_position(
                            axis,
                            0.,
                            self.tuning.stiffness_n_m,
                            self.tuning.damping_n_s_m,
                        )
                        .motor_max_force(axis, self.tuning.maximum_force_n / 3.0_f32.sqrt());
                }
                let handle = world.impulse_joints.insert(fist, c.body, joint, true);
                *slot = Some(Attachment {
                    joint: handle,
                    state: GripState {
                        target,
                        region: c.region,
                        created_tick: tick,
                        hand_anchor: a.coords.into(),
                        target_anchor: b.coords.into(),
                    },
                });
            }
        }
    }
}
