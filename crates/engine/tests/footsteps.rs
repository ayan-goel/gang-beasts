use brawler_engine::{Duel, FighterId, PhysicsConfig};

#[test]
fn roof_travel_produces_spaced_steps_while_rest_and_a_standing_jump_stay_quiet() {
    for running in [false, true] {
        let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
        let walker = FighterId::First;
        for _ in 0..120 {
            game.step();
        }
        assert_eq!(game.last_footstep(walker), None);
        game.set_jump(walker, true);
        game.set_jump(walker, false);
        for _ in 0..150 {
            game.step();
        }
        assert_eq!(game.last_footstep(walker), None);
        game.set_input(walker, [0., 1.], running).unwrap();
        let mut ticks = Vec::new();
        for _ in 0..100 {
            game.step();
            if game.last_footstep(walker) == Some(game.tick()) {
                ticks.push(game.tick());
            }
        }
        assert!(ticks.len() >= 3, "running {running}, ticks {ticks:?}");
        assert!(ticks.windows(2).all(|w| w[1] - w[0] >= 9));
        assert_eq!(game.last_footstep(FighterId::Second), None);
        game.set_input(walker, [0., 0.], false).unwrap();
        let last = game.last_footstep(walker);
        for _ in 0..90 {
            game.step();
        }
        assert_eq!(game.last_footstep(walker), last);
        game.reset();
        assert_eq!(game.last_footstep(walker), None);
    }
}

#[test]
fn moving_in_the_air_cannot_sound_a_step() {
    use brawler_engine::BodyPart;
    let mut game = Duel::skyline(PhysicsConfig::default()).unwrap();
    let walker = FighterId::First;
    for _ in 0..90 {
        game.step();
    }
    game.set_input(walker, [0., 1.], false).unwrap();
    for _ in 0..30 {
        game.step();
    }
    game.set_jump(walker, true);
    game.set_jump(walker, false);
    let mut airborne = 0;
    for _ in 0..80 {
        game.step();
        let pose = game.body_states(walker);
        if [BodyPart::LeftFoot, BodyPart::RightFoot]
            .iter()
            .all(|p| pose[p.index()].state.position[1] > 0.30)
        {
            airborne += 1;
            assert_ne!(game.last_footstep(walker), Some(game.tick()));
        }
    }
    assert!(
        airborne >= 10,
        "expected a jump with both feet clear, got {airborne} ticks"
    );
}
