use brawler_engine::{Duel, FighterId, Hand, PhysicsConfig};
use serde_json::json;

fn main() {
    let mut duel = Duel::new(PhysicsConfig::default()).unwrap();
    let players = [FighterId::First, FighterId::Second];
    let mut frames = Vec::new();
    let mut hits = Vec::new();
    let mut grips = Vec::new();
    for tick in 0..=240 {
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
        match tick {
            131 => {
                duel.reset();
                duel.set_input(FighterId::Second, [0., 1.], false).unwrap();
                for id in players {
                    duel.set_hand(id, Hand::Left, true);
                }
            }
            156 => duel.set_input(FighterId::Second, [0., 0.], false).unwrap(),
            205 => duel.clear_hands(FighterId::First),
            220 => duel.set_hand(FighterId::Second, Hand::Left, false),
            _ => {}
        }
        if tick > 0 {
            duel.step();
        }
        grips.push(players.map(|id| {
            [Hand::Left, Hand::Right].map(|hand| duel.grip(id, hand).map(|g| g.region.index()))
        }));
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
    println!("{}", json!({"frames":frames,"hits":hits,"grips":grips}));
}
