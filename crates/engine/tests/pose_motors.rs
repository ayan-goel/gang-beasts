use brawler_engine::{
    BodyPart, CharacterDefinition, MotorConfig, PassiveRagdoll, PhysicsConfig, PoseRagdoll,
};

fn pose_rig() -> PoseRagdoll {
    PoseRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        MotorConfig::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap()
}

#[test]
fn disabled_pose_is_exactly_the_passive_simulation() {
    let mut active = pose_rig();
    active.set_enabled(false);
    let mut passive = PassiveRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        [0.0, 2.0, 0.0],
    )
    .unwrap();
    active
        .queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0])
        .unwrap();
    passive
        .queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0])
        .unwrap();
    for _ in 0..240 {
        active.step();
        passive.step();
        assert_eq!(active.rig().body_states(), passive.body_states());
        assert!(
            active
                .motor_torques()
                .iter()
                .flatten()
                .all(|value| *value == 0.0)
        );
    }
}

#[test]
fn driven_drop_remains_finite_and_each_joint_respects_its_effort_budget() {
    let mut active = pose_rig();
    let config = MotorConfig::bundled().unwrap();
    active
        .queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0])
        .unwrap();
    let mut saw_effort = false;
    let mut max_anchor_error = 0.0_f32;
    for _ in 0..1200 {
        active.step();
        for (joint, torque) in active
            .rig()
            .definition()
            .joints
            .iter()
            .zip(active.motor_torques())
        {
            let cap = config
                .joints
                .iter()
                .find(|motor| motor.child == joint.child)
                .unwrap()
                .max_torque_nm;
            let norm = torque.iter().map(|value| value * value).sum::<f32>().sqrt();
            assert!(norm.is_finite() && norm <= cap + 1e-5, "{norm} > {cap}");
            saw_effort |= norm > 0.1;
        }
        for body in active.rig().body_states() {
            assert!(body.state.to_buffer().iter().all(|value| value.is_finite()));
            assert!(body.state.position[1] > -0.05);
        }
        for joint in active.rig().joint_states() {
            max_anchor_error = max_anchor_error.max(joint.anchor_error);
        }
    }
    assert!(saw_effort);
    assert!(max_anchor_error < 0.05, "anchor error {max_anchor_error}");
    println!("driven drop: maximum anchor separation {max_anchor_error:.6} m");
}

#[test]
fn reset_rebuilds_the_driven_trace_and_clears_pending_input() {
    let mut active = pose_rig();
    let initial = active.rig().body_states();
    let mut trace = Vec::new();
    for _ in 0..120 {
        active.step();
        trace.push(active.rig().body_states());
    }
    active
        .queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0])
        .unwrap();
    active.reset();
    assert_eq!(active.rig().tick(), 0);
    assert_eq!(active.rig().body_states(), initial);
    assert!(
        active
            .motor_torques()
            .iter()
            .flatten()
            .all(|value| *value == 0.0)
    );
    for expected in trace {
        active.step();
        assert_eq!(active.rig().body_states(), expected);
    }
}

#[test]
fn tuning_rejects_nonfinite_unbounded_duplicate_and_foreign_entries() {
    let config = MotorConfig::bundled().unwrap();
    for invalid in [f32::NAN, f32::INFINITY, -1.0, 51.0] {
        let mut changed = config.clone();
        changed.joints[0].max_torque_nm = invalid;
        assert!(changed.validate().is_err());
    }
    let mut changed = config.clone();
    changed.joints[0].child = BodyPart::Pelvis;
    assert!(changed.validate().is_err());
    changed = config.clone();
    changed.joints[0].child = changed.joints[1].child;
    assert!(changed.validate().is_err());
    let json = serde_json::to_string(&config).unwrap();
    assert!(MotorConfig::from_json(&json.replace("explicit_pd", "unknown")).is_err());
    assert!(MotorConfig::from_json(&format!("{json}{}", " ".repeat(8192))).is_err());
    assert!(MotorConfig::from_json(&json.replacen("schema_version", "unknown", 1)).is_err());
}
