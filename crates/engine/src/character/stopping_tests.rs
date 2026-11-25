use super::*;

fn stop_case(direction: Vector<Real>, running: bool, movement_ticks: usize, reverse_ticks: usize) {
    stop_on_slope(direction, running, movement_ticks, reverse_ticks, 0.0);
}

fn stop_on_slope(
    direction: Vector<Real>,
    running: bool,
    movement_ticks: usize,
    reverse_ticks: usize,
    degrees: f32,
) {
    let (mut rig, normal, _) = slope(degrees, 0.0);
    for _ in 0..60 {
        rig.step();
    }
    rig.set_input([direction.x, direction.z], running).unwrap();
    for _ in 0..movement_ticks {
        rig.step();
        bounded(&rig, normal);
    }
    if reverse_ticks > 0 {
        rig.set_input([-direction.x, -direction.z], running)
            .unwrap();
        for _ in 0..reverse_ticks {
            rig.step();
            bounded(&rig, normal);
        }
    }
    rig.set_input([0.0; 2], false).unwrap();
    for _ in 0..360 {
        rig.step();
        bounded(&rig, normal);
    }
    let velocity = Vector::from(rig.pose.rig.body_states()[2].state.linear_velocity);
    assert!(velocity.norm() < 0.2, "final velocity {velocity:?}");
    assert_eq!(rig.state().motor_strength, 0.0);
}

#[test]
fn a_sideways_run_can_stop_with_the_body_supported() {
    stop_case(Vector::x(), true, 45, 0);
}

#[test]
fn cancelling_a_reversal_can_stop_with_the_body_supported() {
    stop_case(Vector::x(), true, 45, 5);
}

#[test]
fn cardinal_walks_and_runs_stop_at_different_stride_phases() {
    for running in [false, true] {
        for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
            for quarter in 0..4 {
                let ticks = 45 + quarter * if running { 6 } else { 9 };
                eprintln!(
                    "stop running={running}, direction={direction:?}, movement_ticks={ticks}"
                );
                stop_case(direction, running, ticks, 0);
            }
        }
    }
}

#[test]
fn earlier_and_later_stops_remain_supported_on_flat_ground_and_slopes() {
    for running in [false, true] {
        for ticks in [32, 40, 45, 52, 60, 74, 90] {
            for direction in [Vector::x(), -Vector::x(), Vector::z(), -Vector::z()] {
                stop_case(direction, running, ticks, 0);
            }
            for degrees in [-15.0, 15.0] {
                eprintln!("stop slope={degrees}, running={running}, ticks={ticks}");
                stop_on_slope(-Vector::z(), running, ticks, 0, degrees);
            }
        }
    }
}

#[test]
fn stopping_assistance_still_yields_to_external_chest_impacts() {
    for (delay, impulses, directions) in [
        (0, 3, vec![Vector::x()]),
        (15, 3, vec![Vector::x()]),
        (
            60,
            12,
            vec![Vector::x(), -Vector::x(), Vector::z(), -Vector::z()],
        ),
        (
            360,
            3,
            vec![Vector::x(), -Vector::x(), Vector::z(), -Vector::z()],
        ),
    ] {
        for direction in directions {
            let (mut rig, _, _) = slope(0.0, 0.0);
            for _ in 0..60 {
                rig.step();
            }
            rig.set_input([1.0, 0.0], true).unwrap();
            for _ in 0..45 {
                rig.step();
            }
            rig.set_input([0.0; 2], false).unwrap();
            for _ in 0..delay {
                rig.step();
            }
            if delay == 360 {
                assert_eq!(rig.state().motor_strength, 0.0);
            }
            for _ in 0..impulses {
                rig.queue_impulse(BodyPart::Chest, (direction * 10.0).into())
                    .unwrap();
                rig.step();
            }
            let mut lowest = f32::MAX;
            for _ in 0..480 {
                rig.step();
                lowest = lowest.min(rig.pose.rig.body_states()[0].state.position[1]);
                assert!(Vector::from(rig.state().stopping_moment).norm() <= 120.001);
                for body in rig.pose.rig.body_states() {
                    assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
                }
            }
            assert!(
                lowest < 0.8,
                "resisted {impulses} impacts at {delay}, direction {direction:?}: {lowest}"
            );
        }
    }
}

#[test]
fn stopping_moment_clears_on_disable_reset_support_loss_and_resumed_input() {
    for action in 0..4 {
        let (mut rig, _, _) = slope(0.0, 0.0);
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([1.0, 0.0], true).unwrap();
        for _ in 0..45 {
            rig.step();
        }
        rig.set_input([0.0; 2], false).unwrap();
        rig.step();
        assert!(Vector::from(rig.state().stopping_moment).norm() > 1.0);
        match action {
            0 => rig.set_enabled(false),
            1 => rig.reset(),
            2 => {
                for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
                    if collider.parent().is_none() {
                        collider.set_enabled(false);
                    }
                }
                let before = rig.pose.rig.body_states();
                rig.controller.advance(&mut rig.pose.rig, true);
                assert_eq!(rig.pose.rig.body_states(), before);
            }
            _ => {
                rig.set_input([1.0, 0.0], false).unwrap();
                rig.step();
            }
        }
        assert_eq!(rig.state().stopping_moment, [0.0; 3]);
    }
}
