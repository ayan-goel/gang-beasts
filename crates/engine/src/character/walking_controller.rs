use super::{
    BalanceConfig, BodyPart, CharacterError, LocomotionConfig, PassiveRagdoll,
    support::foot_support, support_motion::SupportMotion,
};
use crate::TICK_SECONDS;
use rapier3d::na::RealField;
use rapier3d::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocomotionState {
    pub motor_strength: f32,
    pub acceleration: [f32; 3],
    pub support_velocity: [f32; 3],
    pub feet: [bool; 2],
    pub phase: f32,
    pub turn_torque: f32,
    pub targets: [[f32; 3]; 14],
}

impl LocomotionState {
    pub fn to_buffer(&self) -> [f32; 11] {
        [
            self.acceleration[0],
            self.acceleration[1],
            self.acceleration[2],
            self.support_velocity[0],
            self.support_velocity[1],
            self.support_velocity[2],
            u8::from(self.feet[0]) as f32,
            u8::from(self.feet[1]) as f32,
            self.phase,
            self.turn_torque,
            self.motor_strength,
        ]
    }
}

pub(super) struct WalkingController {
    config: LocomotionConfig,
    support: BalanceConfig,
    intent: Vector<Real>,
    support_motion: SupportMotion,
    running: bool,
    blend: f32,
    braking_ticks: u32,
    pub state: LocomotionState,
}

impl WalkingController {
    pub fn new(config: LocomotionConfig) -> Self {
        Self {
            config,
            support: BalanceConfig::bundled().expect("bundled support config"),
            intent: Vector::zeros(),
            support_motion: SupportMotion::default(),
            running: false,
            blend: 0.0,
            braking_ticks: 0,
            state: LocomotionState::default(),
        }
    }
    pub fn reset(&mut self) {
        self.intent = Vector::zeros();
        self.support_motion = SupportMotion::default();
        self.running = false;
        self.blend = 0.0;
        self.braking_ticks = 0;
        self.state = LocomotionState::default();
    }
    pub fn set_input(&mut self, direction: [f32; 2], running: bool) -> Result<(), CharacterError> {
        if !direction.iter().all(|value| value.is_finite()) {
            return Err(CharacterError::Invalid("movement intent must be finite"));
        }
        // Scale before normalization so even finite maximum-f32 input cannot overflow.
        let scale = direction[0].abs().max(direction[1].abs()).max(1.0);
        let vector = vector![direction[0] / scale, 0.0, direction[1] / scale];
        if vector.norm_squared() == 0.0 && self.intent.norm_squared() > 0.0 {
            self.braking_ticks = self.config.braking_ticks;
        } else if vector.norm_squared() > 0.0 {
            self.braking_ticks = 0;
        }
        self.intent = vector / vector.norm().max(1.0);
        self.running = running;
        Ok(())
    }
    pub fn advance(&mut self, rig: &mut PassiveRagdoll, enabled: bool) -> [Rotation<Real>; 14] {
        let idle = self.intent.norm_squared() == 0.0 && self.braking_ticks == 0;
        self.braking_ticks = self.braking_ticks.saturating_sub(1);
        let strength = if enabled {
            self.state.motor_strength
        } else {
            0.0
        };
        let phase = self.state.phase;
        self.state = LocomotionState {
            motor_strength: strength,
            phase,
            ..LocomotionState::default()
        };
        if !enabled {
            return [Rotation::identity(); 14];
        }
        let supports = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
            foot_support(
                &rig.world,
                rig.bodies[part.index()],
                &rig.bodies,
                &self.support,
            )
        });
        self.state.feet = supports.map(|support| support.is_some());
        let count = supports.iter().flatten().count();
        let root = &rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]];
        let upright = (root.rotation() * Vector::y()).y > self.config.minimum_upright_y;
        if count == 0 || !upright {
            self.support_motion = SupportMotion::default();
            self.blend = 0.0;
            self.state.motor_strength =
                (strength - TICK_SECONDS / self.config.motor_release_s).max(0.0);
            return [Rotation::identity(); 14];
        }
        if !idle {
            self.state.motor_strength = 1.0;
        }
        let surface = supports
            .iter()
            .flatten()
            .fold(Vector::zeros(), |sum, support| sum + support.velocity)
            / count as f32;
        self.state.support_velocity = surface.into();
        let mut velocity = Vector::zeros();
        let mut mass = 0.0;
        for handle in rig.bodies {
            let body = &rig.world.bodies[handle];
            velocity += body.linvel() * body.mass();
            mass += body.mass();
        }
        velocity /= mass;
        if idle
            && (velocity - surface).norm() < self.config.settle_speed_m_s
            && root.angvel().norm() < self.config.settle_spin_rad_s
            && (root.rotation() * Vector::y()).y > self.config.settle_minimum_up_y
        {
            self.state.motor_strength =
                (self.state.motor_strength - TICK_SECONDS / self.config.motor_release_s).max(0.0);
        }
        let speed = if self.running {
            self.config.run_speed_m_s
        } else {
            self.config.walk_speed_m_s
        };
        let desired = self.intent * speed + surface;
        let mut acceleration = (desired - velocity) / self.config.velocity_response_s;
        acceleration.y = 0.0;
        // A short release brake gives way to weaker velocity damping. Neither
        // stage stores a desired position, so no force pulls the rig back to a point.
        if idle {
            acceleration = ((surface - velocity) / self.config.idle_response_s)
                .cap_magnitude(self.config.idle_maximum_acceleration_m_s2);
            acceleration.y = 0.0;
        }
        let carry = self.support_motion.advance(
            surface,
            velocity + acceleration * TICK_SECONDS,
            self.config.maximum_acceleration_m_s2 * TICK_SECONDS,
        );
        if idle {
            acceleration += carry / TICK_SECONDS;
        }
        acceleration = acceleration.cap_magnitude(self.config.maximum_acceleration_m_s2);
        let local = root.rotation().inverse() * self.intent;
        let forward = root.rotation() * -Vector::z();
        if self.intent.norm_squared() > 0.0 {
            // Rapier's enhanced-determinism feature selects libm through RealField.
            // Native f32 intrinsics can round differently from WASM and diverge.
            let angle = <f32 as RealField>::atan2(
                forward.z * self.intent.x - forward.x * self.intent.z,
                forward.x * self.intent.x + forward.z * self.intent.z,
            );
            self.state.turn_torque = (angle * self.config.turn_stiffness_nm_rad
                - root.angvel().y * self.config.turn_damping_nm_s_rad)
                .clamp(-self.config.turn_cap_nm, self.config.turn_cap_nm);
        }
        let direction = acceleration.try_normalize(0.001).unwrap_or(Vector::zeros());
        let lean = (root.rotation() * Vector::y()).dot(&direction);
        let lean_rate = root.angvel().dot(&Vector::y().cross(&direction));
        // Release braking can oppose foot drag while slowing the torso. Active
        // steering retains its existing allocation; direction changes need gait control.
        let braking = !idle && self.intent.norm_squared() == 0.0;
        let relative = velocity - surface;
        let horizontal = vector![relative.x, 0.0, relative.z];
        let motion_speed = horizontal.norm();
        let resolved =
            (motion_speed / (self.config.maximum_acceleration_m_s2 * TICK_SECONDS)).min(1.0);
        let motion = horizontal.try_normalize(0.000001).unwrap_or(direction);
        let friction_alignment = if braking {
            1.0 + resolved * (motion.dot(&direction) - 1.0)
        } else {
            1.0
        };
        let fraction = if idle {
            0.0
        } else {
            ((supports
                .iter()
                .flatten()
                .map(|support| {
                    support.friction * friction_alignment
                        - support.normal.dot(&direction) / support.normal.y
                })
                .sum::<f32>()
                / count as f32)
                * rig.configuration().gravity_y.abs()
                / acceleration.norm().max(0.001)
                + self.config.drive_tilt_gain * lean
                + self.config.drive_tilt_damping_s * lean_rate)
                .clamp(if braking { -1.0 } else { 0.0 }, 1.0)
        };
        // Counteracting foot friction may require an opposing foot impulse.
        // Normalize both shares so their absolute sum keeps the original budget.
        acceleration /= 1.0 - fraction + fraction.abs();
        self.state.acceleration = acceleration.into();
        // Tick-scoped assistance preserves the total linear impulse budget.
        for handle in rig.bodies {
            let body = &mut rig.world.bodies[handle];
            body.apply_impulse(
                acceleration * body.mass() * TICK_SECONDS * (1.0 - fraction),
                true,
            );
        }
        for (index, part) in [BodyPart::LeftFoot, BodyPart::RightFoot]
            .into_iter()
            .enumerate()
        {
            if supports[index].is_some() {
                rig.world.bodies[rig.bodies[part.index()]].apply_impulse(
                    acceleration * mass * TICK_SECONDS * fraction / count as f32,
                    true,
                );
            }
        }
        rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]]
            .apply_torque_impulse(Vector::y() * self.state.turn_torque * TICK_SECONDS, true);
        self.blend += (self.intent.norm() - self.blend) * (TICK_SECONDS / self.config.gait_blend_s);
        let frequency = if self.running {
            self.config.run_frequency_hz
        } else {
            self.config.walk_frequency_hz
        };
        if self.blend > 0.001 {
            self.state.phase = (phase + std::f32::consts::TAU * frequency * TICK_SECONDS)
                .rem_euclid(std::f32::consts::TAU);
        }
        self.state.targets = super::gait::targets(
            &rig.definition,
            &self.config,
            local,
            self.state.phase,
            self.blend,
        );
        super::terrain_gait::adapt(
            rig,
            supports,
            &self.config,
            self.blend,
            &mut self.state.targets,
        );
        self.state
            .targets
            .map(|[x, y, z]| Rotation::from_euler_angles(x, y, z))
    }
}
