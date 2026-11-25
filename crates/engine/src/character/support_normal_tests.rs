use super::*;

#[path = "support_normal_boundary_tests.rs"]
mod boundaries;

#[path = "support_normal_frame_tests.rs"]
mod frames;

// Preserve the captured f32 quaternion components: normalizing them changes
// the near-touching rounded-foot query that reproduced the incorrect normal.
fn recorded_foot_and_deck() -> (
    Isometry<Real>,
    Isometry<Real>,
    ColliderBuilder,
    ColliderBuilder,
) {
    let pose = |p, [x, y, z, w]: [f32; 4]| {
        Isometry::from_parts(
            Translation::from(Vector::from(p)),
            Rotation::new_unchecked(rapier3d::na::Quaternion::new(w, x, y, z)),
        )
    };
    (
        pose(
            [-17.426172, -4.5939465, -0.17575285],
            [-0.06326039, 0.5544495, 0.11970738, 0.82112974],
        ),
        pose(
            [0.0, -0.5176381, 0.0],
            [-0.09229596, 0.7010574, -0.09229596, -0.7010574],
        ),
        ColliderBuilder::round_cuboid(0.08, 0.05, 0.14, 0.02),
        ColliderBuilder::cuboid(30.0, 0.5, 30.0),
    )
}

#[test]
fn interior_cuboid_face_has_its_current_world_normal_at_a_micrometre_gap() {
    let (foot_pose, deck_pose, foot_shape, deck_shape) = recorded_foot_and_deck();
    let expected = deck_pose.rotation * Vector::y();
    let mut world = PhysicsWorld::new(&crate::PhysicsConfig {
        gravity_y: 0.0,
        ..Default::default()
    });
    let mut preload = foot_pose;
    preload.translation.vector -= expected * 0.003;
    let foot = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().position(preload));
    let f = world
        .colliders
        .insert_with_parent(foot_shape, foot, &mut world.bodies);
    let deck = world.colliders.insert(deck_shape.position(deck_pose));
    world.step();
    world.bodies[foot].set_angvel(Vector::zeros(), true);
    world.bodies[foot].set_linvel(Vector::zeros(), true);
    let cfg = BalanceConfig::bundled().unwrap();
    for offset in [0.0, -0.0005, 0.002, 0.03] {
        let mut current = foot_pose;
        current.translation.vector += expected * offset;
        world.bodies[foot].set_position(current, true);
        let raw = rapier3d::parry::query::contact(
            &current,
            world.colliders[f].shape(),
            &deck_pose,
            world.colliders[deck].shape(),
            cfg.maximum_contact_distance_m,
        )
        .unwrap();
        let selected = foot_support(&world, foot, &[foot], &cfg);
        if offset > cfg.maximum_contact_distance_m {
            assert!(raw.is_none() && selected.is_none());
            continue;
        }
        let raw = raw.unwrap();
        let local = deck_pose.inverse_transform_point(&raw.point2);
        assert!((raw.dist - offset).abs() < 0.00002);
        assert!(local.x.abs() < 18.0 && local.z.abs() < 18.0 && (local.y - 0.5).abs() < 0.00002);
        let chosen = selected.unwrap();
        assert_eq!(chosen.contact_point, raw.point2);
        assert_eq!(chosen.distance, raw.dist);
        assert!(
            (chosen.normal - expected).norm() < 0.00001,
            "selected={:?} expected={expected:?}",
            chosen.normal
        );
    }
}
