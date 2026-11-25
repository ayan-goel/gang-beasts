use brawler_engine::{FixtureSession, PhysicsConfig};

#[test]
fn exported_commands_reproduce_every_completed_native_state() {
    let mut live = FixtureSession::new(PhysicsConfig::default()).unwrap();
    let mut states = vec![live.simulation().fixture_state()];
    for tick in 1..=240 {
        if [20, 90, 150].contains(&tick) {
            live.queue_impulse([0.0, 4.0, 0.0]).unwrap();
        }
        live.step();
        states.push(live.simulation().fixture_state());
    }
    let json = live.export_recording().unwrap();
    let mut replay = FixtureSession::from_recording(&json).unwrap();
    assert_eq!(replay.replay_end(), Some(240));
    assert_eq!(replay.simulation().fixture_state(), states[0]);
    for state in &states[1..] {
        assert!(replay.step());
        assert_eq!(&replay.simulation().fixture_state(), state);
    }
    assert!(!replay.step());
    assert!(replay.queue_impulse([0.0, 4.0, 0.0]).is_err());
}

#[test]
fn rejected_imports_preserve_the_running_world() {
    let mut session = FixtureSession::new(PhysicsConfig::default()).unwrap();
    for _ in 0..10 {
        session.step();
    }
    let before = session.simulation().fixture_state();
    let valid: serde_json::Value =
        serde_json::from_str(&session.export_recording().unwrap()).unwrap();
    let changes = [
        ("schema_version", serde_json::json!(2)),
        ("engine_hash", serde_json::json!("wrong-build")),
        ("config_hash", serde_json::json!("wrong-config")),
        ("seed", serde_json::json!(1)),
        ("ticks", serde_json::json!(36001)),
        (
            "commands",
            serde_json::json!([{"tick":0,"impulse":[0,4,0]}]),
        ),
        (
            "commands",
            serde_json::json!([{"tick":2,"impulse":[0,4,0]},{"tick":2,"impulse":[0,4,0]}]),
        ),
        (
            "commands",
            serde_json::json!([{"tick":11,"impulse":[0,4,0]}]),
        ),
        (
            "commands",
            serde_json::json!([{"tick":1,"impulse":[11,0,0]}]),
        ),
    ];
    for (key, value) in changes {
        let mut invalid = valid.clone();
        invalid[key] = value;
        assert!(
            session.import_recording(&invalid.to_string()).is_err(),
            "accepted {key}"
        );
        assert_eq!(session.simulation().tick(), 10);
        assert_eq!(session.simulation().fixture_state(), before);
    }
    assert!(session.import_recording(&" ".repeat(262145)).is_err());
    assert!(session.import_recording("{malformed}").is_err());
    let mut unknown = valid.clone();
    unknown["unrecognized"] = serde_json::json!(true);
    assert!(session.import_recording(&unknown.to_string()).is_err());
    let mut too_many = valid.clone();
    too_many["ticks"] = serde_json::json!(2000);
    too_many["commands"] = serde_json::json!(
        (1..=1025)
            .map(|tick| serde_json::json!({"tick":tick,"impulse":[0,0,0]}))
            .collect::<Vec<_>>()
    );
    assert!(session.import_recording(&too_many.to_string()).is_err());
}

#[test]
fn tuning_is_transactional_and_export_uses_the_running_config() {
    let mut session = FixtureSession::new(PhysicsConfig::default()).unwrap();
    session.step();
    assert!(
        session
            .configure(r#"{"schema_version":1,"gravity_y":1,"solver_iterations":8}"#)
            .is_err()
    );
    assert_eq!(session.simulation().tick(), 1);
    let tuned = r#"{"schema_version":1,"gravity_y":-3,"solver_iterations":12}"#;
    session.configure(tuned).unwrap();
    assert_eq!(session.simulation().tick(), 0);
    let exported: serde_json::Value =
        serde_json::from_str(&session.export_recording().unwrap()).unwrap();
    assert_eq!(exported["config"]["gravity_y"], -3.0);
    session.step();
    assert!((session.simulation().fixture_state().linear_velocity[1] + 0.05).abs() < 1e-6);
    assert!(session.configure(&" ".repeat(4097)).is_err());
}

#[test]
fn pending_commands_are_excluded_and_reset_starts_a_fresh_log() {
    let mut session = FixtureSession::new(PhysicsConfig::default()).unwrap();
    session.queue_impulse([0.0, 4.0, 0.0]).unwrap();
    let pending: serde_json::Value =
        serde_json::from_str(&session.export_recording().unwrap()).unwrap();
    assert_eq!(pending["commands"], serde_json::json!([]));
    session.step();
    session.reset();
    let reset: serde_json::Value =
        serde_json::from_str(&session.export_recording().unwrap()).unwrap();
    assert_eq!(reset["ticks"], 0);
    assert_eq!(reset["commands"], serde_json::json!([]));
}

#[test]
fn recording_budgets_never_silently_export_a_truncated_run() {
    let mut session = FixtureSession::new(PhysicsConfig::default()).unwrap();
    for _ in 0..1024 {
        session.queue_impulse([0.0, 0.0, 0.0]).unwrap();
        session.step();
    }
    assert!(session.export_recording().is_ok());
    session.queue_impulse([0.0, 0.0, 0.0]).unwrap();
    session.step();
    assert!(!session.recording_available());
    assert!(session.export_recording().is_err());
    assert!(session.step());
    session.reset();
    assert!(session.recording_available());
    for _ in 0..36_000 {
        session.step();
    }
    assert!(session.export_recording().is_ok());
    session.step();
    assert!(session.export_recording().is_err());
}
