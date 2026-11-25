use super::{
    CharacterError, MotorConfig, arm_pose, rig_frame::WriteRig,
    walking_controller::WalkingController,
};
use crate::combat::Hands;
use rapier3d::prelude::*;

pub(super) struct MotorPresets {
    idle: MotorConfig,
    active: MotorConfig,
}

impl MotorPresets {
    pub fn bundled() -> Result<Self, CharacterError> {
        Ok(Self {
            idle: MotorConfig::standing()?,
            active: MotorConfig::locomotion()?,
        })
    }
    pub fn blend(&self, motors: &mut MotorConfig, strength: f32) {
        motors.blend_from(&self.idle, &self.active, strength);
    }
    pub fn prepare(
        &self,
        rig: &mut WriteRig<'_>,
        motors: &mut MotorConfig,
        walking: &mut WalkingController,
        hands: &mut Hands,
        enabled: bool,
    ) -> [Rotation<Real>; 14] {
        let mut targets = walking.advance_frame(rig, enabled);
        self.blend(motors, walking.state.motor_strength);
        if enabled && walking.state.recovery_strength == 0.0 {
            arm_pose::apply(
                hands,
                rig.read(),
                motors,
                walking.state.feet,
                &mut walking.state.targets,
                &mut targets,
            );
        } else {
            hands.clear();
        }
        targets
    }
}
