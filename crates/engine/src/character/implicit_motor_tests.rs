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
    let cap = 3.0 / 3.0_f32.sqrt();
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
