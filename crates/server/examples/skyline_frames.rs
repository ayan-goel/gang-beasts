//! Prints full binary records as hex lines for the cross-runtime browser fixture.
use brawler_engine::{FighterId, Hand, MeleePhase};
use brawler_protocol::{Button, InputFrame};
use brawler_server::Room;
use std::io::{self, Write};

const CORAL: FighterId = FighterId::First;

fn print_frame(room: &Room) {
    let frame = room.capture().expect("valid authoritative state");
    let mut out = io::stdout().lock();
    for b in frame.for_recipient(CORAL) {
        write!(out, "{b:02x}").unwrap();
    }
    writeln!(out).unwrap();
}

fn main() {
    let mut room = Room::skyline().unwrap();
    print_frame(&room);
    for _ in 0..180 {
        room.step();
    }
    let hands = Button::LeftHand as u16 | Button::RightHand as u16;
    let mut frame = InputFrame {
        axes: [0, 127],
        held: hands,
        pressed: hands,
        ..Default::default()
    };
    for _ in 0..150 {
        room.submit(CORAL, 1, &frame.encode().unwrap()).unwrap();
        room.step();
        frame.sequence += 1;
        frame.pressed = 0;
        if [Hand::Left, Hand::Right]
            .into_iter()
            .any(|h| room.game().arena_grip(CORAL, h) == Some(7))
        {
            break;
        }
    }
    print_frame(&room);
    frame.axes = [-127, 0];
    frame.held = Button::Run as u16;
    frame.pressed = frame.held;
    frame.released = hands;
    for _ in 0..1500 {
        room.submit(CORAL, 1, &frame.encode().unwrap()).unwrap();
        room.step();
        frame.sequence += 1;
        frame.pressed = 0;
        frame.released = 0;
        if room.game().match_state().unwrap().phase == MeleePhase::RoundResults {
            break;
        }
    }
    print_frame(&room);
    for _ in 0..300 {
        room.step();
    }
    print_frame(&room);
    for _ in 0..(180 + 9000) {
        room.step();
    }
    print_frame(&room);
}
