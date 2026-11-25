use brawler_engine::{CharacterDefinition, PassiveRagdoll, PhysicsConfig};

#[test]
fn version_one_rejects_wrong_collider_kinds_and_anatomical_axes() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../../../config/character.json")).unwrap();
    for (path, value) in [
        (
            "/bodies/0/shape",
            serde_json::json!({"kind":"capsule","radius":0.18,"half_height":0.05}),
        ),
        (
            "/bodies/1/shape",
            serde_json::json!({"kind":"sphere","radius":0.18}),
        ),
        (
            "/bodies/13/shape",
            serde_json::json!({"kind":"sphere","radius":0.14}),
        ),
        (
            "/joints/4/angular_limits",
            serde_json::json!([[-1, 1], [-1, 1], [-1, 1]]),
        ),
        (
            "/joints/10/angular_limits",
            serde_json::json!([[0, 0], [-1, 0], [0, 0]]),
        ),
        (
            "/joints/6/angular_limits",
            serde_json::json!([[-1, 1], [0, 0], [0, 0]]),
        ),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            CharacterDefinition::from_json(&bad.to_string()).is_err(),
            "accepted {path}"
        );
    }
}

#[test]
fn spawn_validation_checks_body_geometry_above_the_platform() {
    let mut shifted = CharacterDefinition::bundled().unwrap();
    for body in &mut shifted.bodies {
        body.position[1] -= 1.4;
    }
    for joint in &mut shifted.joints {
        joint.anchor[1] -= 1.4;
    }
    assert!(PassiveRagdoll::new(PhysicsConfig::default(), shifted, [0.0, 0.0, 0.0]).is_err());
    assert!(
        PassiveRagdoll::new(
            PhysicsConfig::default(),
            CharacterDefinition::bundled().unwrap(),
            [0.0, 0.0, 0.0]
        )
        .is_ok()
    );
    assert!(
        PassiveRagdoll::new(
            PhysicsConfig::default(),
            CharacterDefinition::bundled().unwrap(),
            [f32::NAN, 0.0, 0.0]
        )
        .is_err()
    );
}
