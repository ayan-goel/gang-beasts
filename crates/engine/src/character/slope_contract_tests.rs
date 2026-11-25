use super::*;

#[test]
fn slope_gait_does_not_command_intersecting_feet() {
    for degrees in [-15.0, 15.0] {
        for terrain_heading in [0.0, std::f32::consts::FRAC_PI_2] {
            let (mut rig, _, _) = slope_with_headings(degrees, 0.0, terrain_heading);
            for _ in 0..120 {
                rig.step();
            }
            let contacts = supports(&rig);
            assert!(contacts.iter().all(Option::is_some));
            let root = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
            rig.pose.rig.world.bodies[root].set_rotation(Rotation::identity(), true);
            let config = LocomotionConfig::bundled().unwrap();
            for direction_index in 0..16 {
                let angle = direction_index as f32 * std::f32::consts::TAU / 16.0;
                let direction = vector![angle.sin(), 0.0, -angle.cos()];
                for sample in 0..120 {
                    let phase = sample as f32 * std::f32::consts::TAU / 120.0;
                    let mut targets = crate::character::gait::targets(
                        &rig.pose.rig.definition,
                        &config,
                        direction,
                        phase,
                        1.0,
                    );
                    crate::character::terrain_gait::adapt(
                        rig.pose.rig.read_frame(),
                        contacts,
                        &config,
                        1.0,
                        &mut targets,
                    );
                    let separation = crate::character::gait::geometry_tests::foot_separation(
                        &rig.pose.rig.definition,
                        targets,
                    );
                    assert!(
                        separation >= -0.00001,
                        "slope {degrees}, terrain {terrain_heading}, direction {direction_index}, phase {sample}: separation {separation}"
                    );
                }
            }
        }
    }
}

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
    crate::character::terrain_gait::adapt(
        rig.pose.rig.read_frame(),
        contacts,
        &config,
        1.0,
        &mut adapted,
    );
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
    crate::character::terrain_gait::adapt(
        rig.pose.rig.read_frame(),
        contacts,
        &config,
        1.0,
        &mut tilted,
    );
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

#[test]
fn stopping_pair_limits_actual_moment_even_with_zero_net_acceleration() {
    for strength in [0.25, 1.0] {
        let (mut walking, _, _) = slope(0.0, 0.0);
        for _ in 0..60 {
            walking.step();
        }
        let rig = &mut walking.pose.rig;
        for handle in rig.bodies {
            rig.world.bodies[handle].set_linvel(Vector::zeros(), true);
            rig.world.bodies[handle].set_angvel(Vector::zeros(), true);
        }
        rig.world.bodies[rig.bodies[BodyPart::Pelvis.index()]]
            .set_rotation(Rotation::from_axis_angle(&Vector::x_axis(), 0.4), true);
        let mut config = LocomotionConfig::bundled().unwrap();
        config.stopping_moment_cap_nm = 30.0;
        let mut controller = crate::character::walking_controller::WalkingController::new(config);
        controller.state.motor_strength = strength;
        let before = rig.body_states();
        let center = rig.bodies.iter().fold(Vector::zeros(), |sum, h| {
            let body = &rig.world.bodies[*h];
            sum + body.center_of_mass().coords * body.mass()
        }) / 40.0;
        let positions = rig
            .bodies
            .map(|h| rig.world.bodies[h].center_of_mass().coords);
        controller.advance(rig, true);
        let mut total = Vector::zeros();
        let mut absolute = 0.0;
        let mut angular = Vector::zeros();
        for (index, (old, new)) in before.iter().zip(rig.body_states()).enumerate() {
            assert_eq!(old.state.position, new.state.position);
            assert_eq!(old.state.rotation, new.state.rotation);
            let impulse = (Vector::from(new.state.linear_velocity)
                - Vector::from(old.state.linear_velocity))
                * old.state.mass;
            total += impulse;
            absolute += impulse.norm();
            angular += (positions[index] - center).cross(&impulse);
            assert_eq!(impulse.y, 0.0);
        }
        assert!(total.norm() < 0.0001);
        assert!(absolute <= 8.0001);
        let moment = angular / crate::TICK_SECONDS;
        assert!((moment - Vector::from(controller.state.stopping_moment)).norm() < 0.001);
        assert!(moment.norm() > 20.0 * strength);
        assert!(
            moment.norm() <= 30.0 * strength + 0.001,
            "moment {moment:?}, strength {strength}"
        );
    }
}

#[test]
fn stopping_allocator_respects_zero_budgets_and_low_speed_continuity() {
    let (mut walking, _, _) = slope(0.0, 0.0);
    for _ in 0..60 {
        walking.step();
    }
    let contacts = supports(&walking);
    assert!(contacts.iter().all(Option::is_some));
    let config = LocomotionConfig::bundled().unwrap();
    for contacts in [contacts, [contacts[0], None], [None, contacts[1]]] {
        for (acceleration, cap, strength) in
            [(0.0, 120.0, 1.0), (12.0, 0.0, 1.0), (12.0, 120.0, 0.0)]
        {
            let mut tuning = config.clone();
            tuning.maximum_acceleration_m_s2 = acceleration;
            tuning.stopping_moment_cap_nm = cap;
            let drive = crate::character::stopping_drive::allocate(
                walking.pose.rig.read_frame(),
                Vector::zeros(),
                Vector::x(),
                contacts,
                strength,
                &tuning,
            );
            assert_eq!(drive.foot, Vector::zeros());
            assert_eq!(drive.moment, Vector::zeros());
            assert_eq!(drive.net, Vector::zeros());
        }
        let command = |v| {
            crate::character::stopping_drive::allocate(
                walking.pose.rig.read_frame(),
                Vector::zeros(),
                Vector::x() * v,
                contacts,
                1.0,
                &config,
            )
        };
        assert!((command(0.000001).foot - command(-0.000001).foot).norm() < 0.001);
        let low_gravity = crate::character::PassiveRagdoll::new(
            PhysicsConfig {
                gravity_y: -0.1,
                ..PhysicsConfig::default()
            },
            CharacterDefinition::bundled().unwrap(),
            [0.0, 0.08, 0.0],
        )
        .unwrap();
        let drive = crate::character::stopping_drive::allocate(
            low_gravity.read_frame(),
            Vector::zeros(),
            Vector::x(),
            contacts,
            1.0,
            &config,
        );
        assert!(drive.foot.norm() < 0.2);
        assert!(drive.moment.norm() < 10.0);
    }
}
