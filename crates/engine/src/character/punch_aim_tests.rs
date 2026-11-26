use super::*;
use crate::{BodyPart, Hand};

fn fixture() -> Duel {
    Duel::skyline_match_with_players(crate::PhysicsConfig::default(), 3).unwrap()
}

fn select(game: &Duel, hand: Hand) -> Option<super::super::punch_aim::Aim> {
    super::super::punch_aim::select(
        &game.world,
        &game.definition,
        &game.fighters,
        &game.vitals,
        FighterId::First,
        hand,
    )
}

// Isolate candidate ordering from contacts between the other limbs of these test rigs.
fn targets(game: &mut Duel, placements: &[(usize, BodyPart, [f32; 3])]) {
    let owner = &game.fighters[0];
    let chest = &game.world.bodies[owner.bodies[BodyPart::Chest.index()]];
    let chest_pose = *chest.position();
    let shoulder = chest_pose * point![-0.24, 0.12, 0.];
    for fighter in &game.fighters[1..] {
        for handle in fighter.bodies {
            game.world.bodies[handle].set_enabled(false);
        }
    }
    for &(slot, part, relative) in placements {
        let handle = game.fighters[slot].bodies[part.index()];
        let body = &mut game.world.bodies[handle];
        body.set_enabled(true);
        body.set_position(
            Isometry::from_parts(
                Translation::from((shoulder + chest_pose.rotation * Vector::from(relative)).coords),
                chest_pose.rotation,
            ),
            true,
        );
    }
    game.world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
}

#[test]
fn visible_reachable_targets_rank_by_alignment_distance_and_stable_slot() {
    let mut game = fixture();
    let hand = BodyPart::LeftHand;
    targets(
        &mut game,
        &[(1, hand, [0.19, 0., -0.30]), (2, hand, [0., 0., -0.54])],
    );
    assert_eq!(select(&game, Hand::Left).unwrap().fighter, FighterId::Third);
    targets(
        &mut game,
        &[(1, hand, [0., 0., -0.52]), (2, hand, [0., 0., -0.34])],
    );
    assert_eq!(select(&game, Hand::Left).unwrap().fighter, FighterId::Third);
    targets(
        &mut game,
        &[(1, hand, [0., 0., -0.40]), (2, hand, [0., 0., -0.40])],
    );
    assert_eq!(
        select(&game, Hand::Left).unwrap().fighter,
        FighterId::Second
    );
    targets(
        &mut game,
        &[(1, hand, [0., 0., 0.32]), (2, hand, [0., 0., -1.1])],
    );
    assert!(select(&game, Hand::Left).is_none());
    targets(&mut game, &[(1, hand, [0.42, 0., -0.14])]);
    assert!(select(&game, Hand::Left).is_none());
}

#[test]
fn a_solid_wall_blocks_assistance_and_a_sensor_preserves_it() {
    let mut game = fixture();
    targets(&mut game, &[(1, BodyPart::Head, [0.32, 0.12, -0.42])]);
    let aim = select(&game, Hand::Left).unwrap();
    assert!(aim.yaw < 0.);
    let chest = &game.world.bodies[game.fighters[0].bodies[BodyPart::Chest.index()]];
    let transform = chest.position() * Isometry::translation(-0.24, 0.12, -0.16);
    let wall = game
        .world
        .colliders
        .insert(ColliderBuilder::cuboid(0.8, 0.8, 0.01).position(transform));
    assert!(select(&game, Hand::Left).is_none());
    game.world.colliders[wall].set_sensor(true);
    assert_eq!(
        select(&game, Hand::Left).unwrap().fighter,
        FighterId::Second
    );
}

fn off_axis_punch(x: f32, z: f32, enabled: bool) -> (usize, f32, f32) {
    let mut g = Duel::new(crate::PhysicsConfig::default()).unwrap();
    let id = FighterId::First;
    for handle in g.fighters[1].bodies {
        let b = &mut g.world.bodies[handle];
        b.set_translation(b.translation() + vector![x + 0.30, 0., 0.56 - z], true);
    }
    for _ in 0..40 {
        g.step();
    }
    if !enabled {
        g.fighters[0].hands.tuning.aim_max_rad = 0.;
    }
    g.set_hand(id, Hand::Right, true);
    g.set_hand(id, Hand::Right, false);
    let mut hits = 0;
    let mut yaw = 0.;
    let mut peak = f32::NEG_INFINITY;
    for _ in 0..24 {
        g.step();
        hits += g.hits().len();
        if let Some(a) = g.fighters[0].aim[1] {
            yaw = a.yaw;
        }
        peak = peak.max(
            g.body_states(id)[BodyPart::RightHand.index()]
                .state
                .position[0],
        );
    }
    (hits, yaw, peak)
}

#[test]
fn bounded_assistance_turns_a_near_miss_into_physical_contact() {
    let (hits, yaw, reach) = off_axis_punch(0.74, 0.34, true);
    let (straight, _, baseline) = off_axis_punch(0.74, 0.34, false);
    assert_eq!((hits, straight), (1, 0));
    assert!((-0.35..-0.1).contains(&yaw));
    assert!(reach > baseline + 0.04);
}

#[test]
fn each_shoulder_selects_independently_and_an_occluded_target_yields_to_a_visible_one() {
    let mut game = fixture();
    targets(
        &mut game,
        &[
            (1, BodyPart::LeftHand, [0., 0., -0.50]),
            (2, BodyPart::RightHand, [0.34, 0., -0.51]),
        ],
    );
    let before = game.body_states(FighterId::First);
    assert_eq!(
        select(&game, Hand::Left).unwrap().fighter,
        FighterId::Second
    );
    assert_eq!(
        select(&game, Hand::Right).unwrap().fighter,
        FighterId::Third
    );
    let chest = &game.world.bodies[game.fighters[0].bodies[BodyPart::Chest.index()]];
    let at = chest.position() * Isometry::translation(-0.24, 0.12, -0.20);
    game.world
        .colliders
        .insert(ColliderBuilder::cuboid(0.035, 0.16, 0.01).position(at));
    assert_eq!(select(&game, Hand::Left).unwrap().fighter, FighterId::Third);
    assert_eq!(before, game.body_states(FighterId::First));
}

#[test]
fn leaving_the_selected_position_before_extension_turns_the_strike_into_a_whiff() {
    let mut game = Duel::new(crate::PhysicsConfig::default()).unwrap();
    for _ in 0..60 {
        game.step();
    }
    let owner = FighterId::First;
    let side = Hand::Left;
    game.set_hand(owner, side, true);
    game.set_hand(owner, side, false);
    game.step();
    assert!(game.fighters[0].aim[0].is_some());
    for handle in game.fighters[1].bodies {
        let body = &mut game.world.bodies[handle];
        body.set_translation(body.translation() + vector![1.5, 0., 0.], true);
    }
    for _ in 0..24 {
        game.step();
        assert!(game.hits().is_empty());
    }
    assert_eq!(game.vitals(FighterId::Second).health, 100.);
    assert!(game.fighters[0].aim[0].is_none());
}

#[test]
fn an_unconscious_fighter_cannot_win_selection_over_an_active_opponent() {
    let mut game = fixture();
    targets(
        &mut game,
        &[
            (1, BodyPart::LeftHand, [0., 0., -0.35]),
            (2, BodyPart::LeftHand, [0.23, 0., -0.50]),
        ],
    );
    game.vitals[1].phase = crate::Consciousness::KnockedOut;
    assert_eq!(select(&game, Hand::Left).unwrap().fighter, FighterId::Third);
}

#[test]
fn a_clinch_preserves_the_natural_punch_at_an_otherwise_assisted_angle() {
    let mut game = fixture();
    let hand = (1, BodyPart::LeftHand, [0.28, 0., -0.42]);
    targets(&mut game, &[hand]);
    let open = select(&game, Hand::Left).unwrap();
    assert!(open.yaw < -0.1);

    // Place the second pelvis against the first without moving the selected hand.
    let position = *game.world.bodies[game.fighters[0].bodies[BodyPart::Pelvis.index()]].position();
    let hips = &mut game.world.bodies[game.fighters[1].bodies[BodyPart::Pelvis.index()]];
    hips.set_enabled(true);
    hips.set_position(position * Isometry::translation(0., 0., -0.36), true);
    game.world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut game.world.colliders);
    let crowded = select(&game, Hand::Left).unwrap();
    assert_eq!(crowded.fighter, open.fighter);
    assert_eq!(crowded.yaw, 0.);
}
