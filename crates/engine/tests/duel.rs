use brawler_engine::{BodyPart, Duel, FighterId, Hand, HandPhase, PhysicsConfig};

#[test]
fn both_fighters_stand_in_one_world_and_either_can_physically_hit_the_other() {
    for attacker in [FighterId::First, FighterId::Second] {
        let defender = attacker.opponent();
        let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
        for _ in 0..60 {
            duel.step();
        }
        let mut control = Duel::new(PhysicsConfig::default()).unwrap();
        for _ in 0..60 {
            control.step();
        }
        let mut hit = false;
        let mut displacement = 0.0_f32;
        duel.set_hand(attacker, Hand::Left, true);
        duel.set_hand(attacker, Hand::Left, false);
        for _ in 0..30 {
            duel.step();
            control.step();
            assert_eq!(duel.hand_state(defender, Hand::Left).phase, HandPhase::Idle);
            hit |= duel.hand_state(attacker, Hand::Left).phase == HandPhase::Strike
                && duel.hand_contact_impulse(attacker, Hand::Left) > 0.01;
            let struck = duel.body_states(defender);
            let quiet = control.body_states(defender);
            assert!(struck[BodyPart::Chest.index()].state.position[1] > 0.85);
            let delta = (struck[BodyPart::Chest.index()].state.position[2]
                - quiet[BodyPart::Chest.index()].state.position[2])
                .abs();
            displacement = displacement.max(delta);
        }
        assert!(hit, "{attacker:?} missed");
        assert!(displacement > 0.005, "opponent moved only {displacement} m");
        assert_eq!(duel.tick(), 90);
        duel.reset();
        assert_eq!(duel.tick(), 0);
        assert_eq!(duel.hand_state(attacker, Hand::Left).phase, HandPhase::Idle);
        assert_eq!(
            duel.body_states(defender),
            Duel::new(PhysicsConfig::default())
                .unwrap()
                .body_states(defender)
        );
    }
}

#[test]
fn one_fighter_can_jump_while_the_other_walks_away() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        duel.step();
    }
    let pelvis = BodyPart::Pelvis.index();
    let start = duel.body_states(FighterId::First)[pelvis].state.position;
    let second = duel.body_states(FighterId::Second)[pelvis].state.position;
    duel.set_jump(FighterId::First, true);
    duel.set_input(FighterId::Second, [-1., 0.], false).unwrap();
    let mut rise = 0.0_f32;
    for _ in 0..90 {
        duel.step();
        let first_y = duel.body_states(FighterId::First)[pelvis].state.position[1];
        let second_y = duel.body_states(FighterId::Second)[pelvis].state.position[1];
        rise = rise.max(first_y - start[1]);
        assert!(
            second_y < second[1] + 0.15,
            "jump leaked into the other fighter"
        );
    }
    assert!(rise > 0.35);
    assert!(duel.body_states(FighterId::Second)[pelvis].state.position[0] < second[0] - 0.6);
}
