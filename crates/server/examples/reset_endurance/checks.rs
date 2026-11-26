use brawler_engine::{
    ArenaBodyState, Consciousness, FighterId, Hand, HandPhase, MeleePhase, RagdollBodyState,
};
use brawler_server::Room;

pub struct Spawn {
    bodies: [Vec<RagdollBodyState>; 8],
    props: Vec<ArenaBodyState>,
}

impl Spawn {
    pub fn capture(room: &Room) -> Self {
        Self {
            bodies: FighterId::ALL.map(|id| room.game().body_states(id)),
            props: room.game().arena_states(),
        }
    }

    pub fn verify(&self, room: &Room, round: u32, scores: &[u8; 8]) {
        let game = room.game();
        let state = game.match_state().unwrap();
        assert_eq!(state.round_number, round);
        assert_eq!(state.phase, MeleePhase::Countdown);
        assert_eq!(state.remaining_ticks, 180);
        assert_eq!(state.winner, None);
        assert_eq!(state.scores(), scores);
        assert_eq!(game.tick(), 0);
        assert_eq!(game.arena_states(), self.props);
        for id in FighterId::ALL {
            assert_eq!(game.body_states(id), self.bodies[id as usize]);
            assert_eq!(room.applied(id), None);
            let vitals = game.vitals(id);
            assert_eq!(vitals.phase, Consciousness::Active);
            assert_eq!(
                (vitals.health, vitals.knockouts, game.stamina(id)),
                (100., 0, 100.)
            );
            for hand in [Hand::Left, Hand::Right] {
                assert_eq!(game.hand_state(id, hand).phase, HandPhase::Idle);
                assert!(game.grip(id, hand).is_none() && game.arena_grip(id, hand).is_none());
            }
        }
    }
}

pub fn verify_motion(room: &Room) -> bool {
    let game = room.game();
    let mut gripping = false;
    for id in FighterId::ALL {
        assert!(
            game.body_states(id)
                .iter()
                .all(|b| b.state.to_buffer().iter().all(|v| v.is_finite()))
        );
        for hand in [Hand::Left, Hand::Right] {
            if let Some(grip) = game.grip(id, hand) {
                assert_ne!(game.vitals(id).phase, Consciousness::Eliminated);
                assert_ne!(game.vitals(grip.target).phase, Consciousness::Eliminated);
                gripping = true;
            }
        }
    }
    gripping
}
