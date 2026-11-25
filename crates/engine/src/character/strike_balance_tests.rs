use super::*;

#[test]
fn reaching_from_a_quiet_stand_keeps_an_unloaded_fighter_on_its_feet() {
    for hands in [[true, false], [false, true], [true, true]] {
        let mut rig = ready();
        for (side, held) in [Hand::Left, Hand::Right].into_iter().zip(hands) {
            rig.set_hand(side, held);
        }
        for tick in 1..=1200 {
            rig.step();
            let body = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[BodyPart::Chest.index()]];
            let up = (body.rotation() * Vector::y()).y;
            assert!(
                up > 0.6,
                "{hands:?} tipped at {tick}: up {up}, chest {:?}",
                body.translation()
            );
            for (side, held) in [Hand::Left, Hand::Right].into_iter().zip(hands) {
                if held && tick >= 25 {
                    assert_eq!(rig.hand_state(side).phase, HandPhase::Reach);
                }
            }
        }
    }
}

#[test]
fn alternating_punches_after_a_two_hand_reach_preserve_standing_control() {
    for initial_delay in [1, 15, 32, 60] {
        let mut rig = ready();
        rig.reset();
        for tick in 0..(initial_delay + 420) {
            let t = tick - initial_delay;
            match t {
                0 => {
                    rig.set_hand(Hand::Left, true);
                    rig.set_hand(Hand::Right, true);
                }
                55 => rig.set_hand(Hand::Left, false),
                75 => rig.set_hand(Hand::Right, false),
                92 => rig.set_hand(Hand::Right, true),
                130 => rig.clear_hands(),
                148 => rig.set_hand(Hand::Left, true),
                200 => rig.set_hand(Hand::Left, false),
                _ => {}
            }
            rig.step();
            let chest = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[BodyPart::Chest.index()]];
            assert!(
                (chest.rotation() * Vector::y()).y > 0.6,
                "delay {initial_delay}, tick {t}, {:?}",
                chest.translation()
            );
        }
    }
}
