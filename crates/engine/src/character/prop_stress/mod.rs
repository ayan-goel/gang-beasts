use super::*;
use crate::{ArenaKind, BodyPart, Consciousness, Hand, MeleePhase};
use std::time::Instant;

mod measurements;
use measurements::Measurements;

fn populated_roof() -> (Duel, [RigidBodyHandle; 100]) {
    let mut game = Duel::skyline_match_with_players(PhysicsConfig::default(), 8).unwrap();
    for _ in 0..180 {
        game.step();
    }
    assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
    let arena = game.arena.as_mut().unwrap();
    let crates = std::array::from_fn(|i| {
        let x = ((i % 5) as f32 - 2.) * 1.25;
        let z = ((i / 5 % 5) as f32 - 2.) * 1.25;
        let y = 2.5 + (i / 25) as f32 * 1.25;
        arena.add_test_crate(&mut game.world, [x, y, z])
    });
    assert_eq!(
        arena
            .definition
            .objects
            .iter()
            .filter(|o| o.kind == ArenaKind::Crate)
            .count(),
        104
    );
    assert_eq!(game.world.bodies.len(), 229);
    (game, crates)
}

fn drive(game: &mut Duel, crates: &[RigidBodyHandle; 100], tick: usize) {
    if tick.is_multiple_of(6) {
        for id in FighterId::ALL {
            let pelvis = game.fighters[id as usize].bodies[BodyPart::Pelvis.index()];
            let p = game.world.bodies[pelvis].translation();
            let d = p.x.hypot(p.z).max(0.5);
            game.set_input(id, [-p.x / d, -p.z / d], false).unwrap();
            let phase = (tick / 6 + 3 * id as usize) % 24;
            game.set_hand(id, Hand::Left, phase < 14);
            game.set_hand(id, Hand::Right, phase == 2 || phase == 18);
            game.set_lift(id, (8..14).contains(&phase));
        }
    }
    // Disturb one crate per tick so a long sample cannot become a sleeping stack.
    {
        let body = &mut game.world.bodies[crates[tick % crates.len()]];
        let p = body.translation();
        let d = p.x.hypot(p.z).max(1.);
        body.apply_impulse(vector![-p.x / d, 1.5, -p.z / d] * body.mass(), true);
    }
}

fn verify_retirement(game: &mut Duel, crates: &[RigidBodyHandle; 100]) {
    // Kick an outside crate clear of the pile and follow the normal fall/retirement path.
    let (index, &handle) = crates
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            game.world.bodies[**a]
                .translation()
                .x
                .total_cmp(&game.world.bodies[**b].translation().x)
        })
        .unwrap();
    let body = &mut game.world.bodies[handle];
    body.apply_impulse(vector![20., 8., 0.] * body.mass(), true);
    let id = 10 + index as u16;
    for _ in 0..300 {
        game.step();
        if !game.world.bodies[handle].is_enabled() {
            break;
        }
    }
    let body = &game.world.bodies[handle];
    assert!(
        !body.is_enabled() && body.translation().y < game.map_definition().unwrap().fall_y - 16.
    );
    let state = game
        .arena_states()
        .into_iter()
        .find(|s| s.id == id)
        .unwrap();
    assert!(!state.visible);
    let arena = game.arena.as_ref().unwrap();
    assert!(
        arena
            .grip_surfaces(&game.world)
            .all(|(obj, _, _)| obj != id)
    );
    assert!(FighterId::ALL.into_iter().all(|who| {
        [Hand::Left, Hand::Right]
            .into_iter()
            .all(|hand| game.arena_grip(who, hand) != Some(id))
    }));
}

fn run(warmup: usize, measured: usize) {
    let (mut game, crates) = populated_roof();
    let mut metrics = Measurements::new(&game, &crates);
    for tick in 0..warmup + measured {
        drive(&mut game, &crates, tick);
        let started = Instant::now();
        game.step();
        let elapsed = started.elapsed();
        assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
        assert!(
            FighterId::ALL
                .into_iter()
                .all(|id| game.vitals(id).phase != Consciousness::Eliminated)
        );
        metrics.observe(&game, &crates, (tick >= warmup).then_some(elapsed));
    }
    metrics.finish(measured);
    verify_retirement(&mut game, &crates);
    game.reset();
    assert_eq!(game.world.bodies.len(), 129);
    assert_eq!(game.world.impulse_joints.len(), 112);
    assert_eq!(game.arena_states().len(), 9);
    for _ in 0..181 {
        game.step();
    }
    assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
    assert!(game.arena_states().iter().all(|s| s.visible));
}

#[test]
fn eight_fighters_share_one_solver_with_a_hundred_extra_physical_crates() {
    if std::env::var_os("BRAWLER_PROFILE_CRATES").is_some() {
        for _ in 0..5 {
            run(1800, 7200);
        }
    } else {
        run(0, 360);
    }
}
