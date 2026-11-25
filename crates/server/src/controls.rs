use brawler_engine::{Duel, FighterId, Hand};
use brawler_protocol::{Button, InputFrame};

pub(crate) fn apply(game: &mut Duel, id: FighterId, frame: InputFrame) {
    game.set_input(id, frame.direction(), frame.held & Button::Run as u16 != 0)
        .expect("decoded axes have finite magnitude");
    game.set_lift(id, frame.held & Button::Lift as u16 != 0);
    apply_button(frame, Button::Jump, |down| game.set_jump(id, down));
    apply_button(frame, Button::LeftHand, |down| {
        game.set_hand(id, Hand::Left, down)
    });
    apply_button(frame, Button::RightHand, |down| {
        game.set_hand(id, Hand::Right, down)
    });
}

fn apply_button(frame: InputFrame, button: Button, mut set: impl FnMut(bool)) {
    for down in frame.edges(button) {
        set(down);
    }
    // A neutral heartbeat repairs a lost release. A held heartbeat after expiry
    // needs a fresh press edge before it can start another jump or strike.
    if (frame.held | frame.pressed | frame.released) & button as u16 == 0 {
        set(false);
    }
}

pub(crate) fn neutralize(game: &mut Duel, id: FighterId) {
    game.clear_hands(id);
    game.clear_jump(id);
    game.set_lift(id, false);
    game.set_input(id, [0.; 2], false)
        .expect("neutral axes are finite");
}
