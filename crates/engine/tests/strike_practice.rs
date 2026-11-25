use brawler_engine::{Hand, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

#[test]
fn each_hand_swings_the_suspended_pad_and_reset_restores_its_rest_pose() {
    for hand in [Hand::Left, Hand::Right] {
        let mut rig = LocomotionRagdoll::strike_practice(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
        )
        .unwrap();
        let start = rig.practice_state().unwrap();
        let [x, y, z] = [start[0], start[1], start[2]];
        let rest_length = start[11] - y;
        for _ in 0..120 {
            rig.step();
        }
        let rest = rig.practice_state().unwrap();
        assert!((rest[1] - y).abs() < 0.005 && (rest[2] - z).abs() < 0.005);
        rig.set_hand(hand, true);
        rig.set_hand(hand, false);
        assert_eq!(rig.practice_state().unwrap(), rest);
        let mut travel = 0.0_f32;
        for _ in 0..90 {
            rig.step();
            let frame = rig.practice_state().unwrap();
            let distance =
                ((frame[0] - x).powi(2) + (frame[1] - start[11]).powi(2) + (frame[2] - z).powi(2))
                    .sqrt();
            assert!((distance - rest_length).abs() < 0.02);
            travel = travel.max(z - frame[2]);
        }
        assert!(travel > 0.08, "{hand:?} moved pad {travel} m");
        rig.reset();
        assert_eq!(rig.practice_state(), Some(start));
        assert_eq!(rig.pose().rig().counts(), (17, 17, 15));
    }
}
