use brawler_protocol::{InputError, InputFrame, is_newer};
use std::collections::VecDeque;

const CAPACITY: usize = 8;
const MAX_FORWARD: u32 = 120;
pub(crate) const STALE_TICKS: u64 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmitError {
    Detached,
    Round,
    Phase,
    Invalid(InputError),
    Sequence,
    Full,
}

#[derive(Default)]
pub(crate) struct Inbox {
    pub pending: VecDeque<InputFrame>,
    last_sequence: Option<u32>,
    pub applied: Option<u32>,
    pub last_received: Option<u64>,
}

impl Inbox {
    pub fn enqueue(&mut self, bytes: &[u8], tick: u64) -> Result<(), SubmitError> {
        let frame = InputFrame::decode(bytes).map_err(SubmitError::Invalid)?;
        if self.last_sequence.is_some_and(|last| {
            !is_newer(frame.sequence, last) || frame.sequence.wrapping_sub(last) > MAX_FORWARD
        }) {
            return Err(SubmitError::Sequence);
        }
        // Each slot has its own work budget. Rejections cannot extend a held input's lifetime.
        if self.pending.len() == CAPACITY {
            return Err(SubmitError::Full);
        }
        self.pending.push_back(frame);
        self.last_sequence = Some(frame.sequence);
        self.last_received = Some(tick);
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.pending.clear();
        self.last_received = None;
    }
}
