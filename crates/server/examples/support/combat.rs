use brawler_engine::{BodyPart, FighterId, MeleePhase};
use brawler_protocol::{Button, INPUT_BYTES, InputFrame};
use brawler_server::Room;

#[derive(Default)]
pub struct Drivers {
    previous: [u16; 8],
}

impl Drivers {
    pub fn inputs(&mut self, room: &Room) -> Option<[[u8; INPUT_BYTES]; 8]> {
        let state = room.game().match_state().unwrap();
        if state.phase != MeleePhase::Playing {
            self.previous.fill(0);
            return None;
        }
        if !room.tick().is_multiple_of(6) {
            return None;
        }
        Some(FighterId::ALL.map(|id| {
            let i = id as usize;
            let pulse = room.tick() / 6;
            let phase = (pulse + 3 * i as u64) % 24;
            let mut held = 0;
            if phase < 14 {
                held |= Button::LeftHand as u16;
            }
            if phase == 2 || phase == 18 {
                held |= Button::RightHand as u16;
            }
            if (8..14).contains(&phase) {
                held |= Button::Lift as u16;
            }
            let bodies = room.game().body_states(id);
            let [x, _, z] = bodies[BodyPart::Pelvis.index()].state.position;
            let length = x.hypot(z).max(0.5);
            let axes = [-x / length, -z / length].map(|axis| (127. * axis).round() as i8);
            let previous = std::mem::replace(&mut self.previous[i], held);
            InputFrame {
                sequence: pulse as u32,
                tick: room.tick() as u32,
                axes,
                held,
                pressed: held & !previous,
                released: previous & !held,
            }
            .encode()
            .unwrap()
        }))
    }
}
