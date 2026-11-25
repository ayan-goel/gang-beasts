use super::*;
use crate::{PhysicsConfig, physics::PhysicsWorld};

fn pair(
    iterations: usize,
    gains: &super::super::JointMotorConfig,
) -> (
    PhysicsWorld,
    RigidBodyHandle,
    RigidBodyHandle,
    ImpulseJointHandle,
) {
    let mut world = PhysicsWorld::new(&PhysicsConfig {
        gravity_y: 0.0,
        solver_iterations: iterations,
        ..PhysicsConfig::default()
    });
    let parent = world.bodies.insert(RigidBodyBuilder::dynamic());
    let child = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().rotation(vector![0.2, 0.0, 0.0]));
    for body in [parent, child] {
        world.colliders.insert_with_parent(
            ColliderBuilder::ball(0.2).mass(1.0),
            body,
            &mut world.bodies,
        );
    }
    let mut joint = GenericJointBuilder::new(JointAxesMask::LOCKED_SPHERICAL_AXES)
        .contacts_enabled(false)
        .build();
    configure_joint(
        &mut joint,
        Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]),
        Rotation::identity(),
        Vector::zeros(),
        [true; 3],
        gains,
    );
    let handle = world.impulse_joints.insert(parent, child, joint, true);
    (world, parent, child, handle)
}

#[test]
fn solver_pd_matches_an_independent_one_axis_implicit_solution() {
    let gains = super::super::JointMotorConfig {
        child: super::super::BodyPart::Head,
        stiffness_nm_per_rad: 6.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    let (mut world, parent, child, joint) = pair(1, &gains);
    world.step();
    // Each unit-mass sphere has I = 2/5 * 0.2² = 0.016 kg·m².
    // Solve τ = -1.2 - 0.4*(2/I)*τ*dt independently of the motor implementation.
    let expected_torque = -1.2 / (1.0 + 0.4 * 125.0 / 60.0);
    let expected_momentum = expected_torque / 60.0;
    assert!((world.bodies[child].angvel().x * 0.016 - expected_momentum).abs() < 1e-6);
    assert!((world.bodies[parent].angvel().x * 0.016 + expected_momentum).abs() < 1e-6);
    let actual = last_substep_effort(&world.impulse_joints.get(joint).unwrap().data, 1);
    assert!((actual.x - expected_torque).abs() < 1e-5);
    assert!(
        world.bodies[parent]
            .rotation()
            .angle_to(world.bodies[child].rotation())
            < 0.2
    );
}

#[test]
fn saturated_effort_and_diagnostics_use_substep_duration() {
    let gains = super::super::JointMotorConfig {
        child: super::super::BodyPart::Head,
        stiffness_nm_per_rad: 200.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    let cap = 3.0;
    for iterations in [1, 8, 16] {
        let (mut world, parent, child, joint) = pair(iterations, &gains);
        world.step();
        let actual =
            last_substep_effort(&world.impulse_joints.get(joint).unwrap().data, iterations);
        assert!(
            (actual.x + cap).abs() < 1e-5,
            "iterations {iterations}, torque {actual:?}"
        );
        assert!((world.bodies[child].angvel().x * 0.016 + cap / 60.0).abs() < 1e-6);
        assert!((world.bodies[parent].angvel().x + world.bodies[child].angvel().x).abs() < 1e-6);
    }
}

#[test]
fn a_nonzero_target_reverses_the_physical_correction_toward_that_pose() {
    let gains = super::super::JointMotorConfig {
        child: super::super::BodyPart::Head,
        stiffness_nm_per_rad: 6.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    let (mut world, parent, child, joint) = pair(1, &gains);
    configure_joint(
        &mut world.impulse_joints.get_mut(joint, true).unwrap().data,
        Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]),
        Rotation::from_scaled_axis(vector![0.4, 0.0, 0.0]),
        Vector::zeros(),
        [true; 3],
        &gains,
    );
    world.step();
    let expected = 1.2 / (1.0 + 0.4 * 125.0 / 60.0);
    assert!((world.bodies[child].angvel().x * 0.016 - expected / 60.0).abs() < 1e-6);
    assert!((world.bodies[parent].angvel().x * 0.016 + expected / 60.0).abs() < 1e-6);
    assert!(
        world.bodies[parent]
            .rotation()
            .angle_to(world.bodies[child].rotation())
            > 0.2
    );
}

#[test]
fn isolated_axis_load_can_use_the_joint_budget_without_lateral_effort() {
    let gains = JointMotorConfig {
        child: super::super::BodyPart::Chest,
        stiffness_nm_per_rad: 200.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    for axis in [Vector::x(), Vector::y(), Vector::z()] {
        for (angle, speed) in [(0.2, 0.0), (0.0, 30.0)] {
            let (mut world, _, child, joint) = pair(8, &gains);
            let rotation = Rotation::from_scaled_axis(axis * angle);
            world.bodies[child].set_rotation(rotation, true);
            world.bodies[child].set_angvel(axis * speed, true);
            configure_joint(
                &mut world.impulse_joints.get_mut(joint, true).unwrap().data,
                rotation,
                Rotation::identity(),
                axis * speed,
                [true; 3],
                &gains,
            );
            world.step();
            let effort = last_substep_effort(&world.impulse_joints.get(joint).unwrap().data, 8);
            assert!(
                (effort + axis * 3.0).norm() < 0.00001,
                "unused axis budget: {effort:?}"
            );
            assert!(effort.norm() <= 3.00001);
        }
    }
}

#[test]
fn simultaneous_axis_loads_remain_within_one_vector_effort_budget() {
    let gains = JointMotorConfig {
        child: super::super::BodyPart::Chest,
        stiffness_nm_per_rad: 200.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    for free in [[true; 3], [true, false, true], [false, true, false]] {
        for spin in [Vector::zeros(), vector![4.0, -3.0, 2.0]] {
            for iterations in [1, 8, 16] {
                let (mut world, parent, child, joint) = pair(iterations, &gains);
                let rotation = Rotation::from_euler_angles(0.2, -0.15, 0.1);
                world.bodies[child].set_rotation(rotation, true);
                world.bodies[child].set_angvel(spin, true);
                let data = &mut world.impulse_joints.get_mut(joint, true).unwrap().data;
                data.motor_axes = JointAxesMask::empty();
                configure_joint(data, rotation, Rotation::identity(), spin, free, &gains);
                let ceiling_norm = AXES
                    .iter()
                    .map(|axis| {
                        data.motor(*axis)
                            .map_or(0.0, |motor| motor.max_force.powi(2))
                    })
                    .sum::<f32>()
                    .sqrt();
                assert!(ceiling_norm <= gains.max_torque_nm + 0.00001);
                world.step();
                let effort =
                    last_substep_effort(&world.impulse_joints.get(joint).unwrap().data, iterations);
                assert!(effort.norm() <= 3.00001, "effort {effort:?}");
                for axis in 0..3 {
                    if !free[axis] {
                        assert_eq!(effort[axis], 0.0);
                    }
                }
                // Equal spherical inertias make total angular momentum observable
                // without confusing local solver axes with world torque diagnostics.
                assert!(
                    (world.bodies[parent].angvel() + world.bodies[child].angvel() - spin).norm()
                        < 0.0001
                );
            }
        }
    }
}

#[test]
fn zero_predicted_demand_keeps_damping_capacity_for_new_impacts() {
    let gains = JointMotorConfig {
        child: super::super::BodyPart::Chest,
        stiffness_nm_per_rad: 200.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: 3.0,
    };
    for axis in [Vector::x(), Vector::y(), Vector::z()] {
        let (mut world, _, child, joint) = pair(8, &gains);
        world.bodies[child].set_rotation(Rotation::identity(), true);
        configure_joint(
            &mut world.impulse_joints.get_mut(joint, true).unwrap().data,
            Rotation::identity(),
            Rotation::identity(),
            Vector::zeros(),
            [true; 3],
            &gains,
        );
        world.bodies[child].set_angvel(axis * 4.0, true);
        world.step();
        let effort = last_substep_effort(&world.impulse_joints.get(joint).unwrap().data, 8);
        assert!(effort.dot(&axis) < 0.0);
        assert!(world.bodies[child].angvel().dot(&axis) < 4.0);
        assert!(effort.norm() <= 3.00001);
    }
}
