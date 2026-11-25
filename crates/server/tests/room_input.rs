use brawler_engine::{Duel, FighterId, Hand, HandPhase, MeleePhase, PhysicsConfig};
use brawler_protocol::{Button, InputError, InputFrame};
use brawler_server::{Room, SubmitError};

const CORAL: FighterId = FighterId::First;
const BLUE: FighterId = FighterId::Second;

fn playing() -> Room {
    let mut room = Room::skyline().unwrap();
    for _ in 0..180 {
        room.step();
    }
    assert_eq!(
        room.game().match_state().unwrap().phase,
        MeleePhase::Playing
    );
    room
}

fn frame(sequence: u32) -> InputFrame {
    InputFrame {
        sequence,
        tick: u32::MAX,
        ..Default::default()
    }
}

fn send(room: &mut Room, id: FighterId, input: InputFrame) -> Result<(), SubmitError> {
    room.submit(id, 1, &input.encode().unwrap())
}

#[test]
fn packet_driven_world_matches_direct_control_for_both_fighters() {
    let mut room = playing();
    let mut direct = Duel::skyline_match(PhysicsConfig::default()).unwrap();
    for _ in 0..180 {
        direct.step();
    }
    let left = Button::LeftHand as u16;
    let right = Button::RightHand as u16;
    // A tap and a separate hold arrive in the same tick, after Blue's movement.
    send(
        &mut room,
        BLUE,
        InputFrame {
            axes: [-127, 0],
            held: Button::Run as u16,
            ..frame(99)
        },
    )
    .unwrap();
    send(
        &mut room,
        CORAL,
        InputFrame {
            pressed: left,
            released: left,
            ..frame(u32::MAX)
        },
    )
    .unwrap();
    send(
        &mut room,
        CORAL,
        InputFrame {
            axes: [0, 127],
            pressed: right,
            held: right,
            ..frame(0)
        },
    )
    .unwrap();
    assert_eq!(room.applied(CORAL), None);
    assert_eq!(
        room.game().hand_state(CORAL, Hand::Left).phase,
        HandPhase::Idle
    );
    direct.set_hand(CORAL, Hand::Left, true);
    direct.set_hand(CORAL, Hand::Left, false);
    direct.set_hand(CORAL, Hand::Right, true);
    direct.set_input(CORAL, [0., 1.], false).unwrap();
    direct.set_input(BLUE, [-1., 0.], true).unwrap();
    for elapsed in 0..35 {
        if elapsed == 15 {
            for id in [CORAL, BLUE] {
                direct.clear_hands(id);
                direct.clear_jump(id);
                direct.set_lift(id, false);
                direct.set_input(id, [0.; 2], false).unwrap();
            }
        }
        direct.step();
        room.step();
        for id in [CORAL, BLUE] {
            assert_eq!(
                room.game().body_states(id),
                direct.body_states(id),
                "tick {elapsed}"
            );
        }
        if elapsed == 0 {
            assert_ne!(
                room.game().hand_state(CORAL, Hand::Left).phase,
                HandPhase::Idle
            );
            assert_eq!(
                room.game().hand_state(BLUE, Hand::Left).phase,
                HandPhase::Idle
            );
        }
    }
    assert_eq!(room.tick(), 215);
    assert_eq!(room.applied(CORAL), Some(0));
    assert_eq!(room.applied(BLUE), Some(99));
}

#[test]
fn admission_limits_and_rejections_do_not_consume_sequence_or_queue_space() {
    let mut room = Room::skyline().unwrap();
    assert_eq!(send(&mut room, CORAL, frame(1)), Err(SubmitError::Phase));
    for _ in 0..180 {
        room.step();
    }
    assert_eq!(
        room.submit(CORAL, 0, &frame(1).encode().unwrap()),
        Err(SubmitError::Round)
    );
    assert_eq!(
        room.submit(CORAL, 1, &[1]),
        Err(SubmitError::Invalid(InputError::Length))
    );
    for seq in 0..8 {
        send(&mut room, CORAL, frame(seq)).unwrap();
    }
    assert_eq!(send(&mut room, CORAL, frame(8)), Err(SubmitError::Full));
    room.step();
    assert_eq!(room.applied(CORAL), Some(7));
    for seq in [7, 5, 128, (1 << 31) + 7] {
        assert_eq!(
            send(&mut room, CORAL, frame(seq)),
            Err(SubmitError::Sequence)
        );
    }
    send(&mut room, CORAL, frame(8)).unwrap();
    room.step();
    assert_eq!(room.applied(CORAL), Some(8));
}

#[test]
fn detachment_discards_pending_edges_and_requires_a_fresh_binding() {
    let mut room = playing();
    let hand = Button::RightHand as u16;
    send(
        &mut room,
        BLUE,
        InputFrame {
            held: hand,
            pressed: hand,
            ..frame(10)
        },
    )
    .unwrap();
    room.detach(BLUE);
    assert_eq!(send(&mut room, BLUE, frame(11)), Err(SubmitError::Detached));
    room.step();
    assert_eq!(room.applied(BLUE), None);
    assert_eq!(
        room.game().hand_state(BLUE, Hand::Right).phase,
        HandPhase::Idle
    );
    room.attach(BLUE);
    assert_eq!(send(&mut room, BLUE, frame(10)), Err(SubmitError::Sequence));
    send(&mut room, BLUE, frame(11)).unwrap();
    room.step();
    assert_eq!(room.applied(BLUE), Some(11));
}
