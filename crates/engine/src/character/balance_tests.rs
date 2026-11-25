use super::*;
use crate::{CharacterDefinition, PhysicsConfig};

fn fixture() -> (PassiveRagdoll, BalanceController) {
    let mut rig = PassiveRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    rig.step();
    let controller = BalanceController {
        config: BalanceConfig::bundled().unwrap(),
        state: BalanceState::default(),
    };
    (rig, controller)
}

#[test]
fn tilt_assistance_halves_with_one_foot_and_stops_without_support() {
    let (mut rig, mut controller) = fixture();
    let root = &mut rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]];
    root.set_rotation(Rotation::from_scaled_axis(vector![0.2, 0.0, 0.0]), true);
    root.set_angvel(Vector::zeros(), true);
    controller.sample(&rig);
    assert_eq!(controller.state.feet, [true; 2]);
    assert!((Vector::from(controller.state.torque) - vector![-30.0, 0.0, 0.0]).norm() < 1e-5);
    for (part, expected) in [(BodyPart::LeftFoot, -15.0), (BodyPart::RightFoot, 0.0)] {
        rig.world.bodies[rig.bodies[part.index()]].set_translation(vector![0.0, 2.0, 0.0], true);
        controller.sample(&rig);
        assert!(
            (Vector::from(controller.state.torque) - vector![expected, 0.0, 0.0]).norm() < 1e-5
        );
    }
}

#[test]
fn yaw_is_free_and_an_exactly_inverted_body_has_a_bounded_correction() {
    let (mut rig, mut controller) = fixture();
    let handle = rig.bodies[BodyPart::Pelvis.index()];
    rig.world.bodies[handle].set_rotation(Rotation::from_axis_angle(&Vector::y_axis(), 1.0), true);
    rig.world.bodies[handle].set_angvel(vector![0.0, 3.0, 0.0], true);
    controller.sample(&rig);
    assert_eq!(controller.state.torque, [0.0; 3]);
    rig.world.bodies[handle].set_rotation(
        Rotation::new_normalize(nalgebra::Quaternion::new(0.0, 1.0, 0.0, 0.0)),
        true,
    );
    rig.world.bodies[handle].set_angvel(Vector::zeros(), true);
    controller.sample(&rig);
    let torque = Vector::from(controller.state.torque);
    assert!(torque.iter().all(|value| value.is_finite()));
    assert!((torque.norm() - 30.0).abs() < 1e-5);
    assert_eq!(torque.y, 0.0);
}
