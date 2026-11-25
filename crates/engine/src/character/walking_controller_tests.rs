use super::*;

#[test]
fn reset_immediately_clears_reversal_and_recent_direction_history() {
    let mut controller = WalkingController::new(LocomotionConfig::bundled().unwrap());
    let old_velocity = Vector::x() * 3.0;
    controller
        .reversal
        .advance(Vector::x(), old_velocity, 0.2, 60);
    assert_eq!(
        controller
            .reversal
            .advance(-Vector::x(), old_velocity, 0.2, 60),
        1.0
    );
    controller.reset();
    assert_eq!(
        controller
            .reversal
            .advance(-Vector::x(), old_velocity, 0.2, 60),
        0.0
    );
}
