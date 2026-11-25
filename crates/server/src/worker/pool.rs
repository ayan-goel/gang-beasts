use super::{DispatchError, Job, RoomClient, run};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{SyncSender, sync_channel},
    },
    thread::{self, JoinHandle},
};

struct Worker {
    sender: Option<SyncSender<Job>>,
    busy: Arc<AtomicBool>,
    overloaded: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

pub struct WorkerPool {
    workers: Vec<Worker>,
    stop: Arc<AtomicBool>,
}

impl WorkerPool {
    pub fn new(size: usize) -> io::Result<Self> {
        if !(1..=64).contains(&size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker count must be 1 through 64",
            ));
        }
        let mut pool = Self {
            workers: Vec::new(),
            stop: Arc::default(),
        };
        for index in 0..size {
            let (sender, receiver) = sync_channel::<Job>(1);
            let stop = Arc::clone(&pool.stop);
            let reserved = Arc::<AtomicBool>::default();
            let busy = Arc::clone(&reserved);
            let behind = Arc::<AtomicBool>::default();
            let overloaded = Arc::clone(&behind);
            let handle = thread::Builder::new()
                .name(format!("skyline-worker-{index}"))
                .spawn(move || {
                    while let Ok(job) = receiver.recv() {
                        if let Err(error) = run(&job, &stop, &overloaded) {
                            let _ = job.ready.try_send(Err(error));
                        }
                        job.shared.closed.store(true, Ordering::Release);
                        overloaded.store(false, Ordering::Release);
                        busy.store(false, Ordering::Release);
                    }
                })?;
            pool.workers.push(Worker {
                sender: Some(sender),
                busy: reserved,
                overloaded: behind,
                handle: Some(handle),
            });
        }
        Ok(pool)
    }

    fn overloaded(&self) -> bool {
        self.workers
            .iter()
            .any(|w| w.overloaded.load(Ordering::Acquire))
    }

    pub fn available(&self) -> usize {
        if self.overloaded() {
            return 0;
        }
        self.workers
            .iter()
            .filter(|w| !w.busy.load(Ordering::Acquire))
            .count()
    }

    pub fn open(&self) -> Result<RoomClient, DispatchError> {
        self.open_with_players(2)
    }

    pub fn open_with_players(&self, count: usize) -> Result<RoomClient, DispatchError> {
        if !(2..=brawler_engine::FighterId::ALL.len()).contains(&count) {
            return Err(DispatchError::Initialization);
        }
        if self.overloaded() {
            return Err(DispatchError::Busy);
        }
        let worker = self
            .workers
            .iter()
            .find(|w| {
                w.busy
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
            })
            .ok_or(DispatchError::Capacity)?;
        let (client, incoming) = RoomClient::channel(count);
        let (ready, started) = sync_channel(1);
        let job = Job {
            shared: Arc::clone(&client.shared),
            incoming,
            ready,
        };
        worker
            .sender
            .as_ref()
            .unwrap()
            .send(job)
            .map_err(|_| DispatchError::Closed)?;
        started.recv().map_err(|_| DispatchError::Closed)??;
        Ok(client)
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for worker in &mut self.workers {
            worker.sender.take();
            if let Some(handle) = worker.handle.take() {
                handle.thread().unpark();
                let _ = handle.join();
            }
        }
    }
}
