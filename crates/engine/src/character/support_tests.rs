use super::*;
use crate::PhysicsConfig;

fn fixture(
    surface: ColliderBuilder,
    surface_first: bool,
) -> (PhysicsWorld, RigidBodyHandle, RigidBodyHandle) {
    let mut world = PhysicsWorld::new(&PhysicsConfig {
        gravity_y: 0.0,
        ..PhysicsConfig::default()
    });
    let platform = world
        .bodies
        .insert(RigidBodyBuilder::kinematic_velocity_based());
    let foot = world
        .bodies
        .insert(RigidBodyBuilder::dynamic().translation(vector![0.0, 0.1, 0.0]));
    if surface_first {
        world
            .colliders
            .insert_with_parent(surface.clone(), platform, &mut world.bodies);
    }
    world.colliders.insert_with_parent(
        ColliderBuilder::ball(0.1).mass(1.0),
        foot,
        &mut world.bodies,
    );
    if !surface_first {
        world
            .colliders
            .insert_with_parent(surface, platform, &mut world.bodies);
    }
    world.step();
    (world, foot, platform)
}

fn floor() -> ColliderBuilder {
    ColliderBuilder::cuboid(2.0, 0.1, 2.0).translation(vector![0.0, -0.1, 0.0])
}

#[test]
fn floor_support_is_independent_of_collider_pair_order() {
    for first in [false, true] {
        let (world, foot, _) = fixture(floor(), first);
        assert!(foot_supported(
            &world,
            foot,
            &[foot],
            &BalanceConfig::bundled().unwrap()
        ));
    }
}

#[test]
fn wall_ceiling_sensor_and_own_body_contacts_are_not_ground() {
    let config = BalanceConfig::bundled().unwrap();
    for surface in [
        ColliderBuilder::cuboid(0.1, 2.0, 2.0).translation(vector![0.2, 0.1, 0.0]),
        ColliderBuilder::cuboid(2.0, 0.1, 2.0).translation(vector![0.0, 0.3, 0.0]),
        floor().sensor(true),
    ] {
        let (world, foot, _) = fixture(surface, false);
        assert!(!foot_supported(&world, foot, &[foot], &config));
    }
    let (world, foot, own_body) = fixture(floor(), false);
    assert!(!foot_supported(&world, foot, &[foot, own_body], &config));
}

#[test]
fn stale_contact_does_not_ground_a_departing_or_separated_foot() {
    let config = BalanceConfig::bundled().unwrap();
    let (mut world, foot, platform) = fixture(floor(), true);
    world.bodies[foot].set_linvel(vector![0.0, 2.0, 0.0], true);
    assert!(!foot_supported(&world, foot, &[foot], &config));
    world.bodies[platform].set_linvel(vector![0.0, 2.0, 0.0], true);
    assert!(
        foot_supported(&world, foot, &[foot], &config),
        "matched support velocity must not look like departure"
    );
    world.bodies[foot].set_translation(vector![0.0, 0.3, 0.0], true);
    assert!(!foot_supported(&world, foot, &[foot], &config));
}

#[test]
fn sliding_off_a_finite_ledge_loses_support_but_sliding_over_it_does_not() {
    let config = BalanceConfig::bundled().unwrap();
    let (mut world, foot, _) = fixture(floor(), true);
    world.bodies[foot].set_translation(vector![1.0, 0.1, 0.0], true);
    assert!(foot_supported(&world, foot, &[foot], &config));
    world.bodies[foot].set_translation(vector![3.0, 0.1, 0.0], true);
    assert!(!foot_supported(&world, foot, &[foot], &config));
}

#[test]
fn contact_distance_and_support_normal_thresholds_are_enforced() {
    let config = BalanceConfig::bundled().unwrap();
    let (mut world, foot, _) = fixture(floor(), false);
    world.bodies[foot].set_translation(
        vector![0.0, 0.1 + config.maximum_contact_distance_m * 0.5, 0.0],
        true,
    );
    assert!(foot_supported(&world, foot, &[foot], &config));
    world.bodies[foot].set_translation(
        vector![0.0, 0.1 + config.maximum_contact_distance_m * 2.0, 0.0],
        true,
    );
    assert!(!foot_supported(&world, foot, &[foot], &config));
    for (angle, expected) in [(0.5_f32, true), (1.0, false)] {
        let normal = vector![angle.sin(), angle.cos(), 0.0];
        let surface = ColliderBuilder::cuboid(2.0, 0.1, 2.0)
            .rotation(vector![0.0, 0.0, -angle])
            .translation(vector![0.0, 0.1, 0.0] - normal * 0.2);
        let (world, foot, _) = fixture(surface, true);
        assert_eq!(foot_supported(&world, foot, &[foot], &config), expected);
    }
}

#[test]
fn support_reports_velocity_at_the_current_contact_including_rotation() {
    let (mut world, foot, platform) = fixture(floor(), true);
    world.bodies[foot].set_translation(vector![0.5, 0.1, 0.0], true);
    world.bodies[platform].set_linvel(vector![1.0, 0.0, 0.0], true);
    world.bodies[platform].set_angvel(vector![0.0, 2.0, 0.0], true);
    let support = foot_support(&world, foot, &[foot], &BalanceConfig::bundled().unwrap()).unwrap();
    assert!((support.velocity - vector![1.0, 0.0, -1.0]).norm() < 1e-5);
    assert!(support.normal.y > 0.999);
}

#[test]
fn support_uses_the_solver_friction_material_combination() {
    for (rule, expected) in [
        (CoefficientCombineRule::Average, 0.65),
        (CoefficientCombineRule::Min, 0.5),
        (CoefficientCombineRule::Multiply, 0.4),
        (CoefficientCombineRule::Max, 0.8),
    ] {
        let (world, foot, _) = fixture(floor().friction(0.8).friction_combine_rule(rule), true);
        let support =
            foot_support(&world, foot, &[foot], &BalanceConfig::bundled().unwrap()).unwrap();
        assert!((support.friction - expected).abs() < 1e-6);
    }
}
