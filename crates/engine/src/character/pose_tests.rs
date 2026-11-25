use super::*;

fn fixture() -> PoseRagdoll {
    PoseRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        MotorConfig::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap()
}

fn perturb(rig: &mut PoseRagdoll) {
    for (index, (_, body)) in rig.rig.world.bodies.iter_mut().enumerate() {
        body.set_rotation(
            Rotation::from_scaled_axis(vector![0.01 * index as f32, 0.02, -0.01]),
            true,
        );
        body.set_angvel(vector![0.1 * index as f32, -0.3, 0.2], true);
    }
}

fn angular_momentum(rig: &PoseRagdoll) -> Vector<Real> {
    rig.rig
        .world
        .bodies
        .iter()
        .fold(Vector::zeros(), |sum, (_, body)| {
            let local_inertia = body
                .mass_properties()
                .local_mprops
                .reconstruct_inertia_matrix();
            sum + body.rotation() * (local_inertia * (body.rotation().inverse() * body.angvel()))
        })
}

#[test]
fn joint_actuation_preserves_total_angular_momentum_and_does_not_move_transforms() {
    let mut rig = fixture();
    perturb(&mut rig);
    let initial = angular_momentum(&rig);
    let states = rig.rig.body_states();
    rig.apply_motors(&[Rotation::identity(); 14]);
    assert!((angular_momentum(&rig) - initial).norm() < 1e-6);
    for (before, after) in states.iter().zip(rig.rig.body_states()) {
        assert_eq!(before.state.position, after.state.position);
        assert_eq!(before.state.rotation, after.state.rotation);
        assert_eq!(before.state.linear_velocity, after.state.linear_velocity);
    }
    assert!(
        states
            .iter()
            .zip(rig.rig.body_states())
            .any(|(a, b)| a.state.angular_velocity != b.state.angular_velocity)
    );
}

#[test]
fn rotating_the_whole_rig_rotates_the_motor_output_without_changing_its_magnitude() {
    let mut first = fixture();
    let mut second = fixture();
    perturb(&mut first);
    perturb(&mut second);
    let rotation = Rotation::from_euler_angles(0.7, -0.9, 0.3);
    for (_, body) in second.rig.world.bodies.iter_mut() {
        body.set_rotation(rotation * body.rotation(), true);
        body.set_angvel(rotation * body.angvel(), true);
    }
    first.apply_motors(&[Rotation::identity(); 14]);
    second.apply_motors(&[Rotation::identity(); 14]);
    for (a, b) in first.motor_torques().iter().zip(second.motor_torques()) {
        assert!((rotation * Vector::from(*a) - Vector::from(*b)).norm() < 2e-5);
    }
}

#[test]
fn a_large_external_spin_overpowers_the_motor_on_its_next_tick() {
    let mut rig = fixture();
    let handle = rig.rig.world.impulse_joints.iter().next().unwrap().1.body2;
    rig.rig.world.bodies[handle].set_angvel(vector![50.0, 0.0, 0.0], true);
    rig.apply_motors(&[Rotation::identity(); 14]);
    assert!(rig.rig.world.bodies[handle].angvel().x > 45.0);
    assert!(rig.rig.world.bodies[handle].angvel().x < 50.0);
    rig.set_enabled(false);
    let before = rig.rig.body_states();
    rig.apply_motors(&[Rotation::identity(); 14]);
    assert_eq!(rig.rig.body_states(), before);
}

#[test]
fn a_displaced_neck_restores_with_the_exact_tick_impulse() {
    let mut rig = fixture();
    for motor in &mut rig.motors.joints {
        if motor.child != BodyPart::Head {
            motor.max_torque_nm = 0.0;
        }
    }
    let joint = rig.rig.world.impulse_joints.iter().next().unwrap().1;
    let (parent, child) = (joint.body1, joint.body2);
    rig.rig.world.bodies[child]
        .set_rotation(Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]), true);
    assert_ne!(
        rig.rig.body_states()[BodyPart::Head.index()].state.rotation,
        [0.0, 0.0, 0.0, 1.0]
    );
    rig.apply_motors(&[Rotation::identity(); 14]);
    assert!((Vector::from(rig.motor_torques()[0]) - vector![-1.2, 0.0, 0.0]).norm() < 1e-6);
    // Independent physical oracle: 6 N·m/rad × 0.2 rad × 1/60 s = 0.02 N·m·s.
    // This catches a missing spring, reversed actuation, or incorrect tick scaling.
    for (handle, expected) in [(child, -0.02), (parent, 0.02)] {
        let body = &rig.rig.world.bodies[handle];
        let inertia = body
            .mass_properties()
            .local_mprops
            .reconstruct_inertia_matrix();
        let momentum = body.rotation() * (inertia * (body.rotation().inverse() * body.angvel()));
        assert!((momentum - vector![expected, 0.0, 0.0]).norm() < 1e-6);
    }
}

#[test]
fn enabling_solver_motors_wakes_a_sleeping_rig() {
    let mut rig = fixture();
    rig.motors = MotorConfig::standing().unwrap();
    rig.set_enabled(false);
    let joint = rig.rig.world.impulse_joints.iter().next().unwrap().1;
    let (parent, child) = (joint.body1, joint.body2);
    rig.rig.world.bodies[child]
        .set_rotation(Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]), true);
    for (_, body) in rig.rig.world.bodies.iter_mut() {
        body.sleep();
    }
    rig.set_enabled(true);
    assert!(
        rig.rig
            .world
            .bodies
            .iter()
            .all(|(_, body)| !body.is_sleeping())
    );
    rig.step();
    assert!(
        rig.rig.world.bodies[parent]
            .rotation()
            .angle_to(rig.rig.world.bodies[child].rotation())
            < 0.2
    );
}

#[test]
fn queued_foot_departure_is_consumed_before_support_and_balance() {
    let mut rig = PoseRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        MotorConfig::standing().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    rig.configure_balance(Some(BalanceConfig::bundled().unwrap()))
        .unwrap();
    rig.step();
    let root = rig.rig.bodies[BodyPart::Pelvis.index()];
    rig.rig.world.bodies[root]
        .set_rotation(Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]), true);
    rig.rig.world.bodies[root].set_angvel(Vector::zeros(), true);
    rig.queue_impulse(BodyPart::LeftFoot, [0.0, 10.0, 0.0])
        .unwrap();
    rig.step();
    assert_eq!(rig.balance_state().unwrap().feet, [false, true]);
    assert!((Vector::from(rig.balance_state().unwrap().torque).norm() - 15.0).abs() < 1e-5);
}
