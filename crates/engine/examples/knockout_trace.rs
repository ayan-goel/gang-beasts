use brawler_engine::{Consciousness, Duel, FighterId, Hand, PhysicsConfig};
use serde_json::json;

fn main() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    let attacker = FighterId::First;
    let defender = FighterId::Second;
    let players = [attacker, defender];
    let mut frames = Vec::new();
    let mut fighting = true;
    for tick in 0..781 {
        let mut direction = None;
        let mut taps = Vec::new();
        if fighting && tick >= 65 {
            let pelvis = |id| {
                duel.body_states(id)[brawler_engine::BodyPart::Pelvis.index()]
                    .state
                    .position
            };
            let (a, b) = (pelvis(attacker), pelvis(defender));
            let (x, z) = (a[0] - b[0], a[2] - b[2]);
            let length = (x * x + z * z).sqrt().max(0.001);
            direction = Some([x / length, z / length]);
            duel.set_input(defender, direction.unwrap(), false).unwrap();
            duel.set_input(attacker, [0., 0.], false).unwrap();
            for (hand, offset) in [(Hand::Left, 0), (Hand::Right, 13)] {
                if (tick - 65) % 26 == offset {
                    taps.push(hand as u8);
                    duel.set_hand(attacker, hand, true);
                    duel.set_hand(attacker, hand, false);
                }
            }
        }
        duel.step();
        if duel.vitals(defender).phase == Consciousness::KnockedOut {
            fighting = false;
        }
        let states = players.map(|id| {
            let v = duel.vitals(id);
            json!([v.phase, v.health, v.knockouts, v.since_tick.to_string()])
        });
        let changes: Vec<_> = duel
            .consciousness_changes()
            .iter()
            .map(|(id, phase)| json!([*id as u8, phase]))
            .collect();
        let bodies = players.map(|id| {
            duel.body_states(id)
                .into_iter()
                .flat_map(|b| b.state.to_buffer())
                .collect::<Vec<_>>()
        });
        frames.push(json!({"direction":direction, "taps":taps, "states":states,"changes":changes,"bodies":bodies}));
    }
    assert_eq!(duel.vitals(defender).knockouts, 1);
    assert!(
        duel.body_states(defender)[brawler_engine::BodyPart::Pelvis.index()]
            .state
            .position[1]
            > 0.75
    );
    println!("{}", serde_json::to_string(&frames).unwrap());
}
