use super::{BodyPart, CharacterError, FighterId, duel_fighter::Fighter, grip_contact::nearest};
use crate::{
    Hand, HandPhase,
    combat::{GripBudget, GripTuning},
    physics::PhysicsWorld,
};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct GripState {
    pub target: FighterId,
    pub region: BodyPart,
    pub load_n: f32,
    pub created_tick: u64,
    pub hand_anchor: [f32; 3],
    pub target_anchor: [f32; 3],
}

struct Attachment {
    attack_id: u64,
    joint: ImpulseJointHandle,
    state: GripState,
}
pub(super) struct Grips {
    slots: [[Option<Attachment>; 2]; 2],
    budgets: [GripBudget; 2],
    failed_attacks: [[Option<u64>; 2]; 2],
    tuning: GripTuning,
}

impl Grips {
    pub fn new() -> Result<Self, CharacterError> {
        let tuning = GripTuning::bundled()?;
        Ok(Self {
            slots: Default::default(),
            budgets: std::array::from_fn(|_| GripBudget::new(&tuning.costs)),
            failed_attacks: Default::default(),
            tuning,
        })
    }

    pub fn stamina(&self, owner: FighterId) -> f32 {
        self.budgets[owner as usize].stamina
    }

    pub fn settle(&mut self, world: &mut PhysicsWorld, iterations: usize) {
        for owner in [FighterId::First, FighterId::Second] {
            let id = owner as usize;
            let loads = self.slots[id].each_ref().map(|slot| {
                let grip = slot.as_ref()?;
                let joint = world
                    .impulse_joints
                    .get(grip.joint)
                    .expect("owned grip joint");
                Some(super::grip_joint::force(&joint.data, iterations))
            });
            let broken = self.budgets[id].advance(loads, &self.tuning.costs);
            for hand in [Hand::Left, Hand::Right] {
                let side = hand as usize;
                if broken[side] {
                    self.failed_attacks[id][side] =
                        self.slots[id][side].as_ref().map(|grip| grip.attack_id);
                    self.release(world, owner, hand);
                } else if let Some(grip) = &mut self.slots[id][side] {
                    grip.state.load_n = self.budgets[id].force(side);
                }
            }
        }
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
    pub fn acquire(&mut self, world: &mut PhysicsWorld, fighters: &[Fighter; 2], tick: u64) {
        for owner in [FighterId::First, FighterId::Second] {
            let fighter = &fighters[owner as usize];
            let target = owner.opponent();
            for hand in [Hand::Left, Hand::Right] {
                let slot = &mut self.slots[owner as usize][hand as usize];
                let state = fighter.hands.state(hand);
                if slot.is_some()
                    || state.phase != HandPhase::Reach
                    || self.failed_attacks[owner as usize][hand as usize] == Some(state.attack_id)
                    || self.budgets[owner as usize].stamina <= 0.
                {
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
                let joint = super::grip_joint::spring(a, b, &self.tuning);
                let handle = world.impulse_joints.insert(fist, c.body, joint, true);
                *slot = Some(Attachment {
                    attack_id: state.attack_id,
                    joint: handle,
                    state: GripState {
                        target,
                        region: c.region,
                        created_tick: tick,
                        load_n: 0.,
                        hand_anchor: a.coords.into(),
                        target_anchor: b.coords.into(),
                    },
                });
            }
        }
    }
}
