use super::*;
use crate::character::jump::{Jump, JumpConfig};
use rapier3d::prelude::*;

#[test]
fn press_window_includes_the_sixth_sample_but_excludes_the_seventh() {
    let config = JumpConfig::default();
    for delay in 0..=6 {
        let mut jump = Jump::default();
        jump.input(true, &config);
        jump.input(false, &config);
        for _ in 0..delay {
            assert!(!jump.takeoff(false, true, &config));
        }
        assert_eq!(jump.takeoff(true, true, &config), delay < 6);
        assert!(!jump.takeoff(true, true, &config));
    }
}

#[test]
fn coyote_launch_has_six_ticks_after_physical_platform_departure() {
    for absent_ticks in [0, 5, 6] {
        let mut rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
            [0.; 3],
        )
        .unwrap();
        for _ in 0..60 {
            rig.step();
        }
        assert_eq!(rig.state().feet, [true, true]);
        for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
            if collider.parent().is_none() {
                collider.set_enabled(false);
            }
        }
        for _ in 0..absent_ticks {
            rig.step();
        }
        rig.set_jump(true);
        rig.step();
        assert_eq!(rig.state().feet, [false; 2]);
        let pelvis = rig.pose.rig.body_states()[BodyPart::Pelvis.index()].state;
        assert_eq!(
            pelvis.linear_velocity[1] > 2.0,
            absent_ticks < 6,
            "elapsed {absent_ticks}, velocity {:?}",
            pelvis.linear_velocity
        );
    }
}

#[test]
fn alternating_air_steering_spends_one_impulse_budget_across_the_whole_jump() {
    let config = JumpConfig::default();
    let mut jump = Jump::default();
    assert_eq!(jump.steer(Vector::x(), 2.4), Vector::zeros());
    jump.input(true, &config);
    assert!(jump.takeoff(true, true, &config));
    let mut total = 0.;
    for tick in 0..100 {
        let axis = if tick % 2 == 0 {
            Vector::x()
        } else {
            -Vector::x()
        };
        let delta = jump.steer(axis, 2.4);
        assert!(delta.norm() <= 2.4 / 60. + 1e-6);
        total += delta.norm();
    }
    assert!((total - config.air_velocity_budget_m_s).abs() < 1e-5);
}

#[test]
fn persistent_support_after_launch_cannot_fund_a_second_jump() {
    let config = JumpConfig::default();
    let mut jump = Jump::default();
    jump.input(true, &config);
    assert!(jump.takeoff(true, true, &config));
    jump.input(false, &config);
    jump.input(true, &config);
    assert!(!jump.takeoff(true, true, &config));
    assert!(!jump.takeoff(false, true, &config));
    assert!(jump.takeoff(true, true, &config));
    jump.input(false, &config);
    jump.input(true, &config);
    assert!(!jump.takeoff(true, false, &config));
    assert!(!jump.takeoff(true, true, &config));
}

#[test]
fn takeoff_preserves_each_limb_pose_and_spin_and_assigns_impulse_by_mass() {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.; 3],
    )
    .unwrap();
    for _ in 0..60 {
        rig.step();
    }
    let before = rig.pose.rig.body_states();
    rig.set_jump(true);
    rig.controller.advance(&mut rig.pose.rig, true);
    let after = rig.pose.rig.body_states();
    let mut total = 0.;
    for (a, b) in before.iter().zip(after) {
        assert_eq!(a.state.position, b.state.position);
        assert_eq!(a.state.rotation, b.state.rotation);
        assert_eq!(a.state.angular_velocity, b.state.angular_velocity);
        let delta = Vector::from(b.state.linear_velocity) - Vector::from(a.state.linear_velocity);
        assert!((delta - Vector::y() * 4.5).norm() < 1e-5);
        total += delta.norm() * a.state.mass;
    }
    assert!((total - 180.).abs() < 1e-3);
}
