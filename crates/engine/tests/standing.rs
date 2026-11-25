use brawler_engine::{
    BalanceConfig, BodyPart, CharacterDefinition, MotorConfig, PhysicsConfig, PoseRagdoll,
};

fn standing(spawn: [f32; 3]) -> PoseRagdoll {
    let mut rig = PoseRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        MotorConfig::standing().unwrap(),
        spawn,
    )
    .unwrap();
    rig.configure_balance(Some(BalanceConfig::bundled().unwrap()))
        .unwrap();
    rig
}

#[test]
fn idle_character_stands_for_sixty_seconds_without_transform_overrides() {
    let mut rig = standing([0.0; 3]);
    let mut max_anchor_error = 0.0_f32;
    let mut min_head_height = f32::MAX;
    let mut supported_ticks = 0;
    let motors = MotorConfig::standing().unwrap();
    for _ in 0..3600 {
        rig.step();
        let states = rig.rig().body_states();
        for body in &states {
            assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
        }
        let head = &states[BodyPart::Head.index()].state;
        min_head_height = min_head_height.min(head.position[1]);
        assert!(
            head.position[1] > 1.15,
            "fell at tick {}: {:?}",
            rig.rig().tick(),
            head.position
        );
        assert!(states[BodyPart::Pelvis.index()].state.position[1] > 0.65);
        assert!(head.position[0].abs() < 0.2 && head.position[2].abs() < 0.2);
        for (joint, torque) in rig
            .rig()
            .definition()
            .joints
            .iter()
            .zip(rig.motor_torques())
        {
            let cap = motors
                .joints
                .iter()
                .find(|motor| motor.child == joint.child)
                .unwrap()
                .max_torque_nm;
            assert!(torque.iter().map(|v| v * v).sum::<f32>().sqrt() <= cap + 1e-4);
        }
        for joint in rig.rig().joint_states() {
            max_anchor_error = max_anchor_error.max(joint.anchor_error);
        }
        let balance = rig.balance_state().unwrap();
        supported_ticks += usize::from(balance.feet.iter().any(|foot| *foot));
        assert!(balance.torque.iter().map(|v| v * v).sum::<f32>().sqrt() <= 30.0 + 1e-5);
    }
    assert!(supported_ticks > 3500);
    assert!(max_anchor_error < 0.05);
    println!(
        "standing: minimum head {min_head_height:.6} m, maximum anchor error {max_anchor_error:.6} m, supported ticks {supported_ticks}"
    );
}

#[test]
fn airborne_character_receives_no_world_balance_assistance() {
    let mut rig = standing([0.0, 2.0, 0.0]);
    for _ in 0..20 {
        rig.step();
        let balance = rig.balance_state().unwrap();
        assert_eq!(balance.feet, [false; 2]);
        assert_eq!(balance.torque, [0.0; 3]);
    }
}

#[test]
fn meaningful_pushes_can_topple_the_standing_character() {
    let mut rig = standing([0.0; 3]);
    for _ in 0..120 {
        rig.step();
    }
    for _ in 0..3 {
        rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])
            .unwrap();
        rig.step();
    }
    let mut min_head_height = f32::MAX;
    for _ in 0..480 {
        rig.step();
        assert!(
            rig.rig()
                .body_states()
                .iter()
                .flat_map(|body| body.state.to_buffer())
                .all(|value| value.is_finite())
        );
        min_head_height = min_head_height.min(
            rig.rig().body_states()[BodyPart::Head.index()]
                .state
                .position[1],
        );
    }
    assert!(
        min_head_height < 0.8,
        "controller resisted the push indefinitely: {min_head_height}"
    );
}

#[test]
fn reset_reproduces_standing_and_disabling_removes_every_controller() {
    let mut first = standing([0.0; 3]);
    let mut trace = Vec::new();
    for _ in 0..120 {
        first.step();
        trace.push((first.rig().body_states(), first.balance_state().cloned()));
    }
    first
        .queue_impulse(BodyPart::Head, [10.0, 0.0, 0.0])
        .unwrap();
    first.reset();
    assert_eq!(first.balance_state().unwrap().torque, [0.0; 3]);
    for (bodies, balance) in trace {
        first.step();
        assert_eq!(first.rig().body_states(), bodies);
        assert_eq!(first.balance_state(), balance.as_ref());
    }
    first.set_enabled(false);
    first.reset();
    let mut passive = brawler_engine::PassiveRagdoll::new(
        PhysicsConfig::default(),
        CharacterDefinition::bundled().unwrap(),
        [0.0; 3],
    )
    .unwrap();
    for _ in 0..120 {
        first.step();
        passive.step();
        assert_eq!(first.rig().body_states(), passive.body_states());
        assert!(
            first
                .motor_torques()
                .iter()
                .flatten()
                .all(|value| *value == 0.0)
        );
        assert_eq!(first.balance_state().unwrap().torque, [0.0; 3]);
    }
}

#[test]
fn invalid_balance_tuning_preserves_the_running_controller() {
    let mut rig = standing([0.0; 3]);
    rig.step();
    let before = rig.balance_state().cloned();
    let bodies = rig.rig().body_states();
    for value in [f32::NAN, f32::INFINITY, -1.0, 51.0] {
        let mut config = BalanceConfig::bundled().unwrap();
        config.max_torque_nm = value;
        assert!(rig.configure_balance(Some(config)).is_err());
        assert_eq!(rig.balance_state(), before.as_ref());
        assert_eq!(rig.rig().body_states(), bodies);
    }
    let json = serde_json::to_string(&BalanceConfig::bundled().unwrap()).unwrap();
    assert!(BalanceConfig::from_json(&json.replace("minimum_normal_y", "unknown")).is_err());
    assert!(BalanceConfig::from_json(&format!("{json}{}", " ".repeat(4096))).is_err());
    rig.configure_balance(None).unwrap();
    assert!(rig.balance_state().is_none());
}
