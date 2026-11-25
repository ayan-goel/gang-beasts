use brawler_engine::{Consciousness, FighterId, MeleePhase};
use brawler_server::Room;

#[test]
fn simultaneous_forfeits_settle_once_and_cannot_farm_points_in_empty_rounds() {
    for (mask, winner) in [(0b101, Some(1)), (0b111, None)] {
        let mut room = Room::skyline_with_players(3).unwrap();
        for _ in 0..180 {
            room.step();
        }
        room.step_with_forfeits(mask);
        let state = room.game().match_state().unwrap();
        assert_eq!(state.phase, MeleePhase::RoundResults);
        assert_eq!(state.winner, winner);
        let scores = state.scores().to_vec();
        assert_eq!(scores.iter().sum::<u8>(), u8::from(winner.is_some()));
        for &id in room.game().fighter_ids() {
            assert_eq!(
                room.game().vitals(id).phase == Consciousness::Eliminated,
                mask & (1 << id as usize) != 0
            );
        }
        for _ in 0..1000 {
            room.step_with_forfeits(mask);
        }
        assert_eq!(room.game().match_state().unwrap().round_id, 1);
        assert_eq!(room.game().match_state().unwrap().scores(), scores);
        assert!(room.finished_for_lack_of_players());
    }
}

#[test]
fn other_players_can_finish_and_start_another_round_without_the_forfeited_fighter() {
    use brawler_protocol::{Button, InputFrame};
    let mut room = Room::skyline_with_players(3).unwrap();
    for _ in 0..181 {
        room.step_with_forfeits(0b100);
    }
    assert_eq!(
        room.game().vitals(FighterId::Third).phase,
        Consciousness::Eliminated
    );
    for sequence in 0..1200 {
        let run = Button::Run as u16;
        let packet = InputFrame {
            sequence,
            axes: [-127, 0],
            held: run,
            pressed: if sequence == 0 { run } else { 0 },
            ..Default::default()
        };
        room.submit(FighterId::First, 1, &packet.encode().unwrap())
            .unwrap();
        room.step();
        if room.game().match_state().unwrap().phase == MeleePhase::RoundResults {
            break;
        }
    }
    assert_eq!(room.game().match_state().unwrap().winner, Some(1));
    for _ in 0..480 {
        room.step();
    }
    assert_eq!(room.game().match_state().unwrap().round_id, 2);
    assert_eq!(room.game().match_state().unwrap().scores(), &[0, 1, 0]);
    assert_eq!(
        room.game().vitals(FighterId::Third).phase,
        Consciousness::Eliminated
    );
    assert!(!room.finished_for_lack_of_players());
}
