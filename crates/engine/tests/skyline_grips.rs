use brawler_engine::{Duel, FighterId, Hand, HandPhase, PhysicsConfig};

#[test]
fn coral_can_walk_to_grab_lift_and_release_the_southern_crate() {
    let who = FighterId::First;
    let hands = [Hand::Left, Hand::Right];
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    game.set_input(who, [0., 1.], false).unwrap();
    for hand in hands {
        game.set_hand(who, hand, true);
    }
    for _ in 0..150 {
        game.step();
        if hands
            .iter()
            .any(|&h| game.hand_state(who, h).phase == HandPhase::Gripping)
        {
            break;
        }
    }
    assert!(
        hands
            .iter()
            .any(|&h| game.hand_state(who, h).phase == HandPhase::Gripping),
        "reaching the southern crate must attach a hand"
    );

    assert!(hands.iter().any(|&h| game.arena_grip(who, h) == Some(7)));
    let initial_y = game.arena_states()[6].position[1];
    game.set_input(who, [0., 0.], false).unwrap();
    game.set_lift(who, true);
    let mut peak_y = initial_y;
    for _ in 0..70 {
        game.step();
        peak_y = peak_y.max(game.arena_states()[6].position[1]);
    }
    assert!(
        peak_y > initial_y + 0.12,
        "crate rose from {initial_y} to {peak_y}"
    );
    assert!(game.stamina(who) < 100.);
    let before = game.arena_states();
    game.clear_hands(who);
    assert_eq!(game.arena_states(), before);
    assert!(
        hands
            .iter()
            .all(|&h| game.hand_state(who, h).phase != HandPhase::Gripping)
    );
}
