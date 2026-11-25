use super::*;
use crate::{Consciousness, MeleePhase};

#[test]
fn skyline_builds_exactly_the_requested_roster_at_authored_spawns() {
    for count in [0, 1, 9, usize::MAX] {
        assert!(Duel::skyline_match_with_players(PhysicsConfig::default(), count).is_err());
    }
    for count in 2..=8 {
        let game = Duel::skyline_match_with_players(PhysicsConfig::default(), count).unwrap();
        assert_eq!(game.fighter_ids().len(), count);
        assert_eq!(game.match_state().unwrap().scores(), vec![0; count]);
        for &id in game.fighter_ids() {
            let slot = id as usize;
            let spawn = game.map_definition().unwrap().spawns[slot].position;
            let pelvis = game.body_states(id)[super::super::BodyPart::Pelvis.index()]
                .state
                .position;
            assert!((pelvis[0] - spawn[0]).abs() < 0.01);
            assert!((pelvis[2] - spawn[2]).abs() < 0.01);
            assert_eq!(game.body_states(id).len(), 15);
        }
    }
}

#[test]
fn last_slot_can_win_and_respawn_through_round_and_match_resets() {
    let mut game = Duel::skyline_match_with_players(PhysicsConfig::default(), 8).unwrap();
    for _ in 0..180 {
        game.step();
    }
    for fighter in &game.fighters[..7] {
        for &h in &fighter.bodies {
            let p = *game.world.bodies[h].translation() - Vector::y() * 15.;
            game.world.bodies[h].set_translation(p, true);
        }
    }
    game.step();
    assert_eq!(game.consciousness_changes().len(), 7);
    assert_eq!(game.match_state().unwrap().winner, Some(7));
    for _ in 0..300 {
        game.step();
    }
    let state = game.match_state().unwrap();
    assert_eq!(state.phase, MeleePhase::Countdown);
    assert_eq!(state.scores(), &[0, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(game.fighter_ids().len(), 8);
    for &id in game.fighter_ids() {
        assert_eq!(game.vitals(id).phase, Consciousness::Active);
    }
    game.reset();
    assert_eq!(game.match_state().unwrap().round_id, 3);
    assert_eq!(game.match_state().unwrap().scores(), &[0; 8]);
}

#[test]
fn a_held_hand_hits_and_grips_a_nonadjacent_fighter_and_releases_on_elimination() {
    let mut game = Duel::create(PhysicsConfig::default(), false, 3).unwrap();
    // Keep the established sparring distance between slots zero and two.
    for handle in game.fighters[1].bodies {
        let p = *game.world.bodies[handle].translation() + Vector::x() * 3.;
        game.world.bodies[handle].set_translation(p, true);
    }
    for _ in 0..60 {
        game.step();
    }
    let (owner, target, hand) = (FighterId::First, FighterId::Third, Hand::Left);
    game.set_hand(owner, hand, true);
    let mut hits = Vec::new();
    for _ in 0..100 {
        game.step();
        hits.extend_from_slice(game.hits());
        if game.grip(owner, hand).is_some() {
            break;
        }
    }
    assert_eq!(hits.len(), 1);
    assert_eq!((hits[0].attacker, hits[0].defender), (owner, target));
    assert_eq!(game.grip(owner, hand).unwrap().target, target);
    game.eliminate(target);
    assert!(game.grip(owner, hand).is_none());
}
