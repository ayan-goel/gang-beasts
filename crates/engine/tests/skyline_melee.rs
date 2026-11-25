use brawler_engine::{Consciousness, Duel, FighterId, MeleePhase, PhysicsConfig};

#[test]
fn running_off_the_roof_scores_once_resets_the_arena_and_reaches_a_rematch() {
    let mut game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    let initial = game.arena_states();
    let (loser, winner) = (FighterId::First, FighterId::Second);
    for round in 1..=3 {
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::Countdown);
        game.set_input(loser, [-1., 0.], true).unwrap();
        for _ in 0..180 {
            game.step();
        }
        assert_eq!(game.tick(), 0);
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
        game.set_input(loser, [-1., 0.], true).unwrap();
        for _ in 0..1200 {
            game.step();
            if game.match_state().unwrap().phase != MeleePhase::Playing {
                break;
            }
        }
        let m = game.match_state().unwrap();
        assert_eq!(
            m.phase,
            MeleePhase::RoundResults,
            "fall did not end round {round}"
        );
        assert_eq!(m.winner, Some(winner as u8));
        assert_eq!(m.scores(), &[0, round]);
        assert_eq!(game.vitals(loser).phase, Consciousness::Eliminated);
        let end = game.tick();
        for _ in 0..299 {
            game.step();
        }
        assert_eq!(game.tick(), end);
        game.step();
        if round < 3 {
            assert_eq!(game.arena_states(), initial);
        }
    }
    let epoch = game.match_state().unwrap().round_id;
    assert_eq!(game.match_state().unwrap().phase, MeleePhase::MatchResults);
    game.reset();
    assert!(game.match_state().unwrap().round_id > epoch);
    assert_eq!(game.match_state().unwrap().scores(), &[0, 0]);
    assert_eq!(game.vitals(loser).phase, Consciousness::Active);
    assert_eq!(game.arena_states(), initial);
}
