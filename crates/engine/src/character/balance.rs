use super::rig_frame::ReadRig;
use super::{
    BalanceConfig, BodyPart, JointMotorConfig, motor::local_torque, support::foot_supported,
};
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "balance_tests.rs"]
mod tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BalanceState {
    pub feet: [bool; 2],
    pub torque: [f32; 3],
}

pub(super) struct BalanceController {
    pub config: BalanceConfig,
    pub state: BalanceState,
}

impl BalanceController {
    pub fn sample(&mut self, rig: ReadRig<'_>) {
        let feet = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
            foot_supported(
                rig.world,
                rig.parts.bodies[part.index()],
                &rig.parts.bodies,
                &self.config,
            )
        });
        self.state = BalanceState {
            feet,
            torque: [0.0; 3],
        };
        let support = feet.into_iter().filter(|foot| *foot).count() as f32 / 2.0;
        if support == 0.0 {
            return;
        }
        let root = &rig.world.bodies[rig.parts.bodies[BodyPart::Pelvis.index()]];
        let up = root.rotation() * Vector::y();
        let correction = Rotation::rotation_between(&up, &Vector::y()).unwrap_or_else(|| {
            // An inverted up vector has no unique cross-product axis. Choose the
            // body's right axis deterministically; the torque cap still limits recovery.
            Rotation::from_axis_angle(
                &nalgebra::Unit::new_normalize(root.rotation() * Vector::x()),
                std::f32::consts::PI,
            )
        });
        let gains = JointMotorConfig {
            child: BodyPart::Pelvis,
            stiffness_nm_per_rad: self.config.stiffness_nm_per_rad,
            damping_nm_s_per_rad: self.config.damping_nm_s_per_rad,
            max_torque_nm: self.config.max_torque_nm,
        };
        // Only tilt is assisted. Yaw remains free, and losing either foot halves
        // effort; losing both removes this world-referenced actuator entirely.
        self.state.torque = (local_torque(
            *root.rotation(),
            correction * root.rotation(),
            *root.angvel(),
            [true, false, true],
            &gains,
        ) * support)
            .into();
    }
}
