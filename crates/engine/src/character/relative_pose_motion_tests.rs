use super::*;
use crate::TICK_SECONDS;

#[test]
fn accelerating_support_does_not_invent_relative_spin() {
    let mut motion = PoseMotion::default();
    let body = Some(RigidBodyHandle::from_raw_parts(3, 1));
    let relative = Rotation::from_euler_angles(0.03, 0.4, -0.02);
    let mut frame = Rotation::from_axis_angle(&Vector::y_axis(), 0.3);
    motion.sample_in_frame(
        0,
        frame * relative,
        Vector::zeros(),
        body,
        frame,
        Vector::zeros(),
    );
    for (tick, previous_speed, current_speed) in [(1, 0.2, 1.2), (2, 1.2, -0.8), (3, -0.8, 0.1)] {
        frame = Rotation::from_axis_angle(&Vector::y_axis(), previous_speed * TICK_SECONDS) * frame;
        let measured = motion.sample_in_frame(
            tick,
            frame * relative,
            Vector::y() * current_speed,
            body,
            frame,
            Vector::y() * current_speed,
        );
        assert!(
            measured.norm() < 0.00001,
            "co-moving rig measured as spinning: {measured:?}"
        );
    }
}

#[test]
fn world_stillness_does_not_hide_motion_relative_to_the_support() {
    let mut motion = PoseMotion::default();
    let body = Some(RigidBodyHandle::from_raw_parts(3, 1));
    motion.sample_in_frame(
        0,
        Rotation::identity(),
        Vector::zeros(),
        body,
        Rotation::identity(),
        Vector::zeros(),
    );
    let frame = Rotation::from_axis_angle(&Vector::y_axis(), 0.6 * TICK_SECONDS);
    let measured = motion.sample_in_frame(
        1,
        Rotation::identity(),
        Vector::zeros(),
        body,
        frame,
        Vector::zeros(),
    );
    assert!(
        (measured + Vector::y() * 0.6).norm() < 0.00001,
        "lost relative rotation: {measured:?}"
    );
}

#[test]
fn relative_rotation_is_reported_in_the_current_world_frame() {
    let mut motion = PoseMotion::default();
    let body = Some(RigidBodyHandle::from_raw_parts(1, 0));
    let frame = Rotation::from_euler_angles(0.2, 0.4, -0.3);
    let relative = Rotation::from_euler_angles(-0.1, 0.3, 0.2);
    motion.sample_in_frame(
        20,
        frame * relative,
        Vector::zeros(),
        body,
        frame,
        Vector::zeros(),
    );
    let next_frame = Rotation::from_axis_angle(&Vector::x_axis(), 0.2) * frame;
    let next_relative = Rotation::from_axis_angle(&Vector::z_axis(), 0.1 * TICK_SECONDS) * relative;
    let result = motion.sample_in_frame(
        21,
        next_frame * next_relative,
        Vector::repeat(5.0),
        body,
        next_frame,
        Vector::zeros(),
    );
    assert!((result - next_frame * Vector::z() * 0.1).norm() < 0.00001);
}

#[test]
fn frame_changes_and_discontinuous_ticks_use_current_relative_velocity() {
    let mut motion = PoseMotion::default();
    let body = Some(RigidBodyHandle::from_raw_parts(1, 0));
    let replacement = Some(RigidBodyHandle::from_raw_parts(1, 1));
    let frame = Rotation::from_euler_angles(0.3, 0.2, 0.1);
    let spin = vector![0.2, -0.3, 0.4];
    let relative = vector![0.1, 0.4, -0.2];
    for (tick, reference) in [
        (0, None),
        (1, body),
        (1, body),
        (3, body),
        (4, replacement),
        (5, None),
    ] {
        let measured = motion.sample_in_frame(
            tick,
            Rotation::identity(),
            spin + relative,
            reference,
            frame,
            spin,
        );
        assert!((measured - relative).norm() < 0.000001);
    }
}
