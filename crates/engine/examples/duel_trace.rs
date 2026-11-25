use brawler_engine::{Duel, FighterId, Hand, PhysicsConfig};
use serde_json::json;

fn main() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    let players = [FighterId::First, FighterId::Second];
    let mut frames = Vec::new();
    let mut hits = Vec::new();
    for tick in 0..=130 {
        if tick == 61 {
            for id in players {
                duel.set_hand(id, Hand::Left, true);
                duel.set_hand(id, Hand::Left, false);
            }
        }
        if tick == 92 {
            duel.set_input(FighterId::Second, [-1., 0.], false).unwrap();
            duel.set_jump(FighterId::First, true);
        }
        if tick > 0 {
            duel.step();
        }
        frames.push(players.map(|id| {
            duel.body_states(id)
                .into_iter()
                .flat_map(|b| b.state.to_buffer())
                .collect::<Vec<_>>()
        }));
        for h in duel.hits() {
            hits.push(json!([
                h.tick.to_string(),
                h.attacker as usize,
                h.defender as usize,
                h.attack_id.to_string(),
                h.hand as usize,
                h.region,
                h.damage,
                h.energy_j,
                h.closing_speed
            ]));
        }
    }
    println!("{}", json!({"frames":frames,"hits":hits}));
}
