use super::*;
use crate::{Hand, HandPhase};
use rapier3d::prelude::*;

fn ready() -> LocomotionRagdoll {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.; 3],
    )
    .unwrap();
    for _ in 0..60 {
        rig.step();
    }
    rig
}

#[test]
fn taps_complete_one_attack_and_held_hands_reach_independently() {
    let mut rig = ready();
    rig.set_hand(Hand::Left, true);
    rig.set_hand(Hand::Left, false);
    for tick in 1..=40 {
        if tick == 4 {
            rig.set_hand(Hand::Right, true);
        }
        rig.step();
        for (side, elapsed, held) in [(Hand::Left, tick, false), (Hand::Right, tick - 3, true)] {
            let expected = match elapsed {
                1..=5 => HandPhase::WindUp,
                6..=12 => HandPhase::Strike,
                13..=24 => HandPhase::Recovery,
                25.. if held => HandPhase::Reach,
                _ => HandPhase::Idle,
            };
            assert_eq!(
                rig.hand_state(side).phase,
                expected,
                "tick {tick} hand {side:?}"
            );
        }
        if tick == 7 {
            assert_ne!(
                rig.hand_state(Hand::Left).attack_id,
                rig.hand_state(Hand::Right).attack_id
            );
        }
    }
    rig.set_hand(Hand::Right, false);
    rig.step();
    assert_eq!(rig.hand_state(Hand::Right).phase, HandPhase::Idle);
}

#[test]
fn a_physical_punch_transfers_momentum_to_a_suspended_target_on_the_striking_side() {
    for (side, part, x) in [
        (Hand::Left, BodyPart::LeftHand, -0.30),
        (Hand::Right, BodyPart::RightHand, 0.30),
    ] {
        let mut rig = ready();
        let world = &mut rig.pose.rig.world;
        let target = world.bodies.insert(
            RigidBodyBuilder::dynamic()
                .gravity_scale(0.)
                .translation(vector![x, 1.12, -0.54]),
        );
        let collider = world.colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.11, 0.14, 0.055).mass(2.),
            target,
            &mut world.bodies,
        );
        let hand = world.bodies[rig.pose.rig.bodies[part.index()]].colliders()[0];
        rig.set_hand(side, true);
        rig.set_hand(side, false);
        let mut hit = false;
        for _ in 0..24 {
            rig.step();
            let world = &rig.pose.rig.world;
            if rig.hand_state(side).phase == HandPhase::Strike {
                hit |= world
                    .narrow_phase
                    .contact_pair(hand, collider)
                    .is_some_and(|pair| {
                        pair.has_any_active_contact && pair.total_impulse_magnitude() > 0.01
                    });
            }
            assert!(
                rig.pose
                    .rig
                    .joint_states()
                    .iter()
                    .all(|joint| joint.anchor_error < 0.05)
            );
        }
        assert!(hit, "missing {side:?} hand strike");
        assert!(rig.pose.rig.world.bodies[target].linvel().z < -0.2);
        assert!(rig.pose.rig.body_states()[0].state.position[1] > 1.1);
    }
}

#[test]
fn focus_loss_cannot_bypass_recovery_and_reset_starts_a_new_attack_sequence() {
    let mut rig = ready();
    rig.set_hand(Hand::Left, true);
    for _ in 0..6 {
        rig.step();
    }
    let first = rig.hand_state(Hand::Left).attack_id;
    rig.clear_hands();
    rig.set_hand(Hand::Left, true);
    rig.set_hand(Hand::Left, false);
    for _ in 7..=24 {
        rig.step();
        assert_eq!(
            rig.hand_state(Hand::Left).attack_id,
            first,
            "cancellation restarted an attack"
        );
    }
    rig.step();
    assert_eq!(rig.hand_state(Hand::Left).phase, HandPhase::Idle);
    rig.set_hand(Hand::Right, true);
    rig.step();
    assert!(rig.hand_state(Hand::Right).attack_id > first);
    rig.set_enabled(false);
    rig.set_hand(Hand::Right, true);
    rig.set_enabled(true);
    rig.step();
    assert_eq!(rig.hand_state(Hand::Right).phase, HandPhase::Idle);
    rig.reset();
    rig.set_hand(Hand::Right, true);
    rig.step();
    assert_eq!(rig.hand_state(Hand::Right).attack_id, first);
}

#[test]
fn authored_arm_limits_and_motor_caps_apply_only_to_the_attacking_arm() {
    let mut rig = ready();
    let shoulder = rig
        .pose
        .rig
        .definition
        .joints
        .iter()
        .position(|joint| joint.child == BodyPart::LeftUpperArm)
        .unwrap();
    rig.pose.rig.definition.joints[shoulder].angular_limits[0][1] = 0.75;
    let base_caps: Vec<_> = rig
        .pose
        .motor_configuration()
        .joints
        .iter()
        .map(|motor| motor.max_torque_nm)
        .collect();
    rig.set_hand(Hand::Left, true);
    rig.set_hand(Hand::Left, false);
    for _ in 0..6 {
        rig.step();
    }
    assert_eq!(rig.state().targets[shoulder][0], 0.75);
    for (motor, base) in rig.pose.motor_configuration().joints.iter().zip(&base_caps) {
        let expected = if Hand::Left.parts().contains(&motor.child) {
            45.0
        } else {
            *base
        };
        assert_eq!(motor.max_torque_nm, expected);
    }
    for _ in 7..=25 {
        rig.step();
    }
    assert!(
        rig.pose
            .motor_configuration()
            .joints
            .iter()
            .zip(base_caps)
            .all(|(motor, base)| motor.max_torque_nm == base)
    );
}

#[path = "strike_balance_tests.rs"]
mod balance;
