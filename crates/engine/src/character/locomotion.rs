use super::walking_controller::{LocomotionState, WalkingController};
use super::{
    BalanceConfig, BodyPart, CharacterDefinition, CharacterError, LocomotionConfig, MotorConfig,
    PoseRagdoll,
};
use crate::{CommandError, PhysicsConfig};

#[cfg(test)]
#[path = "locomotion_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "moving_support_tests.rs"]
mod moving_support_tests;

#[cfg(test)]
#[path = "slope_tests.rs"]
mod slope_tests;

#[cfg(test)]
#[path = "stopping_support_tests.rs"]
mod stopping_support_tests;

#[cfg(test)]
#[path = "settling_tests.rs"]
mod settling_tests;

#[cfg(test)]
#[path = "settling_support_tests.rs"]
mod settling_support_tests;

#[cfg(test)]
#[path = "pose_transition_tests.rs"]
mod pose_transition_tests;

#[cfg(test)]
#[path = "rotating_traversal_tests.rs"]
mod rotating_traversal_tests;

pub struct LocomotionRagdoll {
    pose: PoseRagdoll,
    stair_fixture: bool,
    controller: WalkingController,
    idle_motors: MotorConfig,
    active_motors: MotorConfig,
}

impl LocomotionRagdoll {
    pub fn new(
        physics: PhysicsConfig,
        config: LocomotionConfig,
        spawn: [f32; 3],
    ) -> Result<Self, CharacterError> {
        config.validate()?;
        let mut pose = PoseRagdoll::new(
            physics,
            CharacterDefinition::bundled()?,
            MotorConfig::standing()?,
            spawn,
        )?;
        pose.configure_balance(Some(BalanceConfig::bundled()?))?;
        Ok(Self {
            pose,
            stair_fixture: false,
            controller: WalkingController::new(config),
            idle_motors: MotorConfig::standing()?,
            active_motors: MotorConfig::locomotion()?,
        })
    }
    pub fn stairs(
        physics: PhysicsConfig,
        config: LocomotionConfig,
    ) -> Result<Self, CharacterError> {
        let mut rig = Self::new(physics, config, [0.0, 0.08, 0.0])?;
        super::stair_fixture::install(&mut rig.pose.rig);
        rig.stair_fixture = true;
        Ok(rig)
    }

    pub fn is_stair_fixture(&self) -> bool {
        self.stair_fixture
    }

    pub fn pose(&self) -> &PoseRagdoll {
        &self.pose
    }
    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.pose.collider_lines()
    }
    pub fn pose_targets(&self) -> [[f32; 4]; 14] {
        self.state().targets.map(|[x, y, z]| {
            rapier3d::prelude::Rotation::from_euler_angles(x, y, z)
                .coords
                .into()
        })
    }
    pub fn state(&self) -> &LocomotionState {
        &self.controller.state
    }
    pub fn set_input(&mut self, direction: [f32; 2], running: bool) -> Result<(), CharacterError> {
        self.controller.set_input(direction, running)
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.pose.set_enabled(enabled);
        if !enabled {
            self.controller.reset();
            self.pose
                .blend_motors(&self.idle_motors, &self.active_motors, 0.0);
        }
    }
    pub fn step(&mut self) {
        // Queued foot departures must reach support queries before movement can assist.
        self.pose.rig.apply_pending_impulse();
        let enabled = self.pose.enabled();
        let targets = self.controller.advance(&mut self.pose.rig, enabled);
        self.pose.blend_motors(
            &self.idle_motors,
            &self.active_motors,
            self.controller.state.motor_strength,
        );
        self.pose.step_with_targets(&targets);
    }
    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        self.pose.queue_impulse(part, impulse)
    }
    pub fn reset(&mut self) {
        self.pose.reset();
        if self.stair_fixture {
            super::stair_fixture::install(&mut self.pose.rig);
        }
        self.controller.reset();
        self.pose
            .blend_motors(&self.idle_motors, &self.active_motors, 0.0);
    }
}
