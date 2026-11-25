use super::*;

fn moving(limit: u32) -> ReversalBrake {
    let mut brake = ReversalBrake::default();
    assert_eq!(
        brake.advance(Vector::x(), Vector::x() * 3.0, 0.2, limit),
        0.0
    );
    brake
}

#[test]
fn held_and_updated_reverse_inputs_cannot_extend_the_braking_window() {
    let mut brake = moving(3);
    for magnitude in [1.0, 0.4, 0.8] {
        assert_eq!(
            brake.advance(-Vector::x() * magnitude, Vector::x() * 3.0, 0.2, 3),
            1.0
        );
    }
    assert_eq!(brake.advance(-Vector::x(), Vector::x() * 3.0, 0.2, 3), 0.0);
}

#[test]
fn braking_resumes_the_latest_request_at_one_tick_of_opposing_speed() {
    let mut brake = moving(60);
    assert_eq!(
        brake.advance(-Vector::x(), vector![0.21, 9.0, 0.0], 0.2, 60),
        1.0
    );
    assert_eq!(
        brake.advance(-Vector::x() * 0.35, vector![0.2, 9.0, 0.0], 0.2, 60),
        0.0
    );
    assert_eq!(brake.advance(-Vector::x(), Vector::x() * 3.0, 0.2, 60), 0.0);
}

#[test]
fn neutral_forward_and_zero_budget_requests_do_not_wait() {
    for next in [Vector::zeros(), Vector::x(), Vector::z()] {
        let mut brake = moving(60);
        assert_eq!(brake.advance(-Vector::x(), Vector::x() * 3.0, 0.2, 60), 1.0);
        assert_eq!(brake.advance(next, -next * 3.0, 0.2, 60), 0.0);
    }
    for (ticks, quantum) in [(0, 0.2), (60, 0.0)] {
        let mut brake = moving(ticks);
        assert_eq!(
            brake.advance(-Vector::x(), Vector::x() * 3.0, quantum, ticks),
            0.0
        );
    }
}

#[test]
fn neutral_never_resumes_movement_and_remembers_departure_only_for_a_bounded_gap() {
    let mut brake = moving(3);
    for _ in 0..2 {
        assert_eq!(
            brake.advance(Vector::zeros(), Vector::x() * 3.0, 0.2, 3),
            0.0
        );
    }
    assert_eq!(brake.advance(-Vector::x(), Vector::x() * 3.0, 0.2, 3), 1.0);
    for _ in 0..3 {
        assert_eq!(
            brake.advance(Vector::zeros(), Vector::x() * 3.0, 0.2, 3),
            0.0
        );
    }
    assert_eq!(brake.advance(-Vector::x(), Vector::x() * 3.0, 0.2, 3), 0.0);
}

#[test]
fn braking_fades_continuously_across_a_perpendicular_turn() {
    for x in [-0.0001, 0.0, 0.0001] {
        let next = vector![x, 0.0, 1.0].normalize();
        let mut brake = moving(60);
        let weight = brake.advance(next, -next * 3.0, 0.2, 60);
        assert!((0.0..=0.000000011).contains(&weight));
    }
}
