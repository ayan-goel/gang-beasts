use crate::VERSION;

pub const INPUT_BYTES: usize = 18;
pub const INPUT_KIND: u8 = 1;
const KNOWN_BUTTONS: u16 = (1 << 10) - 1;

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Jump = 1,
    Run = 2,
    LeftHand = 4,
    RightHand = 8,
    Lift = 16,
    Headbutt = 32,
    Kick = 64,
    Crouch = 128,
    Emote = 256,
    Interact = 512,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    Length,
    Kind,
    Version,
    Axis,
    Buttons,
    Edges,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputFrame {
    pub sequence: u32,
    pub tick: u32,
    pub axes: [i8; 2],
    pub held: u16,
    pub pressed: u16,
    pub released: u16,
}

impl InputFrame {
    fn validate(&self) -> Result<(), InputError> {
        if self.axes.contains(&i8::MIN) {
            return Err(InputError::Axis);
        }
        if (self.held | self.pressed | self.released) & !KNOWN_BUTTONS != 0 {
            return Err(InputError::Buttons);
        }
        // A lone edge determines the final state. Both edges are legal in either final state.
        if (self.pressed & !self.released & !self.held)
            | (self.released & !self.pressed & self.held)
            != 0
        {
            return Err(InputError::Edges);
        }
        Ok(())
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, InputError> {
        let bytes: &[u8; INPUT_BYTES] = bytes.try_into().map_err(|_| InputError::Length)?;
        if bytes[0] != INPUT_KIND {
            return Err(InputError::Kind);
        }
        if bytes[1] != VERSION {
            return Err(InputError::Version);
        }
        let u16_at = |i| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        let u32_at = |i| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
        let frame = Self {
            sequence: u32_at(2),
            tick: u32_at(6),
            axes: [bytes[10] as i8, bytes[11] as i8],
            held: u16_at(12),
            pressed: u16_at(14),
            released: u16_at(16),
        };
        frame.validate()?;
        Ok(frame)
    }

    pub fn encode(&self) -> Result<[u8; INPUT_BYTES], InputError> {
        self.validate()?;
        let mut bytes = [0; INPUT_BYTES];
        bytes[..2].copy_from_slice(&[INPUT_KIND, VERSION]);
        bytes[2..6].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[6..10].copy_from_slice(&self.tick.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.axes.map(|v| v as u8));
        bytes[12..14].copy_from_slice(&self.held.to_le_bytes());
        bytes[14..16].copy_from_slice(&self.pressed.to_le_bytes());
        bytes[16..18].copy_from_slice(&self.released.to_le_bytes());
        Ok(bytes)
    }

    pub fn direction(&self) -> [f32; 2] {
        let [x, z] = self.axes.map(f32::from);
        let scale = x.hypot(z).max(127.);
        [x / scale, z / scale]
    }

    pub fn edges(&self, button: Button) -> impl Iterator<Item = bool> {
        let bit = button as u16;
        let final_down = self.held & bit != 0;
        let events = match (self.pressed & bit != 0, self.released & bit != 0) {
            (true, true) => [Some(!final_down), Some(final_down)],
            (true, false) => [Some(true), None],
            (false, true) => [Some(false), None],
            (false, false) => [None, None],
        };
        events.into_iter().flatten()
    }
}
