use brawler_engine::{PhysicsConfig, Simulation};

#[test]
fn falling_body_settles_on_platform_without_nonfinite_state() {
    let mut simulation = Simulation::new(PhysicsConfig::default()).unwrap();
    assert_eq!(simulation.tick(), 0);
    let initial = simulation.fixture_position();
    for _ in 0..30 {
        simulation.step();
    }
    assert!(simulation.fixture_position()[1] < initial[1] - 0.5);
    for _ in 0..570 {
        simulation.step();
    }
    let settled = simulation.fixture_position();
    assert!(settled.iter().all(|v| v.is_finite()));
    assert!((settled[1] - 0.5).abs() < 0.02, "{settled:?}");
    assert_eq!(simulation.tick(), 600);
}

#[test]
fn rebuilding_same_fixture_reproduces_completed_state() {
    let config = PhysicsConfig::default();
    let mut first = Simulation::new(config.clone()).unwrap();
    let mut second = Simulation::new(config).unwrap();
    for _ in 0..120 {
        first.step();
        second.step();
        assert_eq!(first.fixture_position(), second.fixture_position());
    }
}

#[test]
fn invalid_configuration_is_rejected_before_world_creation() {
    for gravity in [f32::NAN, f32::INFINITY, 0.0, 9.81, -1000.0] {
        let config = PhysicsConfig {
            gravity_y: gravity,
            ..PhysicsConfig::default()
        };
        assert!(Simulation::new(config).is_err(), "accepted {gravity}");
    }
    for iterations in [0, 33, usize::MAX] {
        let config = PhysicsConfig {
            solver_iterations: iterations,
            ..PhysicsConfig::default()
        };
        assert!(Simulation::new(config).is_err());
    }
    let config = PhysicsConfig {
        schema_version: 2,
        ..PhysicsConfig::default()
    };
    assert!(Simulation::new(config).is_err());
}

#[test]
fn committed_configuration_matches_native_defaults() {
    let config = PhysicsConfig::from_json(include_str!("../../../config/physics.json")).unwrap();
    assert_eq!(config, PhysicsConfig::default());
    assert!(
        PhysicsConfig::from_json(
            r#"{"schema_version":1,"gravity_y":-9.81,"solver_iterations":8,"unknown":1}"#
        )
        .is_err()
    );
}
