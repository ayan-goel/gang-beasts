use super::*;

#[test]
fn diagonal_edge_corner_and_curved_contact_normals_survive_selection() {
    let cfg = BalanceConfig::bundled().unwrap();
    for (label, builder, axis_pos, axis, radius) in [
        (
            "edge",
            ColliderBuilder::cuboid(2.0, 0.1, 2.0),
            vector![2.0, 0.1, 0.0],
            vector![0.3, 0.9539392, 0.0],
            0.0,
        ),
        (
            "corner",
            ColliderBuilder::cuboid(2.0, 0.1, 2.0),
            vector![2.0, 0.1, 2.0],
            vector![0.24, 0.9327379, 0.26981476],
            0.0,
        ),
        (
            "ball",
            ColliderBuilder::ball(2.0),
            Vector::zeros(),
            vector![0.3, 0.9539392, 0.0],
            2.0,
        ),
        (
            "rounded shoulder",
            ColliderBuilder::round_cuboid(2.0, 0.1, 2.0, 0.2),
            vector![2.0, 0.1, 0.0],
            vector![0.3, 0.9539392, 0.0],
            0.2,
        ),
    ] {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig {
            gravity_y: 0.0,
            ..Default::default()
        });
        let axis = axis.normalize();
        let center = axis_pos + axis * (0.1 + radius);
        let foot = world
            .bodies
            .insert(RigidBodyBuilder::dynamic().translation(center - axis * 0.002));
        let f =
            world
                .colliders
                .insert_with_parent(ColliderBuilder::ball(0.1), foot, &mut world.bodies);
        let platform = world.colliders.insert(builder);
        world.step();
        world.bodies[foot].set_translation(center, true);
        world.bodies[foot].set_angvel(Vector::zeros(), true);
        world.bodies[foot].set_linvel(Vector::zeros(), true);
        let raw = rapier3d::parry::query::contact(
            world.bodies[foot].position(),
            world.colliders[f].shape(),
            &Isometry::identity(),
            world.colliders[platform].shape(),
            cfg.maximum_contact_distance_m,
        )
        .unwrap()
        .unwrap();
        let selected = foot_support(&world, foot, &[foot], &cfg).unwrap();
        assert_eq!(selected.normal, *raw.normal2, "{label}");
        assert!(
            selected.normal.dot(&axis) > 0.999,
            "{label}: {:?}",
            selected.normal
        );
    }
}

#[test]
fn translated_rotated_cuboid_edge_retains_diagonal_contact_and_departure() {
    let pose = Isometry::from_parts(
        Translation::new(64.0, -23.68, 46.72),
        Rotation::new_unchecked(rapier3d::na::Quaternion::new(
            0.97935784,
            0.021178765,
            0.1575428,
            0.124859795,
        )),
    );
    let foot_center = pose.transform_point(&Point::new(0.53, 0.1 + 0.09539392, 0.31));
    let approach = pose.rotation * vector![0.3, 0.9539392, 0.0];
    let cfg = BalanceConfig::bundled().unwrap();
    let mut world = PhysicsWorld::new(&crate::PhysicsConfig {
        gravity_y: 0.0,
        ..Default::default()
    });
    let foot = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().translation(foot_center.coords - approach * 0.002));
    let f = world
        .colliders
        .insert_with_parent(ColliderBuilder::ball(0.1), foot, &mut world.bodies);
    let deck = world
        .colliders
        .insert(ColliderBuilder::cuboid(0.5, 0.1, 0.75).position(pose));
    world.step();
    world.bodies[foot].set_translation(foot_center.coords, true);
    world.bodies[foot].set_angvel(Vector::zeros(), true);
    world.bodies[foot].set_linvel(Vector::zeros(), true);
    let raw = rapier3d::parry::query::contact(
        world.bodies[foot].position(),
        world.colliders[f].shape(),
        &pose,
        world.colliders[deck].shape(),
        cfg.maximum_contact_distance_m,
    )
    .unwrap()
    .unwrap();
    let normal = *raw.normal2;
    let local = pose.inverse_transform_point(&raw.point2);
    assert!((local - Point::new(0.5, 0.1, 0.31)).norm() < 0.00002);
    assert!(normal.dot(&approach) > 0.999);
    assert_eq!(
        foot_support(&world, foot, &[foot], &cfg).unwrap().normal,
        normal
    );
    world.bodies[foot].set_linvel(normal * 0.51, true);
    assert!(foot_support(&world, foot, &[foot], &cfg).is_none());
}
