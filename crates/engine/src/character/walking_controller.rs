#[cfg(test)]
use super::PassiveRagdoll;
use super::rig_frame::WriteRig;
use super::{
    BalanceConfig, BodyPart, CharacterError, LocomotionConfig, support::foot_support,
    support_motion::SupportMotion,
};
use crate::TICK_SECONDS;
use rapier3d::na::RealField;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "walking_controller_tests.rs"]
mod tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocomotionState {
    pub motor_strength: f32,
    pub acceleration: [f32; 3],
    pub support_velocity: [f32; 3],
    pub feet: [bool; 2],
    pub phase: f32,
    pub turn_torque: f32,
    pub stopping_moment: [f32; 3],
    pub recovery_loads: [f32; 2],
    pub recovery_strength: f32,
    pub support_moment: [f32; 3],
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
    swing_completion: super::swing_completion::SwingCompletion,
    pose_motion: super::pose_motion::PoseMotion,
    support_motion: SupportMotion,
    step_approach: super::step_approach::StepApproach,
    reversal: super::reversal::ReversalBrake,
    jump: super::jump::Jump,
    recovery: super::recovery::Recovery,
    stumbled: bool,
    sampled_motion: bool,
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
            swing_completion: Default::default(),
            pose_motion: Default::default(),
            support_motion: SupportMotion::default(),
            step_approach: Default::default(),
            reversal: Default::default(),
            jump: Default::default(),
            recovery: Default::default(),
            stumbled: false,
            sampled_motion: false,
            running: false,
            blend: 0.0,
            braking_ticks: 0,
            state: LocomotionState::default(),
        }
    }
    pub fn set_jump(&mut self, held: bool) {
        self.jump.input(held, &self.config.jump);
    }
    pub fn clear_jump_input(&mut self) {
        self.jump.clear_input();
    }
    pub fn reset(&mut self) {
        self.recovery = Default::default();
        self.jump = Default::default();
        self.step_approach = Default::default();
        self.intent = Vector::zeros();
        self.swing_completion.finish();
        self.pose_motion = Default::default();
        self.support_motion = SupportMotion::default();
        self.reversal = Default::default();
        self.sampled_motion = false;
        self.stumbled = false;
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
            if self.sampled_motion {
                self.swing_completion.arm(self.state.phase);
            }
        } else if vector.norm_squared() > 0.0 {
            self.swing_completion.finish();
            self.braking_ticks = 0;
        }
        let next = vector / vector.norm().max(1.0);
        self.intent = next;
        // Neutral UI events clear the run flag, but changing cadence mid-stride
        // perturbs a braking footfall. Keep cadence until a new movement request.
        if self.intent.norm_squared() > 0.0 {
            self.running = running;
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn advance(&mut self, rig: &mut PassiveRagdoll, enabled: bool) -> [Rotation<Real>; 14] {
        self.advance_frame(&mut rig.frame(), enabled)
    }
    pub fn advance_frame(&mut self, rig: &mut WriteRig<'_>, enabled: bool) -> [Rotation<Real>; 14] {
        self.sampled_motion = false;
        let idle = self.intent.norm_squared() == 0.0 && self.braking_ticks == 0;
        self.braking_ticks = self.braking_ticks.saturating_sub(1);
        let strength = if enabled {
            self.state.motor_strength
        } else {
            0.0
        };
        let phase = self.state.phase;
        let previous_targets = self.state.targets;
        self.state = LocomotionState {
            motor_strength: strength,
            phase,
            ..LocomotionState::default()
        };
        if !enabled {
            self.step_approach = Default::default();
            self.swing_completion.finish();
            self.pose_motion = Default::default();
            return [Rotation::identity(); 14];
        }
        let supports = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
            foot_support(
                rig.world,
                rig.parts.bodies[part.index()],
                &rig.parts.bodies,
                &self.support,
            )
        });
        self.state.feet = supports.map(|support| support.is_some());
        let count = supports.iter().flatten().count();
        let trunk_up = [BodyPart::Pelvis, BodyPart::Chest, BodyPart::Head]
            .map(|p| (rig.world.bodies[rig.parts.bodies[p.index()]].rotation() * Vector::y()).y)
            .into_iter()
            .fold(1.0_f32, f32::min);
        // A folded trunk can briefly cross the entry threshold during a fall.
        // Resume ground drive only once the whole trunk recovers its standing posture.
        if trunk_up <= self.config.minimum_upright_y {
            self.stumbled = true;
        } else if trunk_up >= self.config.settle_minimum_up_y {
            self.stumbled = false;
        }
        if let Some(step) =
            self.recovery
                .advance(rig, trunk_up, self.state.feet, &self.support, &self.config)
        {
            self.jump.takeoff(false, false, &self.config.jump);
            self.state.motor_strength = step.strength;
            self.state.recovery_strength = step.strength;
            self.state.recovery_loads = step.loads;
            self.state.targets = step.targets;
            return step
                .targets
                .map(|[x, y, z]| Rotation::from_euler_angles(x, y, z));
        }
        let upright = !self.stumbled;
        let launch = self.jump.takeoff(count > 0, upright, &self.config.jump);
        if launch {
            super::jump::apply_velocity_change(
                rig,
                Vector::y() * self.config.jump.takeoff_speed_m_s,
            );
            self.state.feet = [false; 2];
        } else if count == 0 && upright {
            let delta = self.jump.steer(
                self.intent,
                self.config.maximum_acceleration_m_s2 * self.config.jump.air_acceleration_fraction,
            );
            if delta != Vector::zeros() {
                super::jump::apply_velocity_change(rig, delta);
                self.state.acceleration = (delta / TICK_SECONDS).into();
            }
        }
        if launch || !upright {
            self.step_approach = Default::default();
        } else {
            self.step_approach.sample(
                rig.read(),
                self.intent,
                supports,
                &self.support,
                &self.config.low_step,
            );
        }
        if launch || count == 0 || !upright {
            self.swing_completion.finish();
            self.pose_motion = Default::default();
            self.support_motion = SupportMotion::default();
            self.reversal = Default::default();
            self.state.motor_strength =
                (strength - TICK_SECONDS / self.config.motor_release_s).max(0.0);
            self.blend = 0.0;
            if upright {
                // Losing support must stop world drive without snapping the legs
                // straight during landing. Only internal pose targets fade here.
                self.state.targets = super::pose_release::toward_bind(
                    previous_targets,
                    rig.parts.definition,
                    self.config.motor_release_s,
                );
                return self
                    .state
                    .targets
                    .map(|[x, y, z]| Rotation::from_euler_angles(x, y, z));
            }
            return [Rotation::identity(); 14];
        }
        let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
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
        let mut center = Vector::zeros();
        let mut mass = 0.0;
        for handle in rig.parts.bodies {
            let body = &rig.world.bodies[handle];
            velocity += body.linvel() * body.mass();
            center += body.center_of_mass().coords * body.mass();
            mass += body.mass();
        }
        velocity /= mass;
        center /= mass;
        // COM feedback needs the deck velocity at COM, not at a moving foot
        // witness. Otherwise rigid co-motion can produce persistent idle drive.
        let surface_at_com = supports
            .iter()
            .flatten()
            .map(|s| s.velocity + s.angular_velocity.cross(&(center - s.contact_point.coords)))
            .sum::<Vector<Real>>()
            / count as f32;
        let relative_spin = self.pose_motion.sample_supported(rig.read(), supports);
        if idle
            && (velocity - surface_at_com).norm() < self.config.settle_speed_m_s
            && relative_spin.norm() < self.config.settle_spin_rad_s
            && trunk_up > self.config.settle_minimum_up_y
        {
            self.state.motor_strength =
                (self.state.motor_strength - TICK_SECONDS / self.config.motor_release_s).max(0.0);
        }
        let relative_velocity = velocity - surface_at_com;
        let reversal_weight = self.reversal.advance(
            self.intent,
            relative_velocity,
            self.config.maximum_acceleration_m_s2 * TICK_SECONDS,
            self.config.braking_ticks,
        );
        let intent = self.intent * (1.0 - reversal_weight);
        let speed = if self.running {
            self.config.run_speed_m_s
        } else {
            self.config.walk_speed_m_s
        };
        let desired = intent * speed + surface_at_com;
        let mut acceleration = (desired - velocity) / self.config.velocity_response_s;
        acceleration.y = 0.0;
        // A short release brake gives way to weaker velocity damping. Neither
        // stage stores a desired position, so no force pulls the rig back to a point.
        if idle {
            acceleration = ((surface_at_com - velocity) / self.config.idle_response_s)
                .cap_magnitude(self.config.idle_maximum_acceleration_m_s2);
            acceleration.y = 0.0;
        }
        let carry = self.support_motion.advance(
            surface_at_com,
            velocity + acceleration * TICK_SECONDS,
            self.config.maximum_acceleration_m_s2 * TICK_SECONDS,
        );
        if idle {
            acceleration += carry / TICK_SECONDS;
        }
        acceleration = acceleration.cap_magnitude(self.config.maximum_acceleration_m_s2);
        let local = root.rotation().inverse() * intent;
        let forward = root.rotation() * -Vector::z();
        if intent.norm_squared() > 0.0 {
            // Rapier's enhanced-determinism feature selects libm through RealField.
            // Native f32 intrinsics can round differently from WASM and diverge.
            let angle = <f32 as RealField>::atan2(
                forward.z * intent.x - forward.x * intent.z,
                forward.x * intent.x + forward.z * intent.z,
            );
            self.state.turn_torque = (angle * self.config.turn_stiffness_nm_rad
                - root.angvel().y * self.config.turn_damping_nm_s_rad)
                .clamp(-self.config.turn_cap_nm, self.config.turn_cap_nm);
        }
        self.state.turn_torque *= 1.0 - reversal_weight;
        let direction = acceleration.try_normalize(0.001).unwrap_or(Vector::zeros());
        let lean = (root.rotation() * Vector::y()).dot(&direction);
        let lean_rate = root.angvel().dot(&Vector::y().cross(&direction));
        // Release and reversal braking counter foot drag while slowing the torso.
        // Ordinary steering retains its nonnegative foot allocation.
        let brake_weight = if !idle && self.intent.norm_squared() == 0.0 {
            1.0
        } else {
            reversal_weight
        };
        let horizontal = vector![relative_velocity.x, 0.0, relative_velocity.z];
        let motion_speed = horizontal.norm();
        let resolved =
            (motion_speed / (self.config.maximum_acceleration_m_s2 * TICK_SECONDS)).min(1.0);
        let motion = horizontal.try_normalize(0.000001).unwrap_or(direction);
        let friction_alignment = if brake_weight > 0.0 {
            1.0 + brake_weight * resolved * (motion.dot(&direction) - 1.0)
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
                * rig.parts.physics.gravity_y.abs()
                / acceleration.norm().max(0.001)
                + self.config.drive_tilt_gain * lean
                + self.config.drive_tilt_damping_s * lean_rate)
                .clamp(
                    if brake_weight > 0.0 {
                        -brake_weight
                    } else {
                        0.0
                    },
                    1.0,
                )
        };
        // Counteracting foot friction may require an opposing foot impulse.
        // Normalize both shares so their absolute sum keeps the original budget.
        let stop_drive = if self.intent.norm_squared() == 0.0 && self.state.motor_strength > 0.0 {
            Some(super::stopping_drive::allocate(
                rig.read(),
                acceleration,
                velocity - surface,
                supports,
                self.state.motor_strength,
                &self.config,
            ))
        } else {
            None
        };
        acceleration /= 1.0 - fraction + fraction.abs();
        if let Some(drive) = stop_drive {
            acceleration = drive.net;
            self.state.stopping_moment = drive.moment.into();
        }
        let base_foot = stop_drive.map_or(acceleration * fraction, |drive| drive.foot);
        let support_drive = super::support_drive::allocate(
            rig.read(),
            supports,
            velocity,
            acceleration,
            base_foot,
            self.intent.norm_squared() > 0.0,
            &self.config,
        );
        let drive = if let Some(support) = support_drive {
            acceleration = support.net;
            self.state.support_moment = support.moment.into();
            self.state.stopping_moment =
                (Vector::from(self.state.stopping_moment) * support.scale).into();
            Some((support.net, support.foot))
        } else {
            stop_drive.map(|drive| (drive.net, drive.foot))
        };
        self.state.acceleration = acceleration.into();
        // Tick-scoped assistance preserves the total linear impulse budget.
        for handle in rig.parts.bodies {
            let body = &mut rig.world.bodies[handle];
            let impulse = if let Some((net, foot)) = drive {
                (net - foot) * body.mass() * TICK_SECONDS
            } else {
                acceleration * body.mass() * TICK_SECONDS * (1.0 - fraction)
            };
            body.apply_impulse(impulse, true);
        }
        for (index, part) in [BodyPart::LeftFoot, BodyPart::RightFoot]
            .into_iter()
            .enumerate()
        {
            if supports[index].is_some() {
                let impulse = if let Some((_, foot)) = drive {
                    foot * mass * TICK_SECONDS / count as f32
                } else {
                    acceleration * mass * TICK_SECONDS * fraction / count as f32
                };
                rig.world.bodies[rig.parts.bodies[part.index()]].apply_impulse(impulse, true);
            }
        }
        rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]]
            .apply_torque_impulse(Vector::y() * self.state.turn_torque * TICK_SECONDS, true);
        self.blend += (intent.norm() - self.blend) * (TICK_SECONDS / self.config.gait_blend_s);
        let frequency = if self.running {
            self.config.run_frequency_hz
        } else {
            self.config.walk_frequency_hz
        };
        if self.blend > 0.001 {
            self.state.phase = (phase + std::f32::consts::TAU * frequency * TICK_SECONDS)
                .rem_euclid(std::f32::consts::TAU);
            self.sampled_motion = intent.norm_squared() > 0.0 && self.state.phase != phase;
        }
        self.state.targets = super::gait::targets(
            rig.parts.definition,
            &self.config,
            local,
            self.state.phase,
            self.blend,
        );
        super::terrain_gait::adapt(
            rig.read(),
            supports,
            &self.config,
            self.blend,
            &mut self.state.targets,
        );
        self.step_approach.pose(
            rig.read(),
            intent,
            &self.config.low_step,
            &mut self.state.targets,
        );
        if !idle && self.intent.norm_squared() == 0.0 && self.state.motor_strength > 0.0 {
            self.swing_completion.advance(
                rig.read(),
                supports,
                relative_velocity,
                &self.config,
                std::f32::consts::TAU * frequency * TICK_SECONDS,
                &mut self.state.targets,
            );
        } else {
            self.swing_completion.finish();
        }
        self.state
            .targets
            .map(|[x, y, z]| Rotation::from_euler_angles(x, y, z))
    }
}
