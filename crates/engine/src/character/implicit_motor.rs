use super::rig_frame::WriteRig;
use super::{JointMotorConfig, MotorConfig, PassiveRagdoll, motor::rotation_error};
use crate::TICK_SECONDS;
use rapier3d::prelude::*;

#[cfg(test)]
#[path = "implicit_motor_tests.rs"]
mod tests;

const AXES: [JointAxis; 3] = [JointAxis::AngX, JointAxis::AngY, JointAxis::AngZ];

pub(super) fn configure(
    rig: &mut WriteRig<'_>,
    motors: &MotorConfig,
    targets: &[Rotation<Real>; 14],
    enabled: bool,
) {
    // Grip and world constraints can share this joint set. Only anatomical handles
    // may receive pose targets or have their solver writeback cleared.
    for (index, handle) in rig.parts.joints.into_iter().enumerate() {
        let joint = rig
            .world
            .impulse_joints
            .get_mut(handle, false)
            .expect("owned anatomical joint");
        joint.data.motor_axes = JointAxesMask::empty();
        // A sleeping island may skip solving; never retain an earlier tick's effort.
        for axis in AXES {
            joint.data.motors[axis as usize].impulse = 0.0;
        }
        if !enabled {
            continue;
        }
        let definition = &rig.parts.definition.joints[index];
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
            parent.inverse()
                * (rig.world.bodies[joint.body2].angvel() - rig.world.bodies[joint.body1].angvel()),
            free,
            config,
        );
    }
}

pub(super) fn drive(
    rig: &mut WriteRig<'_>,
    motors: &MotorConfig,
    targets: &[Rotation<Real>; 14],
    balance: Option<&mut super::balance::BalanceController>,
    enabled: bool,
) {
    configure(rig, motors, targets, enabled);
    if enabled && let Some(balance) = balance {
        balance.sample(rig.read());
        let pelvis = rig.parts.bodies[super::BodyPart::Pelvis.index()];
        rig.world.bodies[pelvis]
            .apply_torque_impulse(Vector::from(balance.state.torque) * TICK_SECONDS, true);
    }
}

fn configure_joint(
    joint: &mut GenericJoint,
    current: Rotation<Real>,
    target: Rotation<Real>,
    angular_velocity: Vector<Real>,
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
    // Allocate predicted effort first, then reserve spare squared capacity equally.
    // The sum of squared axis ceilings stays within the joint's vector budget,
    // while a saturated pitch request can use capacity idle axes do not need.
    let demand = Vector::from(std::array::from_fn(|axis| {
        if free[axis] {
            config.stiffness_nm_per_rad * error[axis] - damping * angular_velocity[axis]
        } else {
            0.0
        }
    }))
    .cap_magnitude(config.max_torque_nm);
    let reserve = (config.max_torque_nm.powi(2) - demand.norm_squared()).max(0.0) / count as f32;
    for (index, axis) in AXES.into_iter().enumerate() {
        if free[index] {
            joint
                .set_motor_model(axis, MotorModel::ForceBased)
                .set_motor_velocity(axis, velocity[index], damping)
                .set_motor_max_force(axis, (demand[index].powi(2) + reserve).sqrt());
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
    for (index, handle) in rig.joints.into_iter().enumerate() {
        let joint = rig
            .world
            .impulse_joints
            .get(handle)
            .expect("owned anatomical joint");
        output[index] =
            last_substep_effort(&joint.data, rig.configuration().solver_iterations).into();
    }
    output
}

pub(super) fn targets(rig: &PassiveRagdoll) -> [[f32; 3]; 14] {
    let mut output = [[0.0; 3]; 14];
    for (index, handle) in rig.joints.into_iter().enumerate() {
        let joint = rig
            .world
            .impulse_joints
            .get(handle)
            .expect("owned anatomical joint");
        output[index] =
            AXES.map(|axis| joint.data.motor(axis).map_or(0.0, |motor| motor.target_vel));
    }
    output
}

pub(super) fn torques(rig: &PassiveRagdoll) -> [[f32; 3]; 14] {
    let mut output = local_efforts(rig);
    for (index, handle) in rig.joints.into_iter().enumerate() {
        let joint = rig
            .world
            .impulse_joints
            .get(handle)
            .expect("owned anatomical joint");
        // Display local solver effort on the completed frame. Rapier's solve-time
        // frame precedes integration, so this is not a measured world impulse vector.
        let frame = rig.world.bodies[joint.body1].rotation() * joint.data.local_frame1.rotation;
        output[index] = (frame * Vector::from(output[index])).into();
    }
    output
}
