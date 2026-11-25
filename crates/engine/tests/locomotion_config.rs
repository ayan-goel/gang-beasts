use brawler_engine::{LocomotionConfig, MotorConfig};

#[test]
fn locomotion_schema_two_requires_an_explicit_stopping_moment_limit() {
    let config = serde_json::to_value(LocomotionConfig::bundled().unwrap()).unwrap();
    assert_eq!(config["schema_version"], 2);
    assert_eq!(config["stopping_moment_cap_nm"], 120.0);
    let mut legacy = config.clone();
    legacy["schema_version"] = serde_json::json!(1);
    assert!(LocomotionConfig::from_json(&legacy.to_string()).is_err());
    let mut missing = config.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("stopping_moment_cap_nm");
    assert!(LocomotionConfig::from_json(&missing.to_string()).is_err());
    for value in [0.0, 120.0] {
        let mut valid = LocomotionConfig::bundled().unwrap();
        valid.stopping_moment_cap_nm = value;
        valid.validate().unwrap();
    }
    for value in [f32::NAN, f32::INFINITY] {
        let mut invalid = LocomotionConfig::bundled().unwrap();
        invalid.stopping_moment_cap_nm = value;
        assert!(invalid.validate().is_err());
    }
    let mut excessive = config;
    excessive["stopping_moment_cap_nm"] = serde_json::json!(120.01);
    assert!(LocomotionConfig::from_json(&excessive.to_string()).is_err());
}

#[test]
fn locomotion_tuning_rejects_unbounded_unknown_and_oversized_values() {
    let base = serde_json::to_value(LocomotionConfig::bundled().unwrap()).unwrap();
    for key in base.as_object().unwrap().keys() {
        for value in [
            serde_json::json!(-1),
            serde_json::json!(1e20),
            serde_json::json!("invalid"),
        ] {
            let mut invalid = base.clone();
            invalid[key] = value;
            assert!(
                LocomotionConfig::from_json(&invalid.to_string()).is_err(),
                "accepted {key}"
            );
        }
    }
    let mut invalid = base.clone();
    invalid["unknown"] = serde_json::json!(0);
    assert!(LocomotionConfig::from_json(&invalid.to_string()).is_err());
    let mut invalid = base.clone();
    invalid["run_speed_m_s"] = serde_json::json!(1.0);
    assert!(LocomotionConfig::from_json(&invalid.to_string()).is_err());
    assert!(LocomotionConfig::from_json(&format!("{}{}", base, " ".repeat(4096))).is_err());
    let mut invalid = LocomotionConfig::bundled().unwrap();
    invalid.drive_tilt_gain = f32::NAN;
    assert!(invalid.validate().is_err());
}

#[test]
fn stronger_implicit_leg_motors_keep_finite_caps_and_explicit_limits() {
    let active = MotorConfig::locomotion().unwrap();
    active.validate().unwrap();
    let mut explicit = active.clone();
    explicit.model = "explicit_pd".into();
    assert!(explicit.validate().is_err());
    for value in [151.0, f32::NAN, f32::INFINITY, -1.0] {
        let mut invalid = active.clone();
        invalid.joints[8].max_torque_nm = value;
        assert!(invalid.validate().is_err());
    }
    let mut invalid = active.clone();
    invalid.joints[8].stiffness_nm_per_rad = 601.0;
    assert!(invalid.validate().is_err());
    let mut invalid = active;
    invalid.joints[6].max_torque_nm = 1.0;
    assert!(invalid.validate().is_err());
}

#[test]
fn settling_tuning_cannot_require_an_impossible_strict_endpoint() {
    let base = serde_json::to_value(LocomotionConfig::bundled().unwrap()).unwrap();
    for (key, value) in [
        ("settle_speed_m_s", 0.0),
        ("settle_spin_rad_s", 0.0),
        ("settle_minimum_up_y", 1.0),
    ] {
        let mut invalid = base.clone();
        invalid[key] = serde_json::json!(value);
        assert!(
            LocomotionConfig::from_json(&invalid.to_string()).is_err(),
            "accepted impossible settling {key}"
        );
    }
}
