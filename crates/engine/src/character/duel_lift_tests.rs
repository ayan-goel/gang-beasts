use crate::{BodyPart, Duel, FighterId, Hand, PhysicsConfig};

fn encounter() -> Duel {
    let owner = FighterId::First;
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    duel.set_input(owner, [-0.5, -1.], false).unwrap();
    duel.set_hand(owner, Hand::Right, true);
    for tick in 0..150 {
        if tick == 25 {
            duel.set_hand(owner, Hand::Left, true);
        }
        duel.step();
        if [Hand::Left, Hand::Right]
            .iter()
            .all(|&hand| duel.grip(owner, hand).is_some())
        {
            duel.set_input(owner, [0., 0.], false).unwrap();
            return duel;
        }
    }
    panic!("expected both hands to reach the approaching opponent");
}

#[test]
fn lift_with_empty_hands_leaves_both_rigs_unchanged() {
    let mut active = Duel::new(PhysicsConfig::default()).unwrap();
    let mut idle = Duel::new(PhysicsConfig::default()).unwrap();
    active.set_lift(FighterId::First, true);
    for _ in 0..60 {
        active.step();
        idle.step();
        for id in [FighterId::First, FighterId::Second] {
            assert_eq!(active.body_states(id), idle.body_states(id));
        }
    }
}

#[test]
fn two_hands_can_raise_an_opponent_clear_of_the_floor_and_release_in_motion() {
    let owner = FighterId::First;
    let mut lifted = encounter();
    let mut baseline = encounter();
    lifted.set_lift(owner, true);
    let mut max_gain = f32::NEG_INFINITY;
    let mut cleared = false;
    let starting_y = lifted.body_states(owner.opponent())[BodyPart::Pelvis.index()]
        .state
        .position[1];
    for _ in 0..100 {
        lifted.step();
        baseline.step();
        let actual = lifted.body_states(owner.opponent());
        let idle = baseline.body_states(owner.opponent());
        let pelvis = BodyPart::Pelvis.index();
        max_gain = max_gain.max(actual[pelvis].state.position[1] - idle[pelvis].state.position[1]);
        let clear = lifted.fighters[owner.opponent() as usize]
            .bodies
            .iter()
            .all(|&body| {
                lifted.world.bodies[body]
                    .colliders()
                    .iter()
                    .all(|&collider| lifted.world.colliders[collider].compute_aabb().mins.y > 0.015)
            });
        if clear
            && actual[pelvis].state.position[1] > starting_y + 0.08
            && actual[pelvis].state.position[1] > idle[pelvis].state.position[1] + 0.10
            && actual[pelvis].state.linear_velocity[1] > 0.1
            && [Hand::Left, Hand::Right]
                .iter()
                .all(|&hand| lifted.grip(owner, hand).is_some())
        {
            cleared = true;
            lifted.clear_hands(owner);
            assert_eq!(actual, lifted.body_states(owner.opponent()));
            break;
        }
    }
    assert!(
        cleared,
        "expected a two-hand lift with upward release momentum; max pelvis gain {max_gain}"
    );
}

#[test]
fn lifting_assistance_is_capped_per_attached_hand_and_stops_on_support_loss() {
    use crate::character::rig_frame::RigFrame;
    use rapier3d::prelude::*;
    for grounded in [true, false] {
        for hands in [[true, false], [true, true]] {
            let mut duel = encounter();
            if !grounded {
                let deck = duel
                    .world
                    .colliders
                    .iter()
                    .find(|(_, c)| c.parent().is_none())
                    .unwrap()
                    .0;
                duel.world.remove_collider(deck);
            }
            let owner = &duel.fighters[0];
            let mut rig = RigFrame {
                parts: owner.parts(&duel.definition, &duel.physics, duel.tick),
                world: &mut duel.world,
            };
            let momentum = |rig: &RigFrame<&mut crate::physics::PhysicsWorld>| {
                rig.parts
                    .bodies
                    .iter()
                    .map(|&id| {
                        let body = &rig.world.bodies[id];
                        body.linvel() * body.mass()
                    })
                    .sum::<Vector<Real>>()
            };
            let before = momentum(&rig);
            let mut targets = [Rotation::identity(); 14];
            super::super::lifting::apply(
                &mut rig,
                &owner.hands.tuning,
                hands,
                &crate::character::BalanceConfig::bundled().unwrap(),
                &mut targets,
            );
            let impulse = momentum(&rig) - before;
            let count = hands.into_iter().filter(|&h| h).count() as f32;
            let cap = owner.hands.tuning.lift_force_per_hand_n * count * crate::TICK_SECONDS;
            if grounded {
                assert!(impulse.y > 0. && impulse.y <= cap + 1e-4);
            } else {
                assert_eq!(impulse, Vector::zeros());
            }
            assert_eq!([impulse.x, impulse.z], [0., 0.]);
            for (i, joint) in duel.definition.joints.iter().enumerate() {
                let raised = [Hand::Left, Hand::Right]
                    .into_iter()
                    .any(|hand| hands[hand as usize] && hand.parts().contains(&joint.child));
                assert_eq!(targets[i] != Rotation::identity(), raised);
            }
        }
    }
}
