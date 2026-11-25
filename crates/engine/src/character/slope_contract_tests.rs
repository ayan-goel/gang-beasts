use super::*;

#[test]
fn slope_targets_preserve_placement_when_the_ankle_saturates_and_do_not_follow_body_tilt() {
    let (mut rig, normal, _) = slope(15.0, 0.0);
    for _ in 0..120 {
        rig.step();
    }
    let contacts = supports(&rig);
    assert!(contacts.iter().all(Option::is_some));
    let root = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
    rig.pose.rig.world.bodies[root].set_rotation(Rotation::identity(), true);
    let config = LocomotionConfig::bundled().unwrap();
    let mut nominal = [[0.0; 3]; 14];
    nominal[8] = [0.5, 0.0, 0.0];
    nominal[10] = [-1.0, 0.0, 0.0];
    nominal[12] = [0.5, 0.0, 0.0];
    let mut adapted = nominal;
    crate::character::terrain_gait::adapt(&rig.pose.rig, contacts, &config, 1.0, &mut adapted);
    let z = -0.02 * <f32 as ComplexField>::sin(0.5);
    let y = -0.62 * <f32 as ComplexField>::cos(0.5) + 0.62 * 0.25 * normal.z / normal.y
        - normal.z * z / normal.y;
    let rotation = |a: [f32; 3]| Rotation::from_euler_angles(a[0], a[1], a[2]);
    let actual =
        rotation(adapted[8]) * (-Vector::y() * 0.32 + rotation(adapted[10]) * -Vector::y() * 0.30);
    assert!((actual - vector![0.0, y, z]).norm() < 0.00001);
    assert_eq!(adapted[12][0], 0.5);
    // A limited ankle may miss the exact normal; moving the hip to erase that
    // error would displace the requested step and caused the regression.
    let sole = rotation(adapted[8]) * rotation(adapted[10]) * rotation(adapted[12]);
    assert!((sole * Vector::y() - normal).norm() > 0.01);
    rig.pose.rig.world.bodies[root]
        .set_rotation(Rotation::from_axis_angle(&Vector::x_axis(), 0.2), true);
    let shin = rig.pose.rig.bodies[BodyPart::LeftShin.index()];
    rig.pose.rig.world.bodies[shin]
        .set_rotation(Rotation::from_axis_angle(&Vector::x_axis(), -0.3), true);
    let mut tilted = nominal;
    crate::character::terrain_gait::adapt(&rig.pose.rig, contacts, &config, 1.0, &mut tilted);
    assert_eq!(adapted, tilted);
}

#[test]
fn braking_counteracts_foot_drag_without_exceeding_the_absolute_impulse_budget() {
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
        let before = rig.pose.rig.body_states();
        rig.controller.advance(&mut rig.pose.rig, true);
        let mut total = Vector::zeros();
        let mut absolute = 0.0;
        let mut feet = Vector::zeros();
        for (index, (old, new)) in before.iter().zip(rig.pose.rig.body_states()).enumerate() {
            assert_eq!(old.state.position, new.state.position);
            assert_eq!(old.state.rotation, new.state.rotation);
            let impulse = (Vector::from(new.state.linear_velocity)
                - Vector::from(old.state.linear_velocity))
                * old.state.mass;
            total += impulse;
            absolute += impulse.norm();
            if index == BodyPart::LeftFoot.index() || index == BodyPart::RightFoot.index() {
                feet += impulse;
            }
        }
        assert!(
            total.z > 0.0 && feet.z < 0.0,
            "total {total:?}, feet {feet:?}"
        );
        assert!(absolute <= 40.0 * 12.0 * crate::TICK_SECONDS + 0.0001);
        assert!(
            (total - Vector::from(rig.state().acceleration) * 40.0 * crate::TICK_SECONDS).norm()
                < 0.0001
        );
        assert_eq!(total.y, 0.0);
    }
}
