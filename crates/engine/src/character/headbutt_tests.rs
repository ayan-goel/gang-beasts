use super::*;
use crate::BodyPart;

fn facing_fighters(distance: f32) -> Duel {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    for (slot, sign) in [1., -1.].into_iter().enumerate() {
        let shift = vector![-sign * 0.15, 0., sign * (distance * 0.5 - 0.28)];
        for handle in duel.fighters[slot].bodies {
            let body = &mut duel.world.bodies[handle];
            body.set_translation(body.translation() + shift, true);
        }
    }
    for _ in 0..40 {
        duel.step();
    }
    duel
}

#[test]
fn a_held_headbutt_hits_once_in_reach_and_whiffs_out_of_reach() {
    for (distance, expected) in [(0.41, 1), (1.3, 0)] {
        let mut duel = facing_fighters(distance);
        let id = FighterId::First;
        duel.set_headbutt(id, true);
        let mut hits = Vec::new();
        for _ in 0..70 {
            duel.step();
            hits.extend_from_slice(duel.hits());
        }
        assert_eq!(hits.len(), expected, "distance {distance}: {hits:?}");
        for hit in hits {
            assert_eq!(hit.source, crate::StrikeSource::Headbutt);
            assert_eq!((hit.attacker, hit.defender), (id, FighterId::Second));
            assert!(hit.energy_j > 0. && hit.damage > 0.);
        }
        assert_eq!(duel.fighters[0].headbutt.attack_id, 1);
    }
}

#[test]
fn a_tap_drives_the_head_forward_and_spends_stamina_with_independent_hand_input() {
    let mut duel = facing_fighters(2.);
    let first = FighterId::First;
    let z = |d: &Duel| d.body_states(first)[BodyPart::Head.index()].state.position[2];
    let rest = z(&duel);
    duel.set_headbutt(first, true);
    duel.set_headbutt(first, false);
    duel.set_hand(first, Hand::Left, true);
    let mut forward: f32 = 0.;
    for tick in 0..32 {
        duel.step();
        if tick == 0 {
            assert!(duel.stamina(first) < 100.);
            assert_ne!(
                duel.hand_state(first, Hand::Left).attack_id,
                duel.fighters[0].headbutt.attack_id
            );
        }
        forward = forward.max(rest - z(&duel));
    }
    assert!(forward > 0.04, "head travel {forward}");
}

#[test]
fn a_wall_between_fighters_blocks_headbutt_damage() {
    let mut game = facing_fighters(0.41);
    game.world
        .colliders
        .insert(ColliderBuilder::cuboid(2., 2., 0.012).translation(vector![0., 1., 0.]));
    game.set_headbutt(FighterId::First, true);
    for _ in 0..50 {
        game.step();
        assert!(game.hits().is_empty());
    }
}

#[test]
fn cancellation_cooldown_stamina_and_countdown_gate_queued_attacks() {
    let first = FighterId::First;
    let mut game = facing_fighters(2.);
    game.set_headbutt(first, true);
    game.set_headbutt(first, false);
    game.clear_headbutt(first);
    game.step();
    assert_eq!(game.fighters[0].headbutt.attack_id, 0);
    game.fighters[0].walking.state.recovery_strength = 1.;
    game.set_headbutt(first, true);
    game.fighters[0].walking.state.recovery_strength = 0.;
    game.step();
    assert_eq!(game.fighters[0].headbutt.attack_id, 0);
    *game.grips.stamina_mut(first) = 1.;
    game.set_headbutt(first, true);
    game.step();
    assert_eq!(game.fighters[0].headbutt.attack_id, 0);
    *game.grips.stamina_mut(first) = 100.;
    game.set_headbutt(first, false);
    game.set_headbutt(first, true);
    game.step();
    game.clear_headbutt(first);
    for _ in 1..38 {
        game.set_headbutt(first, false);
        game.set_headbutt(first, true);
        game.step();
        assert_eq!(game.fighters[0].headbutt.attack_id, 1);
    }
    game.set_headbutt(first, false);
    game.set_headbutt(first, true);
    game.step();
    assert_eq!(game.fighters[0].headbutt.attack_id, 2);
    game.reset();
    assert_eq!(game.fighters[0].headbutt.attack_id, 0);
    let mut match_game = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    match_game.set_headbutt(first, true);
    for _ in 0..181 {
        match_game.step();
    }
    assert_eq!(match_game.fighters[0].headbutt.attack_id, 0);
}
