use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

fn settled() -> LocomotionRagdoll {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.; 3],
    )
    .unwrap();
    for _ in 0..60 {
        rig.step();
    }
    rig
}

fn momentum(rig: &LocomotionRagdoll, axis: usize) -> f32 {
    rig.pose()
        .rig()
        .body_states()
        .iter()
        .map(|b| b.state.mass * b.state.linear_velocity[axis])
        .sum()
}

#[test]
fn short_tap_launches_once_with_the_whole_rig_budget_and_lands_on_both_feet() {
    let mut rig = settled();
    let initial = rig.pose().rig().body_states()[BodyPart::Pelvis.index()]
        .state
        .position[1];
    let before = momentum(&rig, 1);
    rig.set_jump(true);
    rig.set_jump(false);
    rig.step();
    let delivered =
        momentum(&rig, 1) - before - 40. * rig.pose().rig().configuration().gravity_y / 60.;
    assert!((delivered - 180.).abs() < 1., "takeoff impulse {delivered}");
    assert!(rig.state().feet.iter().all(|v| !v));
    let mut peak = initial;
    for _ in 0..180 {
        rig.step();
        let states = rig.pose().rig().body_states();
        peak = peak.max(states[BodyPart::Pelvis.index()].state.position[1]);
        assert!(
            states
                .iter()
                .all(|b| b.state.to_buffer().iter().all(|v| v.is_finite()))
        );
        assert!(
            rig.pose()
                .rig()
                .joint_states()
                .iter()
                .all(|j| j.anchor_error < 0.05)
        );
    }
    assert!(
        (0.85..1.2).contains(&(peak - initial)),
        "apex {peak} from {initial}"
    );
    assert_eq!(rig.state().feet, [true, true]);
    assert!(
        rig.pose().rig().body_states()[BodyPart::Head.index()]
            .state
            .position[1]
            > 1.1
    );
}

#[test]
fn held_input_survives_landing_without_retrigger_and_a_fresh_press_launches() {
    let mut rig = settled();
    for tick in 0..140 {
        rig.set_jump(true);
        rig.step();
        if tick > 80 {
            assert!(momentum(&rig, 1).abs() < 10., "extra launch at {tick}");
        }
    }
    rig.set_jump(false);
    rig.set_jump(true);
    rig.step();
    assert!(momentum(&rig, 1) > 160.);
}

#[test]
fn reset_disable_and_input_cancellation_discard_pending_jump_edges() {
    for action in 0..3 {
        let mut rig = settled();
        rig.set_jump(true);
        match action {
            0 => rig.reset(),
            1 => {
                rig.set_enabled(false);
                rig.set_jump(true);
                rig.set_enabled(true);
            }
            _ => rig.clear_jump_input(),
        }
        for _ in 0..15 {
            rig.step();
            assert!(momentum(&rig, 1) < 20.);
        }
    }
}
