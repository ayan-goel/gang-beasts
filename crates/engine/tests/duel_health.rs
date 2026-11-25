use brawler_engine::{Consciousness, Duel, FighterId, Hand, PhysicsConfig};

#[test]
fn punches_damage_the_defender_and_regeneration_waits_for_two_quiet_seconds() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        duel.step();
    }
    let attacker = FighterId::First;
    let defender = FighterId::Second;
    duel.set_hand(attacker, Hand::Left, true);
    duel.set_hand(attacker, Hand::Left, false);
    for _ in 0..24 {
        duel.step();
        if !duel.hits().is_empty() {
            break;
        }
    }
    let hit = duel.hits()[0];
    let hurt = duel.vitals(defender);
    assert_eq!(hurt.health, 100. - hit.damage);
    assert_eq!(hurt.phase, Consciousness::Active);
    assert_eq!(duel.vitals(attacker).health, 100.);
    for _ in 1..120 {
        duel.step();
        assert_eq!(duel.vitals(defender).health, hurt.health);
    }
    for _ in 0..60 {
        duel.step();
    }
    assert!((duel.vitals(defender).health - (hurt.health + 8.).min(100.)).abs() < 0.001);
    duel.reset();
    assert_eq!(duel.vitals(defender).health, 100.);
}
