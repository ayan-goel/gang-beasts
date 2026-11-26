use crate::{RoomFrame, SubmitError};
use brawler_engine::FighterId;
use brawler_protocol::{INPUT_BYTES, InputFrame};
use std::{
    sync::{
        Arc,
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
    pub latest: tokio::sync::watch::Sender<Option<Arc<RoomFrame>>>,
    pub closed: AtomicBool,
    pub rejected: AtomicU64,
    pub return_to_lobby: AtomicBool,
    pub(super) series_complete: AtomicBool,
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
            latest: tokio::sync::watch::Sender::new(None),
            closed: AtomicBool::new(false),
            owners: Default::default(),
            return_to_lobby: AtomicBool::new(false),
            series_complete: AtomicBool::new(false),
            rejected: AtomicU64::new(0),
            delayed: AtomicU64::new(0),
        });
        (Self { senders, shared }, incoming)
    }

    pub fn latest(&self) -> Option<Arc<RoomFrame>> {
        self.shared.latest.borrow().clone()
    }

    pub(crate) fn publications(&self) -> tokio::sync::watch::Receiver<Option<Arc<RoomFrame>>> {
        let mut frames = self.shared.latest.subscribe();
        // A late join or reconnect needs the retained frame even if the worker has just published.
        frames.mark_changed();
        frames
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

    pub(crate) fn series_complete(&self) -> bool {
        self.shared.series_complete.load(Ordering::Acquire)
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

#[cfg(test)]
mod publication_tests {
    use super::*;
    use crate::Room;

    #[tokio::test]
    async fn new_connections_get_current_state_and_slow_connections_skip_old_frames() {
        let (client, _) = RoomClient::channel(2);
        let mut room = Room::skyline().unwrap();
        let publish = |room: &Room| {
            client
                .shared
                .latest
                .send_replace(Some(Arc::new(room.capture().unwrap())));
        };
        publish(&room);
        let mut prompt = client.publications();
        let mut slow = client.publications();
        assert!(prompt.has_changed().unwrap());
        assert_eq!(prompt.borrow_and_update().as_ref().unwrap().tick(), 0);
        assert!(!prompt.has_changed().unwrap());
        for expected in [2, 4] {
            room.step();
            room.step();
            publish(&room);
            assert!(prompt.has_changed().unwrap());
            prompt.changed().await.unwrap();
            assert_eq!(
                prompt.borrow_and_update().as_ref().unwrap().tick(),
                expected
            );
        }
        assert_eq!(slow.borrow_and_update().as_ref().unwrap().tick(), 4);
        assert!(!slow.has_changed().unwrap());
        assert!(Arc::ptr_eq(
            slow.borrow().as_ref().unwrap(),
            prompt.borrow().as_ref().unwrap()
        ));
    }
}
