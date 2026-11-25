use super::*;
use crate::{Hand, HandPhase, PhysicsConfig};

#[test]
fn a_physical_hit_disables_motor_drive_and_actions_until_the_wake_ramp_finishes() {
    let mut d = Duel::new(PhysicsConfig::default()).unwrap();
    let victim = FighterId::Second;
    for _ in 0..60 {
        d.step();
    }
    d.vitals[victim as usize].advance(d.tick, 99.9, &d.health_rules);
    d.set_hand(FighterId::First, Hand::Left, true);
    for _ in 0..24 {
        d.step();
        if d.vitals(victim).phase == Consciousness::KnockedOut {
            break;
        }
    }
    assert_eq!(
        d.consciousness_changes(),
        &[(victim, Consciousness::KnockedOut)]
    );
    let since = d.tick();
    let joints = d.fighters[1]
        .parts(&d.definition, &d.physics, d.tick)
        .joints;
    d.set_input(victim, [1., 0.], true).unwrap();
    d.set_lift(victim, true);
    d.set_jump(victim, true);
    d.set_hand(victim, Hand::Right, true);
    for _ in 0..180 {
        d.step();
        assert_eq!(d.hand_state(victim, Hand::Right).phase, HandPhase::Idle);
        for j in joints {
            assert!(
                d.world
                    .impulse_joints
                    .get(j)
                    .unwrap()
                    .data
                    .motor_axes
                    .is_empty()
            );
        }
    }
    assert_eq!(
        d.consciousness_changes(),
        &[(victim, Consciousness::Recovering)]
    );
    let cap = |d: &Duel| {
        joints
            .iter()
            .flat_map(|&h| d.world.impulse_joints.get(h).unwrap().data.motors)
            .filter(|m| m.stiffness != 0. || m.damping != 0.)
            .map(|m| m.max_force)
            .fold(0., f32::max)
    };
    d.step();
    let first = cap(&d);
    assert!(first > 0.);
    for _ in 1..30 {
        d.step();
    }
    assert!(cap(&d) > first * 10.);
    assert_eq!(d.tick(), since + 210);
    assert_eq!(
        d.consciousness_changes(),
        &[(victim, Consciousness::Active)]
    );
    assert!(!d.lifting[1]);
    assert_eq!(d.hand_state(victim, Hand::Right).phase, HandPhase::Idle);
    d.set_hand(victim, Hand::Right, true);
    d.step();
    assert_eq!(d.hand_state(victim, Hand::Right).phase, HandPhase::WindUp);
}

#[test]
fn losing_consciousness_releases_only_outgoing_grips_without_changing_body_state() {
    let mut d = Duel::new(PhysicsConfig::default()).unwrap();
    d.set_input(FighterId::Second, [0., 1.], false).unwrap();
    for id in [FighterId::First, FighterId::Second] {
        d.set_hand(id, Hand::Left, true);
    }
    for _ in 0..25 {
        d.step();
    }
    for id in [FighterId::First, FighterId::Second] {
        assert!(d.grip(id, Hand::Left).is_some());
    }
    let victim = FighterId::Second;
    let before = d.body_states(victim);
    // Inject a completed hit at the resolver boundary to isolate grip ownership from punch geometry.
    d.hits = vec![crate::StrikeHit {
        tick: d.tick,
        attacker: FighterId::First,
        defender: victim,
        hand: Hand::Right,
        attack_id: 2,
        region: crate::BodyPart::Chest,
        closing_speed: 3.,
        energy_j: 5.,
        damage: 30.,
    }];
    d.vitals[1].advance(d.tick - 1, 55., &d.health_rules);
    d.resolve_health();
    assert_eq!(d.vitals(victim).phase, Consciousness::KnockedOut);
    assert!(d.grip(victim, Hand::Left).is_none());
    assert!(d.grip(FighterId::First, Hand::Left).is_some());
    assert_eq!(before, d.body_states(victim));
    d.reset();
    assert_eq!(d.vitals(victim).knockouts, 0);
    assert!(d.consciousness_changes().is_empty());
}

#[test]
fn simultaneous_real_punches_can_knock_out_both_fighters_on_the_same_tick() {
    let mut d = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        d.step();
    }
    for id in [FighterId::First, FighterId::Second] {
        d.vitals[id as usize].advance(d.tick, 99., &d.health_rules);
        d.set_hand(id, Hand::Left, true);
        d.set_hand(id, Hand::Left, false);
    }
    for _ in 0..24 {
        d.step();
        if !d.consciousness_changes().is_empty() {
            break;
        }
    }
    assert_eq!(d.hits().len(), 2);
    assert_eq!(
        d.consciousness_changes(),
        &[
            (FighterId::First, Consciousness::KnockedOut),
            (FighterId::Second, Consciousness::KnockedOut)
        ]
    );
    d.step();
    assert!(d.consciousness_changes().is_empty());
}
