#[path = "support_drive_contract_tests.rs"]
mod contracts;

use super::*;
use rapier3d::prelude::*;

fn rotating_deck(spin: f32, radius: f32) -> (LocomotionRagdoll, RigidBodyHandle) {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [radius, 0.0, 0.0],
    )
    .unwrap();
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let deck = world.bodies.insert(
        RigidBodyBuilder::kinematic_velocity_based()
            .translation(vector![0.0, -0.5, 0.0])
            .angvel(Vector::y() * spin),
    );
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(30.0, 0.5, 30.0),
        deck,
        &mut world.bodies,
    );
    (rig, deck)
}

#[test]
fn sustained_traversal_on_rotating_support_preserves_posture_and_relative_progress() {
    for spin in [-0.4, 0.4] {
        for radius in [0.0, 2.0] {
            for running in [false, true] {
                for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
                    let (mut rig, deck) = rotating_deck(spin, radius);
                    for _ in 0..60 {
                        rig.step();
                    }
                    rig.set_input([direction.x, direction.z], running).unwrap();
                    let mut distance = 0.0;
                    for tick in 0..240 {
                        rig.step();
                        let bodies = rig.pose.rig.body_states();
                        assert!(
                            bodies[0].state.position[1] > 1.1,
                            "spin={spin}, radius={radius}, running={running}, direction={direction:?}, tick={tick}"
                        );
                        for body in bodies {
                            assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
                        }
                        let mut center = Vector::zeros();
                        let mut velocity = Vector::zeros();
                        for handle in rig.pose.rig.bodies {
                            let body = &rig.pose.rig.world.bodies[handle];
                            center += body.center_of_mass().coords * body.mass() / 40.0;
                            velocity += body.linvel() * body.mass() / 40.0;
                        }
                        let surface =
                            rig.pose.rig.world.bodies[deck].velocity_at_point(&Point::from(center));
                        distance += (velocity - surface).dot(&direction) * crate::TICK_SECONDS;
                        for joint in rig.pose.rig.joint_states() {
                            assert!(joint.anchor_error < 0.05);
                        }
                        assert!(Vector::from(rig.state().acceleration).norm() <= 12.0001);
                        assert_eq!(rig.state().acceleration[1], 0.0);
                        assert!(Vector::from(rig.state().support_moment).norm() <= 120.001);
                        assert!(rig.state().turn_torque.abs() <= 20.0001);
                    }
                    let speed = if running { 4.5 } else { 2.5 };
                    assert!(distance > speed * 4.0 * 0.7, "relative progress {distance}");
                    rig.set_input([0.0; 2], false).unwrap();
                    for release_tick in 0..360 {
                        rig.step();
                        assert!(
                            rig.pose.rig.body_states()[0].state.position[1] > 1.1,
                            "release: spin={spin}, radius={radius}, running={running}, direction={direction:?}, tick={release_tick}"
                        );
                    }
                    let pelvis = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[2]];
                    let surface =
                        rig.pose.rig.world.bodies[deck].velocity_at_point(pelvis.center_of_mass());
                    assert!((pelvis.linvel() - surface).norm() < 0.2);
                    assert_eq!(rig.state().motor_strength, 0.0);
                    assert_eq!(rig.state().stopping_moment, [0.0; 3]);
                }
            }
        }
    }
}

#[test]
fn rotating_support_assistance_yields_to_sustained_chest_impacts() {
    for spin in [-0.4, 0.4] {
        for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
            let (mut rig, _) = rotating_deck(spin, 0.0);
            for _ in 0..60 {
                rig.step();
            }
            rig.set_input([1.0, 0.0], true).unwrap();
            let mut toppled = false;
            for tick in 0..480 {
                // Repeated bounded shoves test sustained external overpowering,
                // rather than requiring every small impact to defeat support.
                if (99..111).contains(&tick) {
                    rig.queue_impulse(BodyPart::Chest, (direction * 10.0).into())
                        .unwrap();
                }
                rig.step();
                let bodies = rig.pose.rig.body_states();
                toppled |= bodies[0].state.position[1] < 0.8;
                assert!(
                    bodies
                        .iter()
                        .all(|body| body.state.to_buffer().iter().all(|v| v.is_finite()))
                );
                assert!(Vector::from(rig.state().support_moment).norm() <= 120.001);
                assert!(Vector::from(rig.state().acceleration).norm() <= 12.0001);
            }
            assert!(toppled, "spin={spin}, impact={direction:?}");
        }
    }
}

#[test]
fn support_assistance_clears_on_disable_reset_lost_support_and_ineligible_posture() {
    for exit in 0..4 {
        let (mut rig, deck) = rotating_deck(0.4, 2.0);
        for _ in 0..60 {
            rig.step();
        }
        // Exercise compensation during relative movement before clearing it;
        // steady deck carry can already be supplied by contacts and idle drive.
        rig.set_input([1.0, 0.0], true).unwrap();
        rig.step();
        assert!(Vector::from(rig.state().support_moment).norm() > 0.01);
        match exit {
            0 => rig.set_enabled(false),
            1 => rig.reset(),
            _ => {
                if exit == 2 {
                    rig.pose.rig.world.bodies[deck].set_enabled(false);
                } else {
                    let root = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
                    rig.pose.rig.world.bodies[root].set_rotation(
                        Rotation::from_axis_angle(&Vector::x_axis(), std::f32::consts::PI),
                        true,
                    );
                }
                let before = rig.pose.rig.body_states();
                rig.controller.advance(&mut rig.pose.rig, true);
                assert_eq!(before, rig.pose.rig.body_states());
            }
        }
        assert_eq!(rig.state().support_moment, [0.0; 3]);
        assert_eq!(rig.state().acceleration, [0.0; 3]);
    }
}

#[test]
fn zero_linear_budget_disables_rotating_support_impulses() {
    let (mut rig, _) = rotating_deck(0.4, 2.0);
    for _ in 0..60 {
        rig.step();
    }
    let mut config = LocomotionConfig::bundled().unwrap();
    config.maximum_acceleration_m_s2 = 0.0;
    config.validate().unwrap();
    rig.controller = WalkingController::new(config);
    let before = rig.pose.rig.body_states();
    rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.state().support_moment, [0.0; 3]);
    assert_eq!(rig.state().acceleration, [0.0; 3]);
    assert_eq!(before, rig.pose.rig.body_states());
}

#[path = "swing_completion_tests.rs"]
mod swing_completion_tests;

#[path = "swing_completion_lifecycle_tests.rs"]
mod swing_completion_lifecycle_tests;
