use super::{JointMotorConfig, MotorConfig, PassiveRagdoll, motor::rotation_error};
use crate::TICK_SECONDS;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "implicit_motor_tests.rs"]
mod tests;

const AXES: [JointAxis; 3] = [JointAxis::AngX, JointAxis::AngY, JointAxis::AngZ];

pub(super) fn configure(
    rig: &mut PassiveRagdoll,
    motors: &MotorConfig,
    targets: &[Rotation<Real>; 14],
    enabled: bool,
) {
    for (index, (_, joint)) in rig.world.impulse_joints.iter_mut().enumerate() {
        joint.data.motor_axes = JointAxesMask::empty();
        // A sleeping island may skip solving; never retain an earlier tick's effort.
        for axis in AXES {
            joint.data.motors[axis as usize].impulse = 0.0;
        }
        if !enabled {
            continue;
        }
        let definition = &rig.definition.joints[index];
        let config = motors
            .joints
            .iter()
            .find(|motor| motor.child == definition.child)
            .unwrap();
        let free = definition.angular_limits.map(|limits| limits != [0.0, 0.0]);
        let parent = rig.world.bodies[joint.body1].rotation() * joint.data.local_frame1.rotation;
        let child = rig.world.bodies[joint.body2].rotation() * joint.data.local_frame2.rotation;
        configure_joint(
            &mut joint.data,
            parent.inverse() * child,
            targets[index],
            free,
            config,
        );
    }
}

fn configure_joint(
    joint: &mut GenericJoint,
    current: Rotation<Real>,
    target: Rotation<Real>,
    free: [bool; 3],
    config: &JointMotorConfig,
) {
    let count = free.into_iter().filter(|axis| *axis).count();
    let damping = config.damping_nm_s_per_rad + config.stiffness_nm_per_rad * TICK_SECONDS;
    if count == 0 || damping == 0.0 || config.max_torque_nm == 0.0 {
        return;
    }
    let error = rotation_error(current, target);
    // The solver evaluates damping against its next velocity. Adding Kp*dt
    // gives a first-order implicit position term without quaternion component subtraction.
    let velocity = error * (config.stiffness_nm_per_rad / damping);
    // Independent axis caps conservatively preserve the whole-vector budget.
    // A single-axis request cannot borrow unused effort from the other free axes.
    let cap = config.max_torque_nm / (count as f32).sqrt();
    for (index, axis) in AXES.into_iter().enumerate() {
        if free[index] {
            joint
                .set_motor_model(axis, MotorModel::ForceBased)
                .set_motor_velocity(axis, velocity[index], damping)
                .set_motor_max_force(axis, cap);
        }
    }
}

fn last_substep_effort(joint: &GenericJoint, iterations: usize) -> Vector<Real> {
    // Rapier 0.26 writes the final substep's parent-side impulse. Negate for
    // child effort and divide by substep dt, not the full simulation tick.
    let dt = TICK_SECONDS / iterations as f32;
    Vector::from(AXES.map(|axis| joint.motor(axis).map_or(0.0, |motor| -motor.impulse / dt)))
}

pub(super) fn local_efforts(rig: &PassiveRagdoll) -> [[f32; 3]; 14] {
    let mut output = [[0.0; 3]; 14];
    for (index, (_, joint)) in rig.world.impulse_joints.iter().enumerate() {
        output[index] =
            last_substep_effort(&joint.data, rig.configuration().solver_iterations).into();
    }
    output
}

pub(super) fn targets(rig: &PassiveRagdoll) -> [[f32; 3]; 14] {
    let mut output = [[0.0; 3]; 14];
    for (index, (_, joint)) in rig.world.impulse_joints.iter().enumerate() {
        output[index] =
            AXES.map(|axis| joint.data.motor(axis).map_or(0.0, |motor| motor.target_vel));
    }
    output
}

pub(super) fn torques(rig: &PassiveRagdoll) -> [[f32; 3]; 14] {
    let mut output = local_efforts(rig);
    for (index, (_, joint)) in rig.world.impulse_joints.iter().enumerate() {
        // Display local solver effort on the completed frame. Rapier's solve-time
        // frame precedes integration, so this is not a measured world impulse vector.
        let frame = rig.world.bodies[joint.body1].rotation() * joint.data.local_frame1.rotation;
        output[index] = (frame * Vector::from(output[index])).into();
    }
    output
}
