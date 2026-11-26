use super::{Duel, FighterId, ledge_tests::falling_reach};
use crate::{BodyPart, Hand};

pub(super) const CORAL: FighterId = FighterId::First;

pub(super) fn hanging() -> Duel {
    let mut game = falling_reach(&[Hand::Left, Hand::Right], 0.53);
    for _ in 0..195 {
        game.step();
    }
    assert_eq!(game.arena_grip(CORAL, Hand::Left), Some(1));
    assert_eq!(game.arena_grip(CORAL, Hand::Right), None);
    game
}

fn hips(game: &Duel) -> [f32; 3] {
    game.body_states(CORAL)[BodyPart::Pelvis.index()]
        .state
        .position
}

#[test]
fn one_hand_pull_returns_a_hanging_fighter_to_supported_play() {
    let mut game = hanging();
    let mut previous = hips(&game);
    assert!(previous[1] < -0.5);
    game.set_jump(CORAL, true);
    for tick in 0..300 {
        game.step();
        let now = hips(&game);
        let displacement =
            rapier3d::prelude::Vector::from(now) - rapier3d::prelude::Vector::from(previous);
        assert!(displacement.norm() < 0.24, "unphysical step at {tick}");
        previous = now;
    }
    assert!(
        previous[1] > 0.55 && previous[2] > -10.0,
        "pelvis after pulling {previous:?}"
    );
    for h in [Hand::Left, Hand::Right] {
        assert_eq!(game.arena_grip(CORAL, h), None);
    }
    assert!(game.fighters[0].walking.state.feet.contains(&true));
    game.clear_jump(CORAL);
    game.clear_hands(CORAL);
    game.set_input(CORAL, [0., 1.], false).unwrap();
    for _ in 0..90 {
        game.step();
    }
    assert!(hips(&game)[2] > previous[2] + 1.);
}

#[test]
fn pulling_from_two_hands_finishes_on_the_roof() {
    let mut game =
        super::ledge_tests::falling_at(&[Hand::Left, Hand::Right], 0.57, std::f32::consts::PI);
    for _ in 0..100 {
        game.step();
    }
    for h in [Hand::Left, Hand::Right] {
        assert_eq!(game.arena_grip(CORAL, h), Some(1), "{h:?}");
    }
    game.set_jump(CORAL, true);
    for _ in 0..360 {
        game.step();
    }
    let p = game.body_states(CORAL)[BodyPart::Pelvis.index()]
        .state
        .position;
    assert!(p[1] > 0.55 && p[2] > -10., "two-hand pelvis {p:?}");
    assert!(game.fighters[0].walking.state.feet.contains(&true));
}

#[test]
fn the_real_west_walkway_catch_can_be_mantled() {
    use crate::PhysicsConfig;
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    for tick in 0..7720 {
        match tick {
            6800 => game.set_input(CORAL, [-1., 0.], false).unwrap(),
            7050 => game.set_input(CORAL, [1., 0.], false).unwrap(),
            7078 => game.set_input(CORAL, [0., 0.], false).unwrap(),
            7160 => {
                for h in [Hand::Left, Hand::Right] {
                    game.set_hand(CORAL, h, true);
                }
            }
            7320 => {
                assert_eq!(game.arena_grip(CORAL, Hand::Right), Some(1));
                game.set_jump(CORAL, true);
            }
            _ => {}
        }
        game.step();
    }
    let p = game.body_states(CORAL)[BodyPart::Pelvis.index()]
        .state
        .position;
    assert!(p[0] > -10. && p[1] > 0.55, "west rescue pelvis {p:?}");
    assert!(game.fighters[0].walking.state.feet.contains(&true));
}

#[test]
fn nearby_catches_allow_the_fall_to_settle_before_the_player_pulls_up() {
    // A few input ticks change the landing angle on the same walkway. Each run
    // must make real palm contact, bear the hanging load and return to foot support.
    for stop in 7076..=7083 {
        let mut game = Duel::skyline(crate::PhysicsConfig::default()).unwrap();
        let mut caught = None;
        for tick in 0..7750 {
            match tick {
                6800 => game.set_input(CORAL, [-1., 0.], false).unwrap(),
                7050 => game.set_input(CORAL, [1., 0.], false).unwrap(),
                t if t == stop => game.set_input(CORAL, [0., 0.], false).unwrap(),
                7160 => {
                    for hand in [Hand::Left, Hand::Right] {
                        game.set_hand(CORAL, hand, true);
                    }
                }
                _ => {}
            }
            let held = [Hand::Left, Hand::Right]
                .iter()
                .any(|&hand| game.arena_grip(CORAL, hand) == Some(1));
            if tick >= 7200 && held && caught.is_none() {
                caught = Some(tick);
            }
            if caught.is_some_and(|start| tick == start + 40) {
                let hip = hips(&game);
                assert!(
                    held && hip[0] < -10. && hip[1] < -0.4,
                    "stop {stop}: lost catch at {hip:?}"
                );
                game.set_jump(CORAL, true);
            }
            game.step();
        }
        assert!(caught.is_some(), "stop {stop}: missed lip");
        let hip = hips(&game);
        assert!(
            hip[0] > -10. && hip[1] > 0.55 && game.fighters[0].walking.state.feet.contains(&true),
            "stop {stop}: failed to recover at {hip:?}"
        );
    }
}
