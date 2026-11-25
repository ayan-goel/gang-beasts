#[path = "slope_contract_tests.rs"]
mod contracts;

#[path = "turning_tests.rs"]
mod turning;

#[path = "stopping_tests.rs"]
mod stopping;

use super::*;
use rapier3d::na::ComplexField;
use rapier3d::prelude::*;

fn slope(degrees: f32, yaw: f32) -> (LocomotionRagdoll, Vector<Real>, Vector<Real>) {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.0, 0.08, 0.0],
    )
    .unwrap();
    let heading = Rotation::from_axis_angle(&Vector::y_axis(), yaw);
    let rotation = heading * Rotation::from_axis_angle(&Vector::x_axis(), degrees.to_radians());
    let normal = rotation * Vector::y();
    let world = &mut rig.pose.rig.world;
    for handle in rig.pose.rig.bodies {
        let body = &mut world.bodies[handle];
        body.set_position(
            Isometry::from_parts(Translation::from(heading * body.translation()), heading),
            true,
        );
    }
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let platform = world
        .colliders
        .insert(
            ColliderBuilder::cuboid(30.0, 0.5, 30.0).position(Isometry::from_parts(
                Translation::from(vector![0.0, -0.5 / normal.y, 0.0]),
                rotation,
            )),
        );
    for part in [BodyPart::LeftFoot, BodyPart::RightFoot] {
        let body = &world.bodies[rig.pose.rig.bodies[part.index()]];
        let foot = &world.colliders[body.colliders()[0]];
        let surface = &world.colliders[platform];
        let pose = body.position() * foot.position_wrt_parent().unwrap();
        let contact = rapier3d::parry::query::contact(
            &pose,
            foot.shape(),
            surface.position(),
            surface.shape(),
            0.2,
        )
        .unwrap()
        .unwrap();
        assert!(
            contact.dist > 0.0,
            "fixture starts penetrating by {}",
            -contact.dist
        );
    }
    (rig, normal, heading * -Vector::z())
}

fn bounded(rig: &LocomotionRagdoll, normal: Vector<Real>) {
    for body in rig.pose.rig.body_states() {
        assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
    }
    assert!(Vector::from(rig.state().acceleration).norm() <= 12.0001);
    assert_eq!(rig.state().acceleration[1], 0.0);
    assert!(rig.state().turn_torque.abs() <= 20.0001);
    assert!(
        Vector::from(rig.state().stopping_moment).norm()
            <= 120.0 * rig.state().motor_strength + 0.001
    );
    assert!(Vector::from(rig.pose.balance_state().unwrap().torque).norm() <= 30.0001);
    for joint in rig.pose.rig.joint_states() {
        assert!(joint.anchor_error < 0.05);
    }
    for (effort, motor) in rig
        .pose
        .solver_motor_efforts()
        .unwrap()
        .iter()
        .zip(&rig.pose.motor_configuration().joints)
    {
        assert!(Vector::from(*effort).norm() <= motor.max_torque_nm + 0.001);
    }
    let head = Vector::from(rig.pose.rig.body_states()[0].state.position);
    assert!(
        normal.dot(&head) / normal.y > 1.1,
        "tick {} head {head:?}, normal {normal:?}",
        rig.pose.rig.tick()
    );
}

#[test]
fn standing_remains_supported_on_moderate_slopes() {
    for degrees in [-15.0, 15.0] {
        for yaw in [
            0.0,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            std::f32::consts::PI * 1.5,
        ] {
            let (mut rig, normal, _) = slope(degrees, yaw);
            for _ in 0..3600 {
                rig.step();
                bounded(&rig, normal);
            }
            let pelvis = Vector::from(rig.pose.rig.body_states()[2].state.position);
            assert!(vector![pelvis.x, 0.0, pelvis.z].norm() < 0.25);
        }
    }
}

#[test]
fn walking_climbs_and_descends_moderate_slopes() {
    for degrees in [-15.0, 15.0] {
        for yaw in [
            0.0,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            std::f32::consts::PI * 1.5,
        ] {
            for running in [false, true] {
                let (mut rig, normal, direction) = slope(degrees, yaw);
                for _ in 0..120 {
                    rig.step();
                    bounded(&rig, normal);
                }
                let start = Vector::from(rig.pose.rig.body_states()[2].state.position);
                rig.set_input([direction.x, direction.z], running).unwrap();
                for _ in 0..240 {
                    rig.step();
                    bounded(&rig, normal);
                }
                let end = Vector::from(rig.pose.rig.body_states()[2].state.position);
                let speed = if running { 4.5 } else { 2.5 };
                assert!((end - start).dot(&direction) > speed * 4.0 * 0.7);
            }
        }
    }
}

fn supports(rig: &LocomotionRagdoll) -> [Option<super::super::support::FootSupport>; 2] {
    [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
        super::super::support::foot_support(
            &rig.pose.rig.world,
            rig.pose.rig.bodies[part.index()],
            &rig.pose.rig.bodies,
            &BalanceConfig::bundled().unwrap(),
        )
    })
}

#[test]
fn sloped_standing_still_yields_to_chest_impacts() {
    for degrees in [-15.0, 15.0] {
        let (mut rig, normal, _) = slope(degrees, 0.0);
        for _ in 0..120 {
            rig.step();
            bounded(&rig, normal);
        }
        for _ in 0..3 {
            rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])
                .unwrap();
            rig.step();
        }
        let mut lowest = 2.0_f32;
        for _ in 0..480 {
            rig.step();
            let head = Vector::from(rig.pose.rig.body_states()[0].state.position);
            lowest = lowest.min(normal.dot(&head) / normal.y);
        }
        assert!(lowest < 0.8, "slope {degrees} resisted impacts: {lowest}");
    }
}

#[test]
fn removed_slope_supplies_no_remaining_drive_or_world_torque() {
    let (mut rig, normal, _) = slope(15.0, 0.0);
    for _ in 0..120 {
        rig.step();
    }
    rig.set_input([0.0, -1.0], true).unwrap();
    for _ in 0..60 {
        rig.step();
        bounded(&rig, normal);
    }
    for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let before = rig.pose.rig.body_states();
    let targets = rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.state().feet, [false; 2]);
    assert_eq!(rig.state().acceleration, [0.0; 3]);
    assert_eq!(rig.state().turn_torque, 0.0);
    assert_eq!(targets, [Rotation::identity(); 14]);
    assert_eq!(rig.pose.rig.body_states(), before);
}

#[test]
fn releasing_movement_on_a_slope_returns_to_supported_idle() {
    for degrees in [-15.0, 15.0] {
        let (mut rig, normal, _) = slope(degrees, 0.0);
        for _ in 0..120 {
            rig.step();
        }
        rig.set_input([0.0, -1.0], true).unwrap();
        for _ in 0..90 {
            rig.step();
            bounded(&rig, normal);
        }
        rig.set_input([0.0, 0.0], false).unwrap();
        for _ in 0..600 {
            rig.step();
            bounded(&rig, normal);
        }
        let pelvis = rig.pose.rig.body_states()[2].state;
        assert!(Vector::from(pelvis.linear_velocity).norm() < 0.2);
        assert_eq!(rig.state().motor_strength, 0.0);
    }
}
