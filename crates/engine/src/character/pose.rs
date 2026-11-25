use super::{
    BalanceConfig, BalanceState, BodyPart, CharacterDefinition, CharacterError, MotorConfig,
    PassiveRagdoll, balance::BalanceController, motor::local_torque,
};
use crate::{CommandError, PhysicsConfig, TICK_SECONDS};
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "pose_diagnostics_tests.rs"]
mod diagnostics_tests;

pub struct PoseRagdoll {
    rig: PassiveRagdoll,
    motors: MotorConfig,
    enabled: bool,
    torques: [[f32; 3]; 14],
    balance: Option<BalanceController>,
}

impl PoseRagdoll {
    pub fn new(
        physics: PhysicsConfig,
        definition: CharacterDefinition,
        motors: MotorConfig,
        spawn: [f32; 3],
    ) -> Result<Self, CharacterError> {
        motors.validate()?;
        Ok(Self {
            rig: PassiveRagdoll::new(physics, definition, spawn)?,
            motors,
            enabled: true,
            torques: [[0.0; 3]; 14],
            balance: None,
        })
    }

    pub fn rig(&self) -> &PassiveRagdoll {
        &self.rig
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn motor_configuration(&self) -> &MotorConfig {
        &self.motors
    }
    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.rig.collider_lines()
    }
    /// World-aligned effort display: explicit tick commands, or implicit local
    /// solver effort rotated onto completed joint frames (not solve-time world torque).
    pub fn motor_torques(&self) -> &[[f32; 3]; 14] {
        &self.torques
    }

    /// Child-side N·m components along the final substep's parent-joint solver axes.
    pub fn solver_motor_efforts(&self) -> Option<[[f32; 3]; 14]> {
        (self.motors.model == "implicit_velocity_pd").then(|| {
            if self.enabled {
                super::implicit_motor::local_efforts(&self.rig)
            } else {
                [[0.0; 3]; 14]
            }
        })
    }

    pub fn solver_motor_targets(&self) -> Option<[[f32; 3]; 14]> {
        (self.motors.model == "implicit_velocity_pd").then(|| {
            if self.enabled {
                super::implicit_motor::targets(&self.rig)
            } else {
                [[0.0; 3]; 14]
            }
        })
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled == self.enabled {
            return;
        }
        if enabled && !self.enabled {
            // GenericJoint motor changes do not wake Rapier islands. A sleeping
            // fallen rig must respond to re-enabling without an external push.
            for (_, body) in self.rig.world.bodies.iter_mut() {
                body.wake_up(true);
            }
        }
        self.enabled = enabled;
        // Paused disable/re-enable must not revive last tick's solver writeback.
        for (_, joint) in self.rig.world.impulse_joints.iter_mut() {
            for motor in &mut joint.data.motors {
                motor.impulse = 0.0;
                motor.target_vel = 0.0;
            }
        }
        self.torques = [[0.0; 3]; 14];
        if let Some(balance) = &mut self.balance {
            balance.state = BalanceState::default();
        }
    }

    pub fn configure_balance(
        &mut self,
        config: Option<BalanceConfig>,
    ) -> Result<(), CharacterError> {
        if let Some(config) = &config {
            config.validate()?;
        }
        self.balance = config.map(|config| BalanceController {
            config,
            state: BalanceState::default(),
        });
        Ok(())
    }

    pub fn balance_state(&self) -> Option<&BalanceState> {
        self.balance.as_ref().map(|balance| &balance.state)
    }

    pub fn step(&mut self) {
        // Support must see queued departures on this tick, before balance decides effort.
        self.rig.apply_pending_impulse();
        self.apply_motors();
        self.rig.step();
        if self.enabled && self.motors.model == "implicit_velocity_pd" {
            self.torques = super::implicit_motor::torques(&self.rig);
        }
    }

    fn apply_motors(&mut self) {
        self.torques = [[0.0; 3]; 14];
        super::implicit_motor::configure(
            &mut self.rig,
            &self.motors,
            self.enabled && self.motors.model == "implicit_velocity_pd",
        );
        if self.enabled {
            if let Some(balance) = &mut self.balance {
                balance.sample(&self.rig);
            }
            let world = &mut self.rig.world;
            if self.motors.model == "implicit_velocity_pd" {
                if let Some(balance) = &self.balance {
                    world.bodies[self.rig.bodies[BodyPart::Pelvis.index()]].apply_torque_impulse(
                        Vector::from(balance.state.torque) * TICK_SECONDS,
                        true,
                    );
                }
                return;
            }
            // Compute every actuator from the same pre-actuation state. Applying
            // while evaluating would make shared-body damping depend on joint order.
            for (index, (_, joint)) in world.impulse_joints.iter().enumerate() {
                let parent = &world.bodies[joint.body1];
                let child = &world.bodies[joint.body2];
                let parent_frame = parent.rotation() * joint.data.local_frame1.rotation;
                let child_frame = child.rotation() * joint.data.local_frame2.rotation;
                let definition = &self.rig.definition.joints[index];
                let config = self
                    .motors
                    .joints
                    .iter()
                    .find(|config| config.child == definition.child)
                    .expect("validated motor topology");
                let torque = local_torque(
                    parent_frame.inverse() * child_frame,
                    Rotation::identity(),
                    parent_frame.inverse() * (child.angvel() - parent.angvel()),
                    definition.angular_limits.map(|limits| limits != [0.0, 0.0]),
                    config,
                );
                self.torques[index] = (parent_frame * torque).into();
            }
            for ((_, joint), torque) in world.impulse_joints.iter().zip(self.torques) {
                let impulse = Vector::from(torque) * TICK_SECONDS;
                // INVARIANT: internal actuation applies equal/opposite angular impulses.
                // Tick-scoped impulses also leave no persistent force after disabling control.
                world.bodies[joint.body1].apply_torque_impulse(-impulse, true);
                world.bodies[joint.body2].apply_torque_impulse(impulse, true);
            }
            if let Some(balance) = &self.balance {
                world.bodies[self.rig.bodies[BodyPart::Pelvis.index()]]
                    .apply_torque_impulse(Vector::from(balance.state.torque) * TICK_SECONDS, true);
            }
        }
    }

    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        self.rig.queue_impulse(part, impulse)
    }

    pub fn reset(&mut self) {
        self.rig.reset();
        self.torques = [[0.0; 3]; 14];
        if let Some(balance) = &mut self.balance {
            balance.state = BalanceState::default();
        }
    }
}
