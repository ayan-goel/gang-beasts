use brawler_engine::{Duel, FighterId, Hand, HandPhase, PhysicsConfig};

#[test]
fn a_held_punch_scores_once_per_target_and_belongs_to_the_attacking_fighter() {
    for who in [FighterId::First, FighterId::Second] {
        let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
        for _ in 0..60 {
            duel.step();
            assert!(duel.hits().is_empty());
        }
        duel.set_hand(who, Hand::Left, true);
        let mut count = 0;
        for _ in 0..180 {
            duel.step();
            for hit in duel.hits() {
                assert_eq!(hit.attacker, who);
                assert_eq!(hit.defender, who.opponent());
                assert_eq!(hit.source.hand(), Some(Hand::Left));
                assert_eq!(
                    duel.hand_state(who, hit.source.hand().unwrap()).phase,
                    HandPhase::Strike
                );
                assert_eq!(
                    hit.attack_id,
                    duel.hand_state(who, hit.source.hand().unwrap()).attack_id
                );
                assert_eq!(hit.tick, duel.tick());
                assert!(hit.damage > 0.0 && hit.damage <= 35.0);
                assert!(hit.closing_speed > 0.5 && hit.energy_j > 0.2);
                count += 1;
            }
        }
        assert_eq!(count, 1);
        duel.reset();
        assert!(duel.hits().is_empty());
    }
}

#[test]
fn an_out_of_reach_punch_scores_no_hit() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        duel.step();
    }
    duel.set_input(FighterId::Second, [-1., 0.], true).unwrap();
    for _ in 0..100 {
        duel.step();
    }
    duel.set_hand(FighterId::First, Hand::Left, true);
    for _ in 0..60 {
        duel.step();
        assert!(duel.hits().is_empty());
    }
}

#[test]
fn simultaneous_left_hand_attacks_keep_actor_ids_and_do_not_cancel_each_other() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        duel.step();
    }
    for who in [FighterId::First, FighterId::Second] {
        duel.set_hand(who, Hand::Left, true);
        duel.set_hand(who, Hand::Left, false);
    }
    let mut scores = [0; 2];
    for _ in 0..24 {
        duel.step();
        for hit in duel.hits() {
            scores[hit.attacker as usize] += 1;
            assert_eq!(hit.attack_id, 1);
        }
    }
    assert_eq!(scores, [1, 1]);
}
