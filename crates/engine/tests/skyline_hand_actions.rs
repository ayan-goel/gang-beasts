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

fn hands(duel: &mut Duel, id: FighterId, held: bool) {
    for hand in [Hand::Left, Hand::Right] {
        duel.set_hand(id, hand, held);
    }
}

fn counter_grip_encounter() -> Duel {
    let mut duel = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    for _ in 0..180 {
        duel.step();
    }
    hands(&mut duel, OWNER, true);
    for tick in 0..600 {
        if tick % 3 == 0 {
            duel.set_input(OWNER, approach(&duel), false).unwrap();
        }
        duel.step();
        if [Hand::Left, Hand::Right]
            .iter()
            .all(|&hand| duel.grip(OWNER, hand).is_some())
        {
            break;
        }
    }
    duel.set_input(OWNER, [0., 0.], false).unwrap();
    hands(&mut duel, TARGET, true);
    for _ in 0..120 {
        duel.step();
        if [Hand::Left, Hand::Right]
            .iter()
            .any(|&hand| duel.grip(TARGET, hand).is_some())
        {
            break;
        }
    }
    duel.set_lift(OWNER, true);
    for _ in 0..17 {
        duel.step();
    }
    duel
}

#[test]
fn a_defenders_counter_grip_physically_limits_separation_after_a_two_hand_release() {
    let retreat = |keep_counter_grip: bool| {
        let mut duel = counter_grip_encounter();
        let identities = |d: &Duel, id| {
            [Hand::Left, Hand::Right].map(|hand| {
                d.grip(id, hand)
                    .map(|g| (g.target, g.region, g.created_tick))
            })
        };
        let counter = identities(&duel, TARGET);
        assert!(counter.iter().flatten().any(|grip| grip.0 == OWNER));
        assert!(
            identities(&duel, OWNER)
                .iter()
                .all(|g| g.is_some_and(|grip| grip.0 == TARGET))
        );
        let bodies = duel.body_states(TARGET);
        assert!(bodies[BodyPart::Pelvis.index()].state.position[1] > 0.90);
        let outward = approach(&duel).map(|axis| -axis);
        hands(&mut duel, OWNER, false);
        duel.set_lift(OWNER, false);
        if !keep_counter_grip {
            hands(&mut duel, TARGET, false);
        }
        assert_eq!(duel.body_states(TARGET), bodies);
        let remaining = if keep_counter_grip {
            counter
        } else {
            [None; 2]
        };
        duel.set_input(OWNER, outward, true).unwrap();
        for _ in 0..60 {
            duel.step();
            assert_eq!(identities(&duel, OWNER), [None; 2]);
            assert_eq!(identities(&duel, TARGET), remaining);
        }
        let pelvis = |id| {
            duel.body_states(id)[BodyPart::Pelvis.index()]
                .state
                .position
        };
        let a = pelvis(OWNER);
        let b = pelvis(TARGET);
        (bodies, (a[0] - b[0]).hypot(a[2] - b[2]))
    };
    let (held_pose, tethered) = retreat(true);
    let (free_pose, separated) = retreat(false);
    // The release pose and retreat are identical. Only the defender's decision to hold differs.
    assert_eq!(held_pose, free_pose);
    assert!(
        tethered < 1.5 && separated > tethered + 1.,
        "retreat with grip: {tethered} m; released: {separated} m"
    );
}
