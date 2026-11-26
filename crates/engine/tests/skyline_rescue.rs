use brawler_engine::{BodyPart, Duel, FighterId, Hand, MeleePhase, PhysicsConfig};

const CORAL: FighterId = FighterId::First;
const HANDS: [Hand; 2] = [Hand::Left, Hand::Right];

fn hips(game: &Duel) -> [f32; 3] {
    game.body_states(CORAL)[BodyPart::Pelvis.index()]
        .state
        .position
}

fn rescue(reach: u64, delay: u64) -> Result<(), &'static str> {
    let mut game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    for _ in 0..180 {
        game.step();
    }
    let mut caught = None;
    for tick in 0..7750 {
        match tick {
            6800 => game.set_input(CORAL, [-1., 0.], false).unwrap(),
            7050 => game.set_input(CORAL, [1., 0.], false).unwrap(),
            7078 => game.set_input(CORAL, [0., 0.], false).unwrap(),
            t if t == reach => HANDS.iter().for_each(|&h| game.set_hand(CORAL, h, true)),
            _ => {}
        }
        if tick >= 7200 {
            let held = HANDS.iter().any(|&h| game.arena_grip(CORAL, h) == Some(1));
            if held {
                caught.get_or_insert(tick);
            }
            if caught.is_some_and(|start| tick == start + delay) {
                let [x, y, _] = hips(&game);
                if !held || x >= -10. || y >= 0. {
                    return Err("no hanging catch at pull");
                }
                game.set_jump(CORAL, true);
            }
        }
        game.step();
    }
    if caught.is_none() {
        return Err("missed roof");
    }
    let [x, y, _] = hips(&game);
    if x <= -10. || y <= 0.55 || HANDS.iter().any(|&h| game.arena_grip(CORAL, h).is_some()) {
        return Err("did not climb onto roof with hands free");
    }
    game.clear_hands(CORAL);
    game.clear_jump(CORAL);
    game.set_input(CORAL, [1., 0.], false).unwrap();
    for _ in 0..90 {
        game.step();
    }
    let round = game.match_state().unwrap();
    if hips(&game)[0] < x + 0.8 || round.phase != MeleePhase::Playing || round.round_number != 1 {
        return Err("did not resume walking in round one");
    }
    Ok(())
}

#[test]
fn standard_west_rescue_succeeds_across_reach_and_pull_timings() {
    let mut misses = Vec::new();
    // The spec allows two misses. Keep the full response-time grid so a tuning change
    // cannot improve the score by dropping a previously unsuccessful approach.
    for reach in [7150, 7157, 7164, 7171] {
        for delay in [25, 40, 55, 70, 85] {
            if let Err(reason) = rescue(reach, delay) {
                misses.push((reach, delay, reason));
            }
        }
    }
    eprintln!(
        "Skyline rescue: {}/20 successful, misses {misses:?}",
        20 - misses.len()
    );
    assert!(misses.len() <= 2, "spec section 29.1: {misses:?}");
}
