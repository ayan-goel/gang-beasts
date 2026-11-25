use super::*;

#[test]
fn independently_rotated_parent_and_local_colliders_use_current_normals() {
    let cfg = BalanceConfig::bundled().unwrap();
    let mut world = PhysicsWorld::new(&crate::PhysicsConfig {
        gravity_y: 0.0,
        ..Default::default()
    });
    let parent = world
        .bodies
        .insert(RigidBodyBuilder::kinematic_velocity_based());
    let local = Isometry::from_parts(
        Translation::new(0.21, -0.4, 0.33),
        Rotation::from_axis_angle(&Vector::z_axis(), 0.22),
    );
    let original = Isometry::from_parts(
        Translation::new(1.0, 0.2, 0.1),
        Rotation::from_axis_angle(&Vector::x_axis(), -0.44),
    );
    let moved = Isometry::from_parts(
        Translation::new(0.3, 0.7, -0.2),
        Rotation::from_axis_angle(&Vector::x_axis(), 0.5),
    );
    world.bodies[parent].set_position(original, true);
    let deck = world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(2.0, 0.1, 2.0).position(local),
        parent,
        &mut world.bodies,
    );
    let foot_offset = vector![0.1, -0.08, 0.06];
    let foot_r = Rotation::from_axis_angle(&Vector::z_axis(), -0.3);
    let foot_pose = |p: Isometry<Real>, gap: f32| {
        let center = p.transform_point(&Point::new(0.3, 0.2 + gap, 0.6));
        Isometry::from_parts(
            Translation::from(center.coords - foot_r * foot_offset),
            foot_r,
        )
    };
    let foot = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().position(foot_pose(original * local, -0.002)));
    world.colliders.insert_with_parent(
        ColliderBuilder::ball(0.1).translation(foot_offset),
        foot,
        &mut world.bodies,
    );
    world.step();
    world.bodies[parent].set_position(moved, true);
    let present = moved * local;
    let up = present.rotation * Vector::y();
    let old = world.colliders[deck].position().rotation * Vector::y();
    assert!((old - up).norm() > 0.4);
    world.bodies[foot].set_position(foot_pose(present, 0.002), true);
    world.bodies[foot].set_angvel(Vector::zeros(), true);
    world.bodies[foot].set_linvel(Vector::zeros(), true);
    let s = foot_support(&world, foot, &[foot], &cfg).unwrap();
    assert!((s.normal - up).norm() < 1.0e-5);
    assert!((s.distance - 0.002).abs() < 1.0e-5);
    world.bodies[foot].set_linvel(up * 0.55, true);
    assert!(foot_support(&world, foot, &[foot], &cfg).is_none());
    world.bodies[parent].set_linvel(up * 0.1, true);
    assert!(foot_support(&world, foot, &[foot], &cfg).is_some());
}

#[test]
fn recorded_normal_is_used_for_departure_and_tangent_velocity() {
    let (foot_pose, deck_pose, foot_shape, deck_shape) = recorded_foot_and_deck();
    let up = deck_pose.rotation * Vector::y();
    let mut world = PhysicsWorld::new(&crate::PhysicsConfig {
        gravity_y: 0.0,
        ..Default::default()
    });
    let mut pre = foot_pose;
    pre.translation.vector -= up * 0.003;
    let foot = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().position(pre));
    world
        .colliders
        .insert_with_parent(foot_shape, foot, &mut world.bodies);
    world.colliders.insert(deck_shape.position(deck_pose));
    world.step();
    world.bodies[foot].set_position(foot_pose, true);
    world.bodies[foot].set_angvel(Vector::zeros(), true);
    let cfg = BalanceConfig::bundled().unwrap();
    world.bodies[foot].set_linvel(up * 0.55, true);
    assert!(foot_support(&world, foot, &[foot], &cfg).is_none());
    world.bodies[foot].set_linvel(-Vector::z() * 1.1, true);
    assert!(foot_support(&world, foot, &[foot], &cfg).is_some());
}

#[test]
fn feature_margin_preserves_both_sides_of_a_rotated_top_edge() {
    let shape = ColliderBuilder::cuboid(2., 0.1, 1.).build();

    let pose = Isometry::from_parts(
        Translation::new(4., -2., 6.),
        Rotation::from_euler_angles(0.11, 0.27, -0.15),
    );
    let fallback = pose.rotation * vector![0.3, 0.9539392, 0.0];
    let good = pose.rotation * Vector::y();
    for (dx, interior) in [
        (-0.000001, false),
        (0.0, false),
        (0.000001, false),
        (0.005, true),
    ] {
        let p = pose.transform_point(&Point::new(2. - dx, 0.1, 0.35));
        let n = contact_normal(&shape, &pose, p, fallback);
        assert!(
            (n - if interior { good } else { fallback }).norm() < 1e-5,
            "dx={dx}"
        );
    }
}
