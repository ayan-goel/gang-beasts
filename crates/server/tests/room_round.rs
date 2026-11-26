use brawler_engine::{FighterId, Hand, MeleePhase};
use brawler_protocol::{Button, InputFrame};
use brawler_server::{Room, SubmitError};

const CORAL: FighterId = FighterId::First;

fn playing() -> Room {
    let mut room = Room::skyline().unwrap();
    for _ in 0..180 {
        room.step();
    }
    room
}

#[test]
fn a_network_ringout_scores_once_and_previous_round_input_cannot_move_the_new_body() {
    let mut room = playing();
    let run = Button::Run as u16;
    let mut movement = InputFrame {
        held: run,
        pressed: run,
        axes: [-127, 0],
        ..Default::default()
    };
    for seq in 0..1200 {
        movement.sequence = seq;
        room.submit(CORAL, 1, &movement.encode().unwrap()).unwrap();
        room.step();
        movement.pressed = 0;
        if room.game().match_state().unwrap().phase == MeleePhase::RoundResults {
            break;
        }
    }
    let result = room.game().match_state().unwrap();
    assert_eq!(result.phase, MeleePhase::RoundResults);
    assert_eq!(result.winner, Some(1));
    assert_eq!(result.scores(), &[0, 1]);
    assert_eq!(room.applied(CORAL), Some(movement.sequence));
    let end = room.tick();
    for _ in 0..480 {
        room.step();
    }
    assert_eq!(room.tick(), end + 480);
    assert_eq!(room.game().tick(), 0);
    assert_eq!(room.game().match_state().unwrap().round_id, 2);
    assert_eq!(room.applied(CORAL), None);
    assert_eq!(
        room.submit(CORAL, 1, &movement.encode().unwrap()),
        Err(SubmitError::Round)
    );
    room.submit(CORAL, 2, &InputFrame::default().encode().unwrap())
        .unwrap();
    room.step();
    assert_eq!(room.applied(CORAL), Some(0));
    assert_eq!(room.game().match_state().unwrap().scores(), &[0, 1]);
}

#[test]
fn duplicate_heartbeats_cannot_keep_a_physical_grip_alive() {
    let mut room = playing();
    let hands = Button::LeftHand as u16 | Button::RightHand as u16;
    let mut input = InputFrame {
        axes: [0, 127],
        held: hands,
        pressed: hands,
        ..Default::default()
    };
    let holds_crate = |room: &Room| {
        [Hand::Left, Hand::Right]
            .into_iter()
            .any(|h| room.game().arena_grip(CORAL, h) == Some(7))
    };
    for seq in 0..150 {
        input.sequence = seq;
        room.submit(CORAL, 1, &input.encode().unwrap()).unwrap();
        room.step();
        input.pressed = 0;
        if holds_crate(&room) {
            break;
        }
    }
    assert!(holds_crate(&room));
    input.axes = [0; 2];
    input.sequence += 1;
    let bytes = input.encode().unwrap();
    room.submit(CORAL, 1, &bytes).unwrap();
    for _ in 0..15 {
        assert_eq!(room.submit(CORAL, 1, &bytes), Err(SubmitError::Sequence));
        room.step();
        assert!(holds_crate(&room));
    }
    room.step();
    assert!(!holds_crate(&room));
    input.sequence += 1;
    room.submit(CORAL, 1, &input.encode().unwrap()).unwrap();
    room.step();
    assert!(!holds_crate(&room));
}

#[test]
fn a_headbutt_tap_reaches_only_its_owners_native_controller() {
    let mut room = playing();
    let bit = Button::Headbutt as u16;
    let tap = InputFrame {
        pressed: bit,
        released: bit,
        ..Default::default()
    };
    room.submit(CORAL, 1, &tap.encode().unwrap()).unwrap();
    room.step();
    assert!(room.game().stamina(CORAL) < 95.);
    assert_eq!(room.game().stamina(FighterId::Second), 100.);
}
