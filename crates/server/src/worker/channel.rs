use crate::{RoomFrame, SubmitError};
use brawler_engine::FighterId;
use brawler_protocol::{INPUT_BYTES, InputFrame};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    Capacity,
    Busy,
    Closed,
    Invalid(SubmitError),
    Initialization,
}

pub(super) enum Command {
    Input {
        generation: u64,
        round: u64,
        bytes: [u8; INPUT_BYTES],
        received: Instant,
    },
    Attach,
    Detach,
}

pub(super) struct Shared {
    pub latest: Mutex<Option<Arc<RoomFrame>>>,
    pub closed: AtomicBool,
    pub rejected: AtomicU64,
    pub owners: [AtomicU64; 2],
    pub delayed: AtomicU64,
}

pub struct RoomClient {
    senders: [SyncSender<Command>; 2],
    pub(super) shared: Arc<Shared>,
}

impl RoomClient {
    pub(super) fn channel() -> (Self, [Receiver<Command>; 2]) {
        let [(a, ra), (b, rb)] = std::array::from_fn(|_| sync_channel(8));
        let shared = Arc::new(Shared {
            latest: Mutex::default(),
            closed: AtomicBool::new(false),
            owners: Default::default(),
            rejected: AtomicU64::new(0),
            delayed: AtomicU64::new(0),
        });
        (
            Self {
                senders: [a, b],
                shared,
            },
            [ra, rb],
        )
    }

    pub fn latest(&self) -> Option<Arc<RoomFrame>> {
        self.shared.latest.lock().unwrap().clone()
    }

    pub fn rejected_inputs(&self) -> u64 {
        self.shared.rejected.load(Ordering::Relaxed)
    }
    pub fn delayed_passes(&self) -> u64 {
        self.shared.delayed.load(Ordering::Relaxed)
    }

    pub fn input(&self, id: FighterId, round: u64, bytes: &[u8]) -> Result<(), DispatchError> {
        self.input_for(id, 0, round, bytes)
    }

    pub(crate) fn bind(&self, id: FighterId, generation: u64) {
        self.shared.owners[id as usize].store(generation, Ordering::Release);
    }

    pub(crate) fn unbind(&self, id: FighterId, generation: u64) {
        let _ = self.shared.owners[id as usize].compare_exchange(
            generation,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub(crate) fn input_for(
        &self,
        id: FighterId,
        generation: u64,
        round: u64,
        bytes: &[u8],
    ) -> Result<(), DispatchError> {
        InputFrame::decode(bytes).map_err(|e| DispatchError::Invalid(SubmitError::Invalid(e)))?;
        self.send(
            id,
            Command::Input {
                generation,
                round,
                bytes: bytes.try_into().expect("validated input length"),
                received: Instant::now(),
            },
        )
    }
    pub fn attach(&self, id: FighterId) -> Result<(), DispatchError> {
        self.send(id, Command::Attach)
    }
    pub fn detach(&self, id: FighterId) -> Result<(), DispatchError> {
        self.send(id, Command::Detach)
    }

    fn send(&self, id: FighterId, command: Command) -> Result<(), DispatchError> {
        if self.shared.closed.load(Ordering::Acquire) {
            return Err(DispatchError::Closed);
        }
        self.senders[id as usize]
            .try_send(command)
            .map_err(|e| match e {
                TrySendError::Full(_) => DispatchError::Busy,
                TrySendError::Disconnected(_) => DispatchError::Closed,
            })
    }
}

impl Drop for RoomClient {
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
    }
}
