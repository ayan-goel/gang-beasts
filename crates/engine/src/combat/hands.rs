use super::HandTuning;
use crate::{BodyPart, CharacterError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    pub(crate) fn body(self) -> BodyPart {
        match self {
            Self::Left => BodyPart::LeftHand,
            Self::Right => BodyPart::RightHand,
        }
    }

    pub fn parts(self) -> [BodyPart; 2] {
        match self {
            Self::Left => [BodyPart::LeftUpperArm, BodyPart::LeftForearm],
            Self::Right => [BodyPart::RightUpperArm, BodyPart::RightForearm],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum HandPhase {
    #[default]
    Idle,
    WindUp,
    Strike,
    Recovery,
    Reach,
    Gripping,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HandState {
    pub phase: HandPhase,
    pub attack_id: u64,
}

#[derive(Default)]
struct Action {
    state: HandState,
    held: bool,
    pending: bool,
    age: u8,
}

pub(crate) struct Hands {
    actions: [Action; 2],
    next_id: u64,
    pub tuning: HandTuning,
}

impl Hands {
    pub fn new() -> Result<Self, CharacterError> {
        Ok(Self {
            actions: Default::default(),
            next_id: 1,
            tuning: HandTuning::bundled()?,
        })
    }
    pub fn state(&self, hand: Hand) -> HandState {
        self.actions[hand as usize].state
    }

    pub fn set(&mut self, hand: Hand, held: bool) {
        let action = &mut self.actions[hand as usize];
        if held && !action.held && (action.age == 0 || action.age == self.tuning.length()) {
            action.pending = true;
        }
        action.held = held;
    }

    pub fn release_inputs(&mut self) {
        for action in &mut self.actions {
            action.pending = false;
            action.held = false;
            // Focus loss retracts a reach, but an accepted strike retains its cooldown.
            if action.state.phase == HandPhase::Reach {
                *action = Action::default();
            }
        }
    }
    pub fn reset(&mut self) {
        self.clear();
        self.next_id = 1;
    }
    pub fn clear(&mut self) {
        self.actions = Default::default();
    }

    pub fn advance(&mut self, hand: Hand) -> Option<[f32; 2]> {
        let action = &mut self.actions[hand as usize];
        let tuning = &self.tuning;
        if std::mem::take(&mut action.pending) {
            action.age = 1;
            action.state.attack_id = self.next_id;
            self.next_id = self
                .next_id
                .checked_add(1)
                .expect("attack ID space exhausted");
        } else if action.age != 0 && action.age < tuning.length() {
            action.age += 1;
        } else {
            // A completed strike can leave the same hand seeking a grip. Holding
            // input never allocates another attack ID or reopens its damage window.
            if action.held && action.state.attack_id != 0 {
                action.state.phase = HandPhase::Reach;
                return Some(tuning.reach_pose_rad);
            }
            *action = Action::default();
            return None;
        }
        if action.age <= tuning.windup_ticks {
            action.state.phase = HandPhase::WindUp;
            Some(tuning.windup_pose_rad)
        } else if action.age <= tuning.windup_ticks + tuning.strike_ticks {
            action.state.phase = HandPhase::Strike;
            Some(tuning.strike_pose_rad)
        } else {
            action.state.phase = HandPhase::Recovery;
            let remaining =
                f32::from(tuning.length() - action.age) / f32::from(tuning.recovery_ticks);
            // A held punch settles directly into reach. Folding both arms to the
            // sides and extending them again would kick the torso forward twice.
            let end = if action.held {
                tuning.reach_pose_rad
            } else {
                [0.; 2]
            };
            Some(std::array::from_fn(|i| {
                end[i] + (tuning.strike_pose_rad[i] - end[i]) * remaining
            }))
        }
    }
}
