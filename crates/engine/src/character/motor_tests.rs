use super::{BodyPart, JointMotorConfig, motor::local_torque};
use rapier3d::prelude::*;
use std::f32::consts::PI;

fn gains(cap: f32) -> JointMotorConfig {
    JointMotorConfig {
        child: BodyPart::Head,
        stiffness_nm_per_rad: 6.0,
        damping_nm_s_per_rad: 0.3,
        max_torque_nm: cap,
    }
}

#[test]
fn pd_uses_radians_and_relative_velocity_in_the_same_frame() {
    let torque = local_torque(
        Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]),
        Rotation::identity(),
        vector![1.0, -2.0, 0.0],
        [true; 3],
        &gains(10.0),
    );
    assert!((torque - vector![-1.5, 0.6, 0.0]).norm() < 1e-6);
    let torque = local_torque(
        Rotation::identity(),
        Rotation::from_scaled_axis(vector![0.0, 0.3, 0.0]),
        Vector::zeros(),
        [true; 3],
        &gains(10.0),
    );
    assert!((torque - vector![0.0, 1.8, 0.0]).norm() < 1e-6);
}

#[test]
fn equivalent_quaternion_signs_use_the_same_short_arc_including_half_turns() {
    for quaternion in [
        nalgebra::Quaternion::new(0.0, 1.0, 0.0, 0.0),
        Rotation::from_scaled_axis(vector![0.1, 0.8, -0.3]).into_inner(),
    ] {
        let rotation = Rotation::new_normalize(quaternion);
        let negated = Rotation::new_normalize(-quaternion);
        let a = local_torque(
            rotation,
            Rotation::identity(),
            Vector::zeros(),
            [true; 3],
            &gains(50.0),
        );
        let b = local_torque(
            negated,
            Rotation::identity(),
            Vector::zeros(),
            [true; 3],
            &gains(50.0),
        );
        assert!((a - b).norm() < 1e-6);
        assert!(a.norm() <= 6.0 * PI + 1e-5);
        assert!(a.norm() > 0.1);
    }
    let a = Rotation::from_axis_angle(&Vector::y_axis(), 179.0_f32.to_radians());
    let b = Rotation::from_axis_angle(&Vector::y_axis(), -179.0_f32.to_radians());
    let torque = local_torque(a, b, Vector::zeros(), [true; 3], &gains(50.0));
    assert!((torque.y - 6.0 * 2.0_f32.to_radians()).abs() < 1e-5);
}

#[test]
fn locked_axes_stay_unactuated_and_the_vector_budget_cannot_be_exceeded() {
    let current = Rotation::from_scaled_axis(vector![0.4, 0.5, 0.6]);
    let all = local_torque(
        current,
        Rotation::identity(),
        vector![1.0, 2.0, 3.0],
        [true; 3],
        &gains(0.5),
    );
    assert!((all.norm() - 0.5).abs() < 1e-6);
    let hinge = local_torque(
        current,
        Rotation::identity(),
        vector![1.0, 2.0, 3.0],
        [true, false, false],
        &gains(0.5),
    );
    assert_eq!(hinge, vector![-0.5, 0.0, 0.0]);
    let fixed = local_torque(
        current,
        Rotation::identity(),
        vector![1.0, 2.0, 3.0],
        [false; 3],
        &gains(0.5),
    );
    assert_eq!(fixed, Vector::zeros());
}

#[test]
fn damping_opposes_motion_and_zero_budget_disables_all_effort() {
    let velocity = vector![1.0, -2.0, 3.0];
    let torque = local_torque(
        Rotation::identity(),
        Rotation::identity(),
        velocity,
        [true; 3],
        &gains(10.0),
    );
    assert!(torque.dot(&velocity) < 0.0);
    assert_eq!(
        local_torque(
            Rotation::identity(),
            Rotation::identity(),
            velocity,
            [true; 3],
            &gains(0.0)
        ),
        Vector::zeros()
    );
}
