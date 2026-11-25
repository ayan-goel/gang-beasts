use brawler_engine::{Duel, FighterId, Hand, HandPhase, PhysicsConfig};

fn settled() -> Duel {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        duel.step();
    }
    duel
}

#[test]
fn held_hand_attaches_to_opponent_and_release_preserves_both_rigs_momentum() {
    let who = FighterId::First;
    let mut duel = settled();
    duel.set_hand(who, Hand::Left, true);
    for _ in 0..24 {
        duel.step();
        assert!(
            duel.grip(who, Hand::Left).is_none(),
            "gripped while punching"
        );
    }
    for _ in 0..100 {
        duel.step();
        if duel.grip(who, Hand::Left).is_some() {
            break;
        }
    }
    let grip = duel
        .grip(who, Hand::Left)
        .expect("held hand reached the opponent");
    assert_eq!(grip.target, who.opponent());
    assert_eq!(duel.hand_state(who, Hand::Left).phase, HandPhase::Gripping);
    duel.set_hand(who, Hand::Right, true);
    duel.set_hand(who, Hand::Right, false);
    for _ in 0..7 {
        duel.step();
    }
    assert_eq!(duel.hand_state(who, Hand::Right).phase, HandPhase::Strike);
    assert_eq!(
        duel.grip(who, Hand::Left).unwrap().created_tick,
        grip.created_tick
    );
    let before = [duel.body_states(who), duel.body_states(who.opponent())];
    duel.set_hand(who, Hand::Left, false);
    assert!(duel.grip(who, Hand::Left).is_none());
    assert_eq!(
        before,
        [duel.body_states(who), duel.body_states(who.opponent())]
    );
}

#[test]
fn a_tap_or_an_out_of_reach_hold_cannot_grab() {
    let mut duel = settled();
    let who = FighterId::Second;
    duel.set_hand(who, Hand::Left, true);
    duel.set_hand(who, Hand::Left, false);
    for _ in 0..60 {
        duel.step();
        assert!(duel.grip(who, Hand::Left).is_none());
    }
    duel.set_input(who, [1., 0.], true).unwrap();
    for _ in 0..90 {
        duel.step();
    }
    duel.set_hand(who, Hand::Right, true);
    for _ in 0..40 {
        duel.step();
        assert!(duel.grip(who, Hand::Right).is_none());
    }
}

#[test]
fn reciprocal_grips_remain_finite_and_clearing_one_player_keeps_the_incoming_grip() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    duel.set_input(FighterId::Second, [0., 1.], false).unwrap();
    for player in [FighterId::First, FighterId::Second] {
        duel.set_hand(player, Hand::Left, true);
    }
    for _ in 0..240 {
        duel.step();
        for player in [FighterId::First, FighterId::Second] {
            for body in duel.body_states(player) {
                assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
            }
        }
        if duel.grip(FighterId::First, Hand::Left).is_some()
            && duel.grip(FighterId::Second, Hand::Left).is_some()
        {
            break;
        }
    }
    duel.set_input(FighterId::Second, [0., 0.], false).unwrap();
    let incoming = duel
        .grip(FighterId::Second, Hand::Left)
        .expect("Blue acquired an incoming grip");
    assert!(duel.grip(FighterId::First, Hand::Left).is_some());
    for _ in 0..60 {
        duel.step();
        for player in [FighterId::First, FighterId::Second] {
            assert!(duel.grip(player, Hand::Left).is_some());
            for body in duel.body_states(player) {
                assert!(body.state.to_buffer().iter().all(|v| v.is_finite()));
            }
        }
    }
    assert_eq!(
        duel.grip(FighterId::Second, Hand::Left)
            .unwrap()
            .created_tick,
        incoming.created_tick
    );
    duel.clear_hands(FighterId::First);
    assert!(duel.grip(FighterId::First, Hand::Left).is_none());
    assert!(duel.grip(FighterId::Second, Hand::Left).is_some());
    duel.reset();
    assert!(duel.grip(FighterId::Second, Hand::Left).is_none());
}

#[test]
fn a_grip_physically_restrains_the_retreating_opponent() {
    let run = |keep: bool| {
        let mut duel = settled();
        duel.set_hand(FighterId::First, Hand::Left, true);
        for _ in 0..100 {
            duel.step();
            if duel.grip(FighterId::First, Hand::Left).is_some() {
                break;
            }
        }
        let grip = duel.grip(FighterId::First, Hand::Left).unwrap();
        if !keep {
            duel.set_hand(FighterId::First, Hand::Left, false);
        }
        duel.set_input(FighterId::Second, [0., -1.], true).unwrap();
        for _ in 0..30 {
            duel.step();
        }
        let hand = duel.body_states(FighterId::First)[brawler_engine::BodyPart::LeftHand.index()]
            .state
            .position;
        let target = duel.body_states(FighterId::Second)[grip.region.index()]
            .state
            .position;
        hand.into_iter()
            .zip(target)
            .map(|(h, t)| (h - t).powi(2))
            .sum::<f32>()
            .sqrt()
    };
    let held = run(true);
    let released = run(false);
    assert!(
        released > held + 0.1,
        "held gap {held}; released gap {released}"
    );
}

#[test]
fn physical_grip_costs_stamina_and_exhaustion_requires_a_fresh_press() {
    let mut duel = settled();
    let owner = FighterId::First;
    let hand = Hand::Left;
    assert_eq!(duel.stamina(owner), 100.);
    duel.set_hand(owner, hand, true);
    let mut exhausted = false;
    for _ in 0..1650 {
        duel.step();
        if let Some(g) = duel.grip(owner, hand) {
            assert!(duel.stamina(owner) < 100.);
            assert!(g.load_n.is_finite() && (0.0..=1200.01).contains(&g.load_n));
        }
        if duel.stamina(owner) == 0. {
            assert!(duel.grip(owner, hand).is_none());
            exhausted = true;
            break;
        }
    }
    assert!(
        exhausted,
        "continuous attached hand exhausted the owner's stamina"
    );
    for _ in 0..360 {
        duel.step();
        assert!(duel.grip(owner, hand).is_none());
    }
    assert_eq!(duel.stamina(owner), 100.);
    assert_eq!(duel.stamina(owner.opponent()), 100.);
}

#[test]
fn one_fighter_can_hold_two_limbs_and_release_only_one() {
    let owner = FighterId::First;
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    duel.set_input(owner, [-0.5, -1.], false).unwrap();
    duel.set_hand(owner, Hand::Right, true);
    for tick in 0..150 {
        if tick == 25 {
            duel.set_hand(owner, Hand::Left, true);
        }
        duel.step();
        if duel.grip(owner, Hand::Left).is_some() && duel.grip(owner, Hand::Right).is_some() {
            break;
        }
    }
    let left = duel.grip(owner, Hand::Left).unwrap();
    let right = duel.grip(owner, Hand::Right).unwrap();
    assert_ne!(left.region, right.region);
    let full = duel.stamina(owner);
    duel.step();
    assert!(full - duel.stamina(owner) >= 7. / 60. - 1e-5);
    duel.set_hand(owner, Hand::Right, false);
    assert!(duel.grip(owner, Hand::Right).is_none());
    assert_eq!(
        duel.grip(owner, Hand::Left).unwrap().created_tick,
        left.created_tick
    );
    let one = duel.stamina(owner);
    duel.step();
    assert!(one - duel.stamina(owner) >= 4. / 60. - 1e-5);
    assert_eq!(duel.stamina(owner.opponent()), 100.);
}
