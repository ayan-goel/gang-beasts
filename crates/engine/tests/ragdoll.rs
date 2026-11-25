use brawler_engine::{BodyPart, CharacterDefinition, PassiveRagdoll, PhysicsConfig};

fn make_rig() -> PassiveRagdoll {
    PassiveRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap()
}

#[test]
fn physical_rig_has_exact_mass_topology_and_aligned_anchors() {
    let rig = make_rig();
    assert_eq!(rig.counts(), (15, 16, 14));
    let states = rig.body_states();
    assert_eq!(states.len(), 15);
    assert!((states.iter().map(|body| body.state.mass).sum::<f32>() - 40.0).abs() < 1e-4);
    for (body, expected) in states.iter().zip([
        4., 10., 6., 1.5, 1.5, 1., 1., 0.5, 0.5, 3.5, 3.5, 2.5, 2.5, 1., 1.,
    ]) {
        assert!((body.state.mass - expected).abs() < 1e-5);
    }
    for joint in rig.joint_states() {
        assert!(!joint.contacts_enabled);
        assert!(joint.anchor_error < 1e-6);
    }
}

#[test]
fn passive_fall_remains_finite_and_jointed_without_motors() {
    let mut rig = make_rig();
    rig.queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0]).unwrap();
    let mut largest_error = 0.0_f32;
    for _ in 0..1200 {
        rig.step();
        for body in rig.body_states() {
            assert!(body.state.to_buffer().iter().all(|value| value.is_finite()));
            assert!(
                body.state.position[1] > -0.05,
                "body tunneled through floor"
            );
            assert!(
                body.state
                    .linear_velocity
                    .iter()
                    .all(|speed| speed.abs() < 20.0)
            );
        }
        for joint in rig.joint_states() {
            largest_error = largest_error.max(joint.anchor_error);
        }
    }
    assert!(largest_error < 0.05, "anchor separation {largest_error} m");
    assert!(
        rig.body_states()[0].state.position[1] < 1.0,
        "unpowered character should collapse"
    );
    println!("1200-tick passive fall: maximum anchor error {largest_error:.6} m");
}

#[test]
fn reset_reproduces_the_rig_and_clears_queued_impulses() {
    let mut rig = make_rig();
    let original = rig.body_states();
    for _ in 0..60 {
        rig.step();
    }
    rig.queue_impulse(BodyPart::Chest, [1.0, 0.0, 0.0]).unwrap();
    rig.reset();
    assert_eq!(rig.tick(), 0);
    assert_eq!(rig.body_states(), original);
    let mut fresh = make_rig();
    for _ in 0..120 {
        fresh.step();
        rig.step();
    }
    assert_eq!(rig.body_states(), fresh.body_states());
}

#[test]
fn invalid_character_assets_are_rejected_before_physics_creation() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("../../../config/character.json")).unwrap();
    for (field, value) in [
        ("schema_version", serde_json::json!(2)),
        ("unexpected", serde_json::json!(true)),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        assert!(CharacterDefinition::from_json(&bad.to_string()).is_err());
    }
    for (path, value) in [
        ("/bodies/0/mass", serde_json::json!(5)),
        ("/bodies/1/part", serde_json::json!("head")),
        ("/bodies/0/shape/radius", serde_json::json!(-0.1)),
        ("/joints/0/child", serde_json::json!("pelvis")),
        ("/joints/0/anchor", serde_json::json!([10, 0, 0])),
        ("/joints/0/angular_limits/0", serde_json::json!([1, -1])),
    ] {
        let mut bad = original.clone();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(
            CharacterDefinition::from_json(&bad.to_string()).is_err(),
            "accepted {path}"
        );
    }
    assert!(CharacterDefinition::from_json(&" ".repeat(32769)).is_err());
}
