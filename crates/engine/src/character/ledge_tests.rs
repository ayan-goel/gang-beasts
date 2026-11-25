use super::{Duel, FighterId};
use crate::{BodyPart, Hand, PhysicsConfig};
use rapier3d::prelude::*;

const OWNER: FighterId = FighterId::First;

fn falling_reach(hands: &[Hand], outside: f32) -> Duel {
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    for hand in hands {
        game.set_hand(OWNER, *hand, true);
    }
    for _ in 0..40 {
        game.step();
    }
    // Move the whole rig to an unsupported fall facing along the roof. Its near hand starts
    // outside contact range, and its shoulder stays within anatomical reach of the lip.
    let fighter = &game.fighters[0];
    let hips = &game.world.bodies[fighter.bodies[BodyPart::Pelvis.index()]];
    let turn = Rotation::from_axis_angle(&Vector::y_axis(), std::f32::consts::FRAC_PI_2)
        * hips.rotation().inverse();
    let origin = *hips.translation();
    for body in fighter.bodies {
        let b = &mut game.world.bodies[body];
        let center = turn * (b.translation() - origin) + vector![0., -0.12, -10. - outside];
        b.set_position(
            Isometry::from_parts(center.into(), turn * b.rotation()),
            true,
        );
        b.set_linvel(vector![0., -0.7, 0.], true);
        b.set_angvel(Vector::zeros(), true);
    }
    game.world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
    game
}

#[test]
fn a_reaching_fighter_catches_an_exposed_lip_and_can_let_go_while_hanging() {
    let hands = [Hand::Left, Hand::Right];
    let mut game = falling_reach(&hands, 0.53);
    for _ in 0..30 {
        game.step();
        if hands.iter().any(|&h| game.arena_grip(OWNER, h) == Some(1)) {
            break;
        }
    }
    assert!(
        hands.iter().any(|&h| game.arena_grip(OWNER, h) == Some(1)),
        "missed reachable lip"
    );
    for _ in 0..180 {
        game.step();
    }
    assert!(
        hands.iter().any(|&h| game.arena_grip(OWNER, h) == Some(1)),
        "lost the hold under body weight"
    );
    let hanging = game.body_states(OWNER)[BodyPart::Pelvis.index()]
        .state
        .position;
    let bodies = game.body_states(OWNER);
    let hand = bodies[BodyPart::LeftHand.index()].state.position;
    assert!(
        (hand[2] + 10.).hypot(hand[1]) < 0.15,
        "palm drifted from lip: {hand:?}"
    );
    assert!(hanging[2] < -10. && (-1.3..-0.4).contains(&hanging[1]));
    game.clear_hands(OWNER);
    for _ in 0..60 {
        game.step();
    }
    let fallen = game.body_states(OWNER)[BodyPart::Pelvis.index()]
        .state
        .position;
    assert!(fallen[1] < hanging[1] - 2.);
}

#[test]
fn a_fighter_can_catch_the_roof_from_a_collapsing_walkway_using_only_controls() {
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    // Only player input changes. The walkway detaches on its normal tick 7200.
    for tick in 0..7320 {
        match tick {
            6800 => game.set_input(OWNER, [-1., 0.], false).unwrap(),
            7050 => game.set_input(OWNER, [1., 0.], false).unwrap(),
            7078 => game.set_input(OWNER, [0., 0.], false).unwrap(),
            7160 => {
                for hand in [Hand::Left, Hand::Right] {
                    game.set_hand(OWNER, hand, true);
                }
            }
            _ => {}
        }
        game.step();
        if tick + 1 >= 7229 {
            assert!(
                [Hand::Left, Hand::Right]
                    .into_iter()
                    .any(|h| game.arena_grip(OWNER, h) == Some(1)),
                "lost roof at {}",
                tick + 1
            );
        }
    }
    let hips = game.body_states(OWNER)[BodyPart::Pelvis.index()]
        .state
        .position;
    assert!(hips[1] < -0.4 && hips[1] > -1.5);
}

#[test]
fn a_far_or_released_hand_cannot_catch_the_roof() {
    for (outside, released) in [(1.6, false), (0.53, true)] {
        let mut game = falling_reach(&[Hand::Left, Hand::Right], outside);
        if released {
            game.clear_hands(OWNER);
        }
        for _ in 0..90 {
            game.step();
            for h in [Hand::Left, Hand::Right] {
                assert_eq!(game.arena_grip(OWNER, h), None);
            }
        }
        assert!(
            game.body_states(OWNER)[BodyPart::Pelvis.index()]
                .state
                .position[1]
                < -5.
        );
    }
}
