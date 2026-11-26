use super::{BodyPart, CharacterDefinition, CharacterError, MotorConfig};
use crate::combat::Hands;
use rapier3d::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rules {
    windup_ticks: u8,
    strike_ticks: u8,
    recovery_ticks: u8,
    windup_pitch: [f32; 2],
    strike_pitch: [f32; 2],
    stiffness: [f32; 2],
    damping: [f32; 2],
    torque_cap_nm: [f32; 2],
    stamina_cost: f32,
    damage_scale: f32,
}

pub(super) struct Headbutt {
    rules: Rules,
    held: bool,
    pending: bool,
    age: u8,
    pub attack_id: u64,
}

impl Headbutt {
    pub fn new() -> Result<Self, CharacterError> {
        let rules: Rules = serde_json::from_str(include_str!("../../../../config/headbutt.json"))?;
        if ![rules.windup_ticks, rules.strike_ticks, rules.recovery_ticks]
            .into_iter()
            .all(|n| (1..=60).contains(&n))
            || !rules
                .windup_pitch
                .into_iter()
                .chain(rules.strike_pitch)
                .all(|n| (-0.5..=0.5).contains(&n))
            || !rules
                .stiffness
                .into_iter()
                .all(|n| (0.0..=600.).contains(&n))
            || !rules.damping.into_iter().all(|n| (0.0..=20.).contains(&n))
            || !rules
                .torque_cap_nm
                .into_iter()
                .all(|n| (0.0..=150.).contains(&n))
            || !(0.0..=100.).contains(&rules.stamina_cost)
            || !(0.01..=4.).contains(&rules.damage_scale)
        {
            return Err(CharacterError::Invalid("invalid headbutt timing or effort"));
        }
        Ok(Self {
            rules,
            held: false,
            pending: false,
            age: 0,
            attack_id: 0,
        })
    }
    fn duration(&self) -> u8 {
        self.rules.windup_ticks + self.rules.strike_ticks + self.rules.recovery_ticks
    }
    pub fn input(&mut self, down: bool) {
        if down && !self.held && (self.age == 0 || self.age == self.duration()) {
            self.pending = true;
        }
        self.held = down;
    }
    pub fn clear_input(&mut self) {
        self.held = false;
        self.pending = false;
    }
    pub fn cancel(&mut self) {
        self.clear_input();
        self.age = 0;
    }
    pub fn strike(&self) -> Option<(u64, f32)> {
        (self.age > self.rules.windup_ticks
            && self.age <= self.rules.windup_ticks + self.rules.strike_ticks)
            .then_some((self.attack_id, self.rules.damage_scale))
    }

    pub fn drive(
        &mut self,
        definition: &CharacterDefinition,
        motors: &mut MotorConfig,
        targets: &mut [Rotation<Real>; 14],
        stamina: &mut f32,
        hands: &mut Hands,
    ) {
        // Spend stamina when the solver accepts the tap so a canceled input costs nothing.
        if std::mem::take(&mut self.pending) && *stamina >= self.rules.stamina_cost {
            *stamina -= self.rules.stamina_cost;
            self.attack_id = hands.allocate_attack();
            self.age = 1;
        } else if self.age > 0 && self.age < self.duration() {
            self.age += 1;
        } else {
            self.age = 0;
            return;
        }
        let end = self.rules.windup_ticks + self.rules.strike_ticks;
        let pitch = if self.age <= self.rules.windup_ticks {
            self.rules.windup_pitch
        } else {
            self.rules.strike_pitch
        };
        let strength = if self.age <= end {
            1.
        } else {
            f32::from(self.duration() - self.age) / f32::from(self.rules.recovery_ticks)
        };
        for (i, part) in [BodyPart::Head, BodyPart::Chest].into_iter().enumerate() {
            let index = definition
                .joints
                .iter()
                .position(|j| j.child == part)
                .expect("anatomical joint");
            let [min, max] = definition.joints[index].angular_limits[0];
            let target = Rotation::from_euler_angles(pitch[i].clamp(min, max), 0., 0.);
            targets[index] = targets[index].slerp(&target, strength);
            let motor = motors
                .joints
                .iter_mut()
                .find(|m| m.child == part)
                .expect("anatomical motor");
            motor.stiffness_nm_per_rad +=
                strength * (self.rules.stiffness[i] - motor.stiffness_nm_per_rad);
            motor.damping_nm_s_per_rad +=
                strength * (self.rules.damping[i] - motor.damping_nm_s_per_rad);
            motor.max_torque_nm += strength * (self.rules.torque_cap_nm[i] - motor.max_torque_nm);
        }
    }
}
