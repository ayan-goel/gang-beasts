use super::*;

fn fixture() -> PoseRagdoll {
    let mut rig = PoseRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        MotorConfig::standing().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap();
    let head = rig.rig.bodies[BodyPart::Head.index()];
    rig.rig.world.bodies[head]
        .set_rotation(Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]), true);
    rig
}

#[test]
fn local_solver_effort_and_completed_frame_display_are_distinct_under_transverse_spin() {
    let mut rig = fixture();
    for (_, body) in rig.rig.world.bodies.iter_mut() {
        body.set_angvel(vector![0.0, 0.0, 3.0], true);
    }
    rig.step();
    let local = Vector::from(rig.solver_motor_efforts().unwrap()[0]);
    let parent = rig.rig.world.impulse_joints.iter().next().unwrap().1.body1;
    let completed_frame = rig.rig.world.bodies[parent].rotation();
    let displayed = Vector::from(rig.motor_torques()[0]);
    assert!((displayed - completed_frame * local).norm() < 1e-5);
    assert!(
        (displayed - local).norm() > 0.001,
        "transverse rotation must distinguish frame conventions"
    );
}

#[test]
fn a_skipped_sleeping_solve_does_not_report_previous_effort() {
    let mut rig = fixture();
    rig.step();
    assert!(
        rig.solver_motor_efforts()
            .unwrap()
            .iter()
            .flatten()
            .any(|value| value.abs() > 0.1)
    );
    for (_, body) in rig.rig.world.bodies.iter_mut() {
        body.sleep();
    }
    rig.step();
    assert!(
        rig.rig
            .world
            .bodies
            .iter()
            .all(|(_, body)| body.is_sleeping())
    );
    assert_eq!(rig.solver_motor_efforts().unwrap(), [[0.0; 3]; 14]);
}

#[test]
fn toggling_control_twice_while_paused_does_not_restore_stale_effort() {
    let mut rig = fixture();
    rig.step();
    assert!(
        rig.solver_motor_efforts()
            .unwrap()
            .iter()
            .flatten()
            .any(|value| value.abs() > 0.1)
    );
    rig.set_enabled(false);
    rig.set_enabled(true);
    assert_eq!(rig.solver_motor_efforts().unwrap(), [[0.0; 3]; 14]);
    assert_eq!(rig.motor_torques(), &[[0.0; 3]; 14]);
}

#[test]
fn solver_velocity_targets_are_current_and_clear_on_toggle_and_reset() {
    let mut rig = fixture();
    assert_eq!(rig.solver_motor_targets().unwrap(), [[0.0; 3]; 14]);
    rig.step();
    assert!(rig.solver_motor_targets().unwrap()[0][0] < 0.0);
    rig.set_enabled(false);
    rig.set_enabled(true);
    assert_eq!(rig.solver_motor_targets().unwrap(), [[0.0; 3]; 14]);
    rig.step();
    assert!(
        rig.solver_motor_targets()
            .unwrap()
            .iter()
            .flatten()
            .any(|value| value.abs() > 0.01)
    );
    rig.reset();
    assert_eq!(rig.solver_motor_targets().unwrap(), [[0.0; 3]; 14]);
}
