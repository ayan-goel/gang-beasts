use super::rig_frame::WriteRig;
use super::{BalanceConfig, BodyPart, LocomotionConfig, recovery_drive, support::foot_support};
use rapier3d::prelude::*;

const REST_TICKS: u8 = 18;
const RAMP_TICKS: f32 = 30.0;
const STANDING_TICKS: u8 = 12;
const ATTEMPT_TICKS: u16 = 240;
const RETRY_TICKS: u8 = 60;

#[derive(Default)]
pub(super) struct Recovery {
    age: u16,
    still_ticks: u8,
    upright_ticks: u8,
    cooldown: u8,
}

pub(super) struct RecoveryStep {
    pub strength: f32,
    pub targets: [[f32; 3]; 14],
    pub loads: [f32; 2],
}

impl Recovery {
    pub fn advance(
        &mut self,
        rig: &mut WriteRig<'_>,
        trunk_up: f32,
        feet: [bool; 2],
        support: &BalanceConfig,
        tuning: &LocomotionConfig,
    ) -> Option<RecoveryStep> {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return None;
        }
        if self.age == 0 && trunk_up > tuning.minimum_upright_y {
            self.still_ticks = 0;
            return None;
        }
        if self.age >= ATTEMPT_TICKS {
            // Rest between failed attempts under an obstruction. Time spent airborne
            // also counts against the deadline, so stale effort cannot survive a fall.
            *self = Self {
                cooldown: RETRY_TICKS,
                ..Self::default()
            };
            return None;
        }
        if self.age > 0 {
            self.age += 1;
        }
        let contact = BodyPart::ALL.into_iter().find_map(|part| {
            foot_support(
                rig.world,
                rig.parts.bodies[part.index()],
                &rig.parts.bodies,
                support,
            )
            .filter(|s| s.body.is_none_or(|h| !rig.world.bodies[h].is_dynamic()))
        });
        let Some(contact) = contact else {
            self.still_ticks = 0;
            self.upright_ticks = 0;
            return None;
        };
        let pelvis = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
        if self.age == 0 {
            // Wait for the fall to settle relative to the supporting surface.
            // Colliding with a wall or another suspended ragdoll supplies no bracing moment.
            let surface = contact.velocity
                + contact
                    .angular_velocity
                    .cross(&(pelvis.center_of_mass() - contact.contact_point));
            if (pelvis.linvel() - surface).norm() > 0.6
                || (pelvis.angvel() - contact.angular_velocity).norm() > 1.0
            {
                self.still_ticks = 0;
                return None;
            }
            self.still_ticks += 1;
            if self.still_ticks < REST_TICKS {
                return None;
            }
            self.age = 1;
        }
        self.upright_ticks = if feet == [true; 2] && trunk_up >= tuning.settle_minimum_up_y {
            self.upright_ticks + 1
        } else {
            0
        };
        if self.upright_ticks >= STANDING_TICKS {
            *self = Self::default();
            return None;
        }
        let strength = (f32::from(self.age) / RAMP_TICKS).min(1.0);
        let up = pelvis.rotation() * Vector::y();
        let facing = (pelvis.rotation() * -Vector::z()).y;
        let tuck = (1.0 - up.y).clamp(0.0, 1.0) * strength;
        let mut targets = [[0.0; 3]; 14];
        for (target, joint) in targets.iter_mut().zip(&rig.parts.definition.joints) {
            // Reverse the shoulder reach between prone and supine falls. A side fall
            // uses neutral shoulder pitch. Extend the knees as the pelvis rises.
            let pitch = match joint.child {
                BodyPart::LeftThigh | BodyPart::RightThigh => 0.65 * tuck,
                BodyPart::LeftShin | BodyPart::RightShin => -1.3 * tuck,
                BodyPart::LeftFoot | BodyPart::RightFoot => 0.45 * tuck,
                BodyPart::LeftUpperArm | BodyPart::RightUpperArm => -facing * tuck,
                BodyPart::LeftForearm | BodyPart::RightForearm => 0.7 * tuck,
                _ => 0.0,
            };
            target[0] = pitch.clamp(joint.angular_limits[0][0], joint.angular_limits[0][1]);
        }
        Some(RecoveryStep {
            strength,
            targets,
            loads: recovery_drive::apply(rig, strength, &tuning.recovery),
        })
    }
}
