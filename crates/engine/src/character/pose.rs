use super::{
    BodyPart, CharacterDefinition, CharacterError, MotorConfig, PassiveRagdoll, motor::local_torque,
};
use crate::{CommandError, PhysicsConfig, TICK_SECONDS};
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;

pub struct PoseRagdoll {
    rig: PassiveRagdoll,
    motors: MotorConfig,
    enabled: bool,
    torques: [[f32; 3]; 14],
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
        })
    }

    pub fn rig(&self) -> &PassiveRagdoll {
        &self.rig
    }
    pub fn motor_torques(&self) -> &[[f32; 3]; 14] {
        &self.torques
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.torques = [[0.0; 3]; 14];
    }

    pub fn step(&mut self) {
        self.apply_motors();
        self.rig.step();
    }

    fn apply_motors(&mut self) {
        self.torques = [[0.0; 3]; 14];
        if self.enabled {
            let world = &mut self.rig.world;
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
        }
    }

    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        self.rig.queue_impulse(part, impulse)
    }

    pub fn reset(&mut self) {
        self.rig.reset();
        self.torques = [[0.0; 3]; 14];
    }
}
