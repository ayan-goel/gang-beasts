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
    pub return_to_lobby: AtomicBool,
    pub owners: [AtomicU64; FighterId::ALL.len()],
    pub delayed: AtomicU64,
}

pub struct RoomClient {
    senders: Vec<SyncSender<Command>>,
    pub(super) shared: Arc<Shared>,
}

impl RoomClient {
    pub(super) fn channel(count: usize) -> (Self, Vec<Receiver<Command>>) {
        let (senders, incoming) = (0..count).map(|_| sync_channel(8)).unzip();
        let shared = Arc::new(Shared {
            latest: Mutex::default(),
            closed: AtomicBool::new(false),
            owners: Default::default(),
            return_to_lobby: AtomicBool::new(false),
            rejected: AtomicU64::new(0),
            delayed: AtomicU64::new(0),
        });
        (Self { senders, shared }, incoming)
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
        self.input_for_slot(id as usize, 0, round, bytes)
    }

    pub(crate) fn returning_to_lobby(&self) -> bool {
        self.shared.return_to_lobby.load(Ordering::Acquire)
    }

    pub(crate) fn bind_slot(&self, slot: usize, generation: u64) {
        self.shared.owners[slot].store(generation, Ordering::Release);
    }

    pub(crate) fn unbind_slot(&self, slot: usize, generation: u64) {
        let _ = self.shared.owners[slot].compare_exchange(
            generation,
            0,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub(crate) fn input_for_slot(
        &self,
        slot: usize,
        generation: u64,
        round: u64,
        bytes: &[u8],
    ) -> Result<(), DispatchError> {
        InputFrame::decode(bytes).map_err(|e| DispatchError::Invalid(SubmitError::Invalid(e)))?;
        self.send(
            slot,
            Command::Input {
                generation,
                round,
                bytes: bytes.try_into().expect("validated input length"),
                received: Instant::now(),
            },
        )
    }
    pub fn attach(&self, id: FighterId) -> Result<(), DispatchError> {
        self.send(id as usize, Command::Attach)
    }
    pub fn detach(&self, id: FighterId) -> Result<(), DispatchError> {
        self.send(id as usize, Command::Detach)
    }

    fn send(&self, slot: usize, command: Command) -> Result<(), DispatchError> {
        if self.shared.closed.load(Ordering::Acquire) {
            return Err(DispatchError::Closed);
        }
        self.senders
            .get(slot)
            .ok_or(DispatchError::Invalid(SubmitError::Detached))?
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
