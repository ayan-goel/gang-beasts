use super::{
    FighterId,
    mantle_tests::{CORAL, hanging},
};
use crate::{BodyPart, Hand};

#[test]
fn an_occupied_landing_or_a_low_ceiling_preserves_the_hang_until_clear() {
    use rapier3d::prelude::*;
    for ceiling in [true, false] {
        let (mut pull, mut idle) = (hanging(), hanging());
        let mut barriers = Vec::new();
        for g in [&mut pull, &mut idle] {
            if ceiling {
                barriers.push(g.world.colliders.insert(
                    ColliderBuilder::cuboid(2., 0.12, 0.8).translation(vector![-0.60, 0.8, -10.]),
                ));
            } else {
                let bodies = g.fighters[1].bodies;
                let shift = vector![-0.60, 0.82, -9.65]
                    - g.world.bodies[bodies[BodyPart::Pelvis.index()]].translation();
                for b in bodies {
                    let p = g.world.bodies[b].translation() + shift;
                    g.world.bodies[b].set_translation(p, true);
                }
                g.world
                    .bodies
                    .propagate_modified_body_positions_to_colliders(&mut g.world.colliders);
            }
        }
        pull.set_jump(CORAL, true);
        for _ in 0..45 {
            pull.step();
            idle.step();
            assert_eq!(pull.body_states(CORAL), idle.body_states(CORAL));
            assert_eq!(pull.stamina(CORAL), idle.stamina(CORAL));
        }
        assert_eq!(pull.arena_grip(CORAL, Hand::Left), Some(1));
        if ceiling {
            pull.world.remove_collider(barriers[0]);
        } else {
            for b in pull.fighters[1].bodies {
                let p = pull.world.bodies[b].translation() + vector![3., 0., 0.];
                pull.world.bodies[b].set_translation(p, true);
            }
            pull.world
                .bodies
                .propagate_modified_body_positions_to_colliders(&mut pull.world.colliders);
        }
        for _ in 0..60 {
            pull.step();
        }
        assert!(
            pull.body_states(CORAL)[BodyPart::Pelvis.index()]
                .state
                .position[1]
                > 0.
        );
    }
}

#[test]
fn cancelling_a_partial_pull_stops_assistance_and_preserves_the_grip() {
    let mut game = hanging();
    game.set_jump(CORAL, true);
    for _ in 0..15 {
        game.step();
    }
    game.clear_jump(CORAL);
    for _ in 0..140 {
        game.step();
    }
    assert_eq!(game.arena_grip(CORAL, Hand::Left), Some(1));
    assert!(
        game.body_states(CORAL)[BodyPart::Pelvis.index()]
            .state
            .position[1]
            < -0.65
    );
}

#[test]
fn starting_a_pull_spends_effort_and_a_panel_losing_support_cannot_supply_it() {
    use rapier3d::prelude::*;
    let mut game = hanging();
    game.set_jump(CORAL, true);
    let stamina = game.stamina(CORAL);
    assert!(game.pull_from_ledge(CORAL));
    assert!(
        game.stamina(CORAL) < stamina,
        "an assisted tick spends climb stamina"
    );
    let mut panel = super::ledge_tests::falling_reach(&[Hand::Left, Hand::Right], 0.53);
    let fighter = &panel.fighters[0];
    let wrist = fighter.bodies[BodyPart::LeftHand.index()];
    let shift = vector![-12.04, 0.025, -2.] - panel.world.bodies[wrist].translation();
    for b in fighter.bodies {
        let position = panel.world.bodies[b].translation() + shift;
        panel.world.bodies[b].set_translation(position, true);
    }
    panel
        .world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut panel.world.colliders);
    panel.grips.acquire(
        &mut panel.world,
        &panel.fighters,
        panel.arena.as_ref(),
        panel.tick,
    );
    assert_eq!(panel.arena_grip(CORAL, Hand::Left), Some(8));
    panel.set_jump(CORAL, true);
    panel.tick = 7199;
    assert!(
        !panel.pull_from_ledge(CORAL),
        "panel detaches in this solve"
    );
    panel.step();
    assert!(
        !panel.pull_from_ledge(CORAL),
        "falling panel supplied assistance"
    );
}

#[test]
fn a_second_fighter_hanging_from_the_climber_loads_the_same_pull() {
    use rapier3d::prelude::*;
    let (mut loaded, mut solo) = (hanging(), hanging());
    for g in [&mut loaded, &mut solo] {
        g.set_hand(FighterId::Second, Hand::Left, true);
        for _ in 0..40 {
            g.step();
        }
    }
    // Place Blue below a shin with the reaching palm at its surface. The normal solver
    // then carries Blue's full mass through an opponent grip and Coral's single roof grip.
    let carrier = loaded.fighters[0].bodies;
    let rider = loaded.fighters[1].bodies;
    let wrist = *loaded.world.bodies[rider[BodyPart::LeftHand.index()]].translation();
    let hips = *loaded.world.bodies[rider[BodyPart::Pelvis.index()]].translation();
    let turn = Rotation::rotation_between(&(wrist - hips), &Vector::y()).unwrap();
    let at = loaded.world.bodies[carrier[BodyPart::RightShin.index()]].translation()
        - Vector::y() * 0.13;
    for b in rider {
        let body = &mut loaded.world.bodies[b];
        let pos = at + turn * (body.translation() - wrist);
        body.set_position(
            Isometry::from_parts(pos.into(), turn * body.rotation()),
            true,
        );
        body.set_linvel(Vector::zeros(), true);
        body.set_angvel(Vector::zeros(), true);
    }
    loaded
        .world
        .bodies
        .propagate_modified_body_positions_to_colliders(&mut loaded.world.colliders);
    loaded.grips.acquire(
        &mut loaded.world,
        &loaded.fighters,
        loaded.arena.as_ref(),
        loaded.tick,
    );
    assert_eq!(
        loaded.grip(FighterId::Second, Hand::Left).unwrap().target,
        CORAL
    );
    for g in [&mut loaded, &mut solo] {
        g.set_jump(CORAL, true);
    }
    for _ in 0..18 {
        loaded.step();
        solo.step();
        assert!(loaded.grip(FighterId::Second, Hand::Left).is_some());
        assert_eq!(loaded.arena_grip(CORAL, Hand::Left), Some(1));
    }
    let height = |g: &super::Duel| {
        g.world.bodies[g.fighters[0].bodies[BodyPart::Pelvis.index()]]
            .translation()
            .y
    };
    assert!(
        height(&solo) > height(&loaded) + 0.06,
        "solo {} vs loaded {}",
        height(&solo),
        height(&loaded)
    );
}
