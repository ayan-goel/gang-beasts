use brawler_engine::{BodyPart, Duel, FighterId, PhysicsConfig};

#[test]
fn a_jump_lands_once_after_descent_and_ordinary_walking_stays_quiet() {
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    let coral = FighterId::First;
    let blue = FighterId::Second;
    for _ in 0..60 {
        game.step();
    }
    assert_eq!(game.last_landing(coral), None);
    for _ in 0..2 {
        game.set_jump(coral, true);
        game.set_jump(coral, false);
        let start = game.tick();
        let prior = game.last_landing(coral);
        let mut events = Vec::new();
        let mut apex = 0f32;
        for _ in 0..130 {
            game.step();
            let hip = game.body_states(coral)[BodyPart::Pelvis.index()]
                .state
                .position[1];
            apex = apex.max(hip);
            if let Some((tick, impulse)) = game.last_landing(coral) {
                if tick == game.tick() {
                    assert!(tick > start + 25 && hip < apex - 0.5);
                    assert!(impulse >= 12);
                    events.push(tick);
                }
            }
        }
        assert_eq!(events.len(), 1, "{events:?}");
        assert_ne!(game.last_landing(coral), prior);
        assert_eq!(game.last_landing(blue), None);
    }
    let last = game.last_landing(coral);
    game.set_input(coral, [0., 1.], false).unwrap();
    for _ in 0..120 {
        game.step();
    }
    assert_eq!(game.last_landing(coral), last);
    game.reset();
    assert_eq!(game.last_landing(coral), None);
}

#[test]
fn an_on_roof_stumble_sounds_before_the_silent_ring_out() {
    let mut game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    for _ in 0..240 {
        game.step();
    }
    game.set_input(FighterId::First, [-1., 0.], true).unwrap();
    let mut deck_impact = None;
    for _ in 0..600 {
        game.step();
        let hip = game.body_states(FighterId::First)[BodyPart::Pelvis.index()]
            .state
            .position;
        let event = game.last_landing(FighterId::First);
        if event.is_some_and(|(tick, _)| tick == game.tick()) {
            assert!(
                hip[0] > -10. && hip[1] > 0. && hip[1] < 0.4,
                "unexpected impact at {hip:?}"
            );
            deck_impact = event;
        }
        if game.match_state().unwrap().phase == brawler_engine::MeleePhase::RoundResults {
            assert!(hip[1] < -12.);
            assert!(
                deck_impact.is_some(),
                "expected the preceding on-roof stumble"
            );
            assert_eq!(event, deck_impact);
            return;
        }
    }
    panic!("fighter never left the roof");
}
