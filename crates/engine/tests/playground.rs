use brawler_engine::{PhysicsConfig, Simulation};

#[test]
fn impulse_is_applied_once_at_the_next_tick_and_has_a_bounded_queue() {
    let mut world = Simulation::new(PhysicsConfig::default()).unwrap();
    world.queue_fixture_impulse([2.0, 4.0, 0.0]).unwrap();
    assert_eq!(world.fixture_state().linear_velocity, [0.0; 3]);
    assert!(world.queue_fixture_impulse([0.0, 1.0, 0.0]).is_err());
    world.step();
    let first = world.fixture_state();
    assert!((first.linear_velocity[0] - 2.0).abs() < 1e-5);
    assert!((first.linear_velocity[1] - (4.0 - 9.81 / 60.0)).abs() < 1e-5);
    world.step();
    assert!((world.fixture_state().linear_velocity[0] - 2.0).abs() < 1e-5);
}

#[test]
fn invalid_impulses_do_not_mutate_or_fill_the_command_slot() {
    let mut world = Simulation::new(PhysicsConfig::default()).unwrap();
    for impulse in [
        [f32::NAN, 0.0, 0.0],
        [0.0, f32::INFINITY, 0.0],
        [8.0, 8.0, 0.0],
    ] {
        assert!(world.queue_fixture_impulse(impulse).is_err());
        assert_eq!(world.tick(), 0);
        assert_eq!(world.fixture_state().linear_velocity, [0.0; 3]);
    }
    world.queue_fixture_impulse([0.0, 4.0, 0.0]).unwrap();
}

#[test]
fn reset_restores_state_and_discards_pending_impulses() {
    let mut world = Simulation::new(PhysicsConfig::default()).unwrap();
    for _ in 0..90 {
        world.step();
    }
    world.queue_fixture_impulse([0.0, 4.0, 0.0]).unwrap();
    world.reset();
    let mut fresh = Simulation::new(PhysicsConfig::default()).unwrap();
    assert_eq!(world.tick(), 0);
    assert_eq!(world.fixture_state(), fresh.fixture_state());
    for _ in 0..120 {
        world.step();
        fresh.step();
        assert_eq!(world.fixture_state(), fresh.fixture_state());
    }
}

#[test]
fn debug_lines_follow_real_collider_transforms() {
    let mut world = Simulation::new(PhysicsConfig::default()).unwrap();
    assert_eq!(world.fixture_state().rotation, [0.0, 0.0, 0.0, 1.0]);
    let initial = world.collider_lines();
    assert_eq!(initial.len(), 24 * 6);
    assert_eq!(
        initial.iter().step_by(3).copied().fold(f32::MIN, f32::max),
        6.0
    );
    let highest = |lines: Vec<f32>| {
        lines
            .iter()
            .skip(1)
            .step_by(3)
            .copied()
            .fold(f32::MIN, f32::max)
    };
    assert!((highest(initial) - 4.5).abs() < 1e-5);
    for _ in 0..600 {
        world.step();
    }
    assert!((highest(world.collider_lines()) - 1.0).abs() < 0.02);
    assert_eq!(world.fixture_state().mass, 1.0);
    let rotation = world.fixture_state().rotation;
    assert!((rotation.iter().map(|value| value * value).sum::<f32>() - 1.0).abs() < 1e-5);
    assert!(rotation[..3].iter().all(|value| value.abs() < 1e-4));
}
