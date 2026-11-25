use brawler_engine::{Hand, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

fn main() {
    let traces = [Hand::Left, Hand::Right].map(|hand| {
        let mut rig = LocomotionRagdoll::strike_practice(
            PhysicsConfig::default(),
            LocomotionConfig::bundled().unwrap(),
        )
        .unwrap();
        let mut frames = vec![rig.practice_state().unwrap()];
        for tick in 1..=150 {
            if tick == 61 {
                rig.set_hand(hand, true);
                rig.set_hand(hand, false);
            }
            rig.step();
            frames.push(rig.practice_state().unwrap());
        }
        frames
    });
    println!("{}", serde_json::to_string(&traces).unwrap());
}
