use super::{
    BodyPart, CharacterError, FighterId,
    duel_fighter::Fighter,
    grip_contact::{Target, nearest},
};
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
    target: Target,
    target_body: RigidBodyHandle,
    ledge: bool,
    created_tick: u64,
    load_n: f32,
    hand_anchor: [f32; 3],
    target_anchor: [f32; 3],
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

    pub fn mantle_tuning(&self) -> crate::combat::MantleTuning {
        self.tuning.mantle
    }
    pub fn spend_pull(&mut self, owner: FighterId) -> bool {
        let stamina = &mut self.budgets[owner as usize].stamina;
        let cost = self.tuning.mantle.stamina_per_s * crate::TICK_SECONDS;
        let enough = *stamina >= cost;
        *stamina = (*stamina - cost).max(0.);
        enough
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
            let ledges = self.slots[id]
                .each_ref()
                .map(|a| a.as_ref().is_some_and(|g| g.ledge));
            let broken = self.budgets[id].advance(loads, &self.tuning.costs, ledges);
            for hand in [Hand::Left, Hand::Right] {
                let side = hand as usize;
                if broken[side] {
                    self.failed_attacks[id][side] =
                        self.slots[id][side].as_ref().map(|grip| grip.attack_id);
                    self.release(world, owner, hand);
                } else if let Some(grip) = &mut self.slots[id][side] {
                    grip.load_n = self.budgets[id].force(side);
                }
            }
        }
    }

    pub fn state(&self, owner: FighterId, hand: Hand) -> Option<GripState> {
        let a = self.slots[owner as usize][hand as usize].as_ref()?;
        let Target::Fighter(target, region) = a.target else {
            return None;
        };
        Some(GripState {
            target,
            region,
            load_n: a.load_n,
            created_tick: a.created_tick,
            hand_anchor: a.hand_anchor,
            target_anchor: a.target_anchor,
        })
    }
    pub fn lip_anchor(
        &self,
        owner: FighterId,
        hand: Hand,
    ) -> Option<(u16, RigidBodyHandle, Point<Real>)> {
        let a = self.slots[owner as usize][hand as usize].as_ref()?;
        match a.target {
            Target::Arena(id) if a.ledge => Some((id, a.target_body, a.target_anchor.into())),
            _ => None,
        }
    }
    pub fn arena_target(&self, owner: FighterId, hand: Hand) -> Option<u16> {
        match self.slots[owner as usize][hand as usize].as_ref()?.target {
            Target::Arena(id) => Some(id),
            Target::Fighter(..) => None,
        }
    }
    pub fn attached(&self, owner: FighterId, hand: Hand) -> bool {
        self.slots[owner as usize][hand as usize].is_some()
    }
    pub fn release_disabled(&mut self, world: &mut PhysicsWorld) {
        for owner in [FighterId::First, FighterId::Second] {
            for hand in [Hand::Left, Hand::Right] {
                if self.slots[owner as usize][hand as usize]
                    .as_ref()
                    .is_some_and(|a| !world.bodies[a.target_body].is_enabled())
                {
                    self.release(world, owner, hand);
                }
            }
        }
    }
    pub fn release(&mut self, world: &mut PhysicsWorld, owner: FighterId, hand: Hand) {
        if let Some(grip) = self.slots[owner as usize][hand as usize].take() {
            // Removing only the constraint preserves both bodies' integrated momentum.
            world.impulse_joints.remove(grip.joint, true);
        }
    }
    pub fn acquire(
        &mut self,
        world: &mut PhysicsWorld,
        fighters: &[Fighter; 2],
        arena: Option<&crate::arena::Skyline>,
        tick: u64,
    ) {
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
                    target,
                    &fighters[target as usize],
                    arena,
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
                let ledge = match c.target {
                    Target::Arena(id) => arena.is_some_and(|a| a.is_lip(id)),
                    Target::Fighter(..) => false,
                };
                let mut joint = super::grip_joint::spring(a, b, &self.tuning);
                if ledge {
                    super::grip_joint::limit_slip(&mut joint, self.tuning.reach_gap_m);
                }
                let handle = world.impulse_joints.insert(fist, c.body, joint, true);
                *slot = Some(Attachment {
                    attack_id: state.attack_id,
                    joint: handle,
                    target: c.target,
                    target_body: c.body,
                    ledge,
                    created_tick: tick,
                    load_n: 0.,
                    hand_anchor: a.coords.into(),
                    target_anchor: b.coords.into(),
                });
            }
        }
    }
}
