use brawler_engine::{BodyPart, Duel, FighterId, Hand, HandPhase, PhysicsConfig};

const OWNER: FighterId = FighterId::First;
const TARGET: FighterId = FighterId::Second;

fn approach(duel: &Duel) -> [f32; 2] {
    let pelvis = |id| {
        duel.body_states(id)[BodyPart::Pelvis.index()]
            .state
            .position
    };
    let a = pelvis(OWNER);
    let b = pelvis(TARGET);
    let (x, z) = (b[0] - a[0], b[2] - a[2]);
    let length = x.hypot(z).max(0.001);
    [x, z].map(|axis| (127. * axis / length).round() / 127.)
}

#[test]
fn either_free_hand_can_damage_an_opponent_without_replacing_the_other_hands_constraint() {
    for (hold, punch) in [(Hand::Left, Hand::Right), (Hand::Right, Hand::Left)] {
        let mut duel = Duel::skyline_match(PhysicsConfig::default()).unwrap();
        for _ in 0..180 {
            duel.step();
        }
        duel.set_hand(OWNER, hold, true);
        let mut attachment = None;
        let mut landed = false;
        for tick in 0..1200 {
            if tick % 3 == 0 {
                let axes = if attachment.is_some() {
                    [0.; 2]
                } else {
                    approach(&duel)
                };
                duel.set_input(OWNER, axes, false).unwrap();
                if attachment.is_some() && tick % 30 == 0 {
                    duel.set_hand(OWNER, punch, true);
                    duel.set_hand(OWNER, punch, false);
                }
            }
            let health = duel.vitals(TARGET).health;
            duel.step();
            let grip = duel
                .grip(OWNER, hold)
                .map(|g| (g.target, g.region, g.created_tick));
            if attachment.is_some() {
                assert_eq!(grip, attachment);
                if let Some(hit) = duel
                    .hits()
                    .iter()
                    .find(|hit| hit.source.hand() == Some(punch))
                {
                    assert_eq!((hit.attacker, hit.defender), (OWNER, TARGET));
                    assert!(hit.damage > 0.);
                    assert_eq!(duel.vitals(TARGET).health, (health - hit.damage).max(0.));
                    assert_eq!(duel.hand_state(OWNER, hold).phase, HandPhase::Gripping);
                    landed = true;
                    break;
                }
            }
            attachment = grip;
        }
        assert!(landed, "no {punch:?} contact while gripping with {hold:?}");
        assert_eq!(attachment.unwrap().0, TARGET);
        duel.set_hand(OWNER, hold, false);
        assert!(duel.grip(OWNER, hold).is_none());
    }
}
