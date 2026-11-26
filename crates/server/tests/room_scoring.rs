use brawler_engine::{Consciousness, FighterId, MeleePhase};
use brawler_protocol::{Button, InputFrame};
use brawler_server::Room;

#[test]
fn two_same_tick_ringouts_settle_once_for_a_draw_or_a_surviving_third_fighter() {
    for count in [2, 3] {
        let mut room = Room::skyline_with_players(count).unwrap();
        while room.game().match_state().unwrap().phase == MeleePhase::Countdown {
            room.step();
        }
        // These starts account for the asymmetric spawns. Both pelvises cross the fall plane
        // in the same solve using only ordinary run inputs refreshed at 10 Hz.
        for tick in 0..900u32 {
            let runners = [(FighterId::First, 221, -127), (FighterId::Second, 38, 127)];
            for (id, start, direction) in runners {
                if tick >= start && (tick - start).is_multiple_of(6) {
                    let run = Button::Run as u16;
                    let frame = InputFrame {
                        sequence: tick,
                        axes: [direction, 0],
                        held: run,
                        pressed: if tick == start { run } else { 0 },
                        ..Default::default()
                    };
                    room.submit(id, 1, &frame.encode().unwrap()).unwrap();
                }
            }
            for id in [FighterId::First, FighterId::Second] {
                assert_ne!(room.game().vitals(id).phase, Consciousness::Eliminated);
            }
            room.step();
            if room.game().match_state().unwrap().phase == MeleePhase::RoundResults {
                break;
            }
        }
        let winner = (count == 3).then_some(2);
        let scores: &[u8] = if count == 3 { &[0, 0, 1] } else { &[0, 0] };
        assert_eq!(room.game().consciousness_changes().len(), 2);
        for id in [FighterId::First, FighterId::Second] {
            assert_eq!(room.game().vitals(id).phase, Consciousness::Eliminated);
            assert!(room.game().fall_state(id).unwrap().below_plane);
        }
        for _ in 0..300 {
            let state = room.game().match_state().unwrap();
            assert_eq!(state.phase, MeleePhase::RoundResults);
            assert_eq!(state.winner, winner);
            assert_eq!(state.scores(), scores);
            room.step();
        }
        let next = room.game().match_state().unwrap();
        assert_eq!(next.phase, MeleePhase::Countdown);
        assert_eq!(next.round_number, 2);
        assert_eq!(next.winner, None);
        assert_eq!(next.scores(), scores);
    }
}
