use FighterId::{First, Second, Third};
use brawler_engine::{Consciousness, FighterId, MeleePhase};
use brawler_protocol::InputFrame;
use brawler_server::{Room, SubmitError};

#[test]
fn a_newcomer_replaces_a_departure_and_a_returning_winner_keeps_their_point() {
    let mut room = Room::skyline().unwrap();
    let next = [None, None, Some(Second)];
    for _ in 0..180 {
        room.step();
    }
    assert!(room.start_next_round(&next).is_err());
    let packet = InputFrame {
        sequence: 41,
        ..Default::default()
    }
    .encode()
    .unwrap();
    room.submit(Second, 1, &packet).unwrap();
    room.step_with_forfeits(1);
    room.detach(Second);
    for _ in 0..298 {
        room.step();
    }
    assert!(room.start_next_round(&next).is_err());
    room.step();
    let before = room.capture().unwrap().for_recipient(Second);
    for invalid in [
        vec![Some(First), None],
        vec![Some(Third), None],
        vec![Some(Second); 2],
        vec![None; 1],
        vec![None; 9],
    ] {
        assert!(room.start_next_round(&invalid).is_err());
        assert_eq!(room.capture().unwrap().for_recipient(Second), before);
    }
    let last_tick = room.tick();
    room.start_next_round(&next).unwrap();
    assert_eq!(room.tick(), last_tick + 1);
    let state = room.game().match_state().unwrap();
    assert_eq!(
        (state.round_number, state.phase, state.remaining_ticks),
        (2, MeleePhase::Countdown, 180)
    );
    assert_eq!(state.scores(), &[0, 0, 1]);
    assert_eq!(room.game().tick(), 0);
    assert!(!room.finished_for_lack_of_players());
    for &id in room.game().fighter_ids() {
        assert_eq!(room.game().vitals(id).phase, Consciousness::Active);
        assert_eq!(room.applied(id), None);
        assert_eq!(room.submit(id, 2, &packet), Err(SubmitError::Detached));
        room.attach(id);
        assert_eq!(room.submit(id, 1, &packet), Err(SubmitError::Round));
        assert_eq!(room.submit(id, 2, &packet), Err(SubmitError::Phase));
    }
    for _ in 0..180 {
        room.step();
    }
    room.submit(First, 2, &packet).unwrap();
    room.step();
    assert_eq!(room.applied(First), Some(41));
    assert_eq!(room.applied(Third), None);
}

#[test]
fn all_eight_fighters_spawn_at_the_boundary_and_a_won_match_cannot_be_extended() {
    let mut room = Room::skyline().unwrap();
    for _ in 0..180 {
        room.step();
    }
    room.step_with_forfeits(1);
    for _ in 1..300 {
        room.step();
    }
    let mut full = [None; 8];
    full[7] = Some(Second);
    room.start_next_round(&full).unwrap();
    let packet = InputFrame::default().encode().unwrap();
    assert_eq!(
        room.submit(FighterId::Eighth, 2, &packet),
        Err(SubmitError::Phase)
    );
    let full_snapshot = room.capture().unwrap().for_spectator();
    assert_eq!(room.game().fighter_ids().len(), 8);
    assert_eq!(
        u16::from_le_bytes(full_snapshot[16..18].try_into().unwrap()),
        129
    );
    for score in [2, 3] {
        for _ in 0..180 {
            room.step();
        }
        room.step_with_forfeits(if score == 2 { 0x7f } else { 1 });
        for _ in 1..300 {
            room.step();
        }
        if score == 2 {
            room.start_next_round(&[None, Some(FighterId::Eighth)])
                .unwrap();
            assert_eq!(room.submit(Second, 3, &packet), Err(SubmitError::Phase));
            assert_eq!(room.game().match_state().unwrap().scores(), &[0, 2]);
            assert_eq!(room.game().fighter_ids().len(), 2);
        } else {
            let before = room.capture().unwrap().for_spectator();
            assert!(room.start_next_round(&[None, Some(Second)]).is_err());
            assert_eq!(room.capture().unwrap().for_spectator(), before);
            assert_eq!(room.game().match_state().unwrap().scores(), &[0, 3]);
        }
    }
}
