use super::*;
use rapier3d::prelude::*;

fn stop_on_deck(spawn: f32, rotating: bool, running: bool, ticks: usize, direction: Vector<Real>) {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [spawn, 0.0, 0.0],
    )
    .unwrap();
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let platform = world.bodies.insert(
        RigidBodyBuilder::kinematic_velocity_based()
            .translation(vector![0.0, -0.5, 0.0])
            .linvel(if rotating {
                Vector::zeros()
            } else {
                Vector::x() * 0.8
            })
            .angvel(if rotating {
                Vector::y() * 0.4
            } else {
                Vector::zeros()
            }),
    );
    let extent = if spawn == 0.0 { 6.0 } else { 30.0 };
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(extent, 0.5, extent),
        platform,
        &mut world.bodies,
    );
    for _ in 0..if spawn == 0.0 { 60 } else { 120 } {
        rig.step();
    }
    rig.set_input([direction.x, direction.z], running).unwrap();
    for _ in 0..ticks {
        rig.step();
        assert!(rig.pose.rig.body_states()[0].state.position[1] > 1.1);
    }
    rig.set_input([0.0; 2], false).unwrap();
    for release_tick in 0..360 {
        rig.step();
        assert!(
            rig.pose.rig.body_states()[0].state.position[1] > 1.1,
            "spawn={spawn}, rotating={rotating}, running={running}, ticks={ticks}, direction={direction:?}, release_tick={release_tick}"
        );
        for body in rig.pose.rig.body_states() {
            assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
        }
        assert!(Vector::from(rig.state().acceleration).norm() <= 12.0001);
        assert!(
            Vector::from(rig.state().stopping_moment).norm()
                <= 120.0 * rig.state().motor_strength + 0.001
        );
        for joint in rig.pose.rig.joint_states() {
            assert!(joint.anchor_error < 0.05);
        }
    }
    let body = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[2]];
    let surface = rig.pose.rig.world.bodies[platform].velocity_at_point(body.center_of_mass());
    assert!((body.linvel() - surface).norm() < 0.2);
    assert_eq!(
        rig.state().motor_strength,
        0.0,
        "spawn={spawn}, rotating={rotating}, running={running}, ticks={ticks}, direction={direction:?}"
    );
    assert_eq!(rig.state().stopping_moment, [0.0; 3]);
}

#[test]
fn supported_deck_stops_relax_both_motors_and_stopping_moment() {
    for spawn in [0.0, 2.0] {
        for rotating in [false, true] {
            for running in [false, true] {
                for ticks in if spawn == 0.0 { [32, 45] } else { [24, 45] } {
                    for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
                        stop_on_deck(spawn, rotating, running, ticks, direction);
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "Known rotating-deck traversal failure also occurs with input held; blocks movement acceptance (terrain-gaps.md)"]
fn longer_rotating_deck_runs_keep_posture_when_released() {
    for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
        stop_on_deck(0.0, true, true, 60, direction);
    }
}
