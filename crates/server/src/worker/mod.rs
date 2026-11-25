mod cadence;
mod channel;
mod pool;

pub use channel::{DispatchError, RoomClient};
pub use pool::WorkerPool;

use crate::Room;
use brawler_engine::FighterId;
use cadence::Cadence;
use channel::{Command, Shared};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

struct Job {
    shared: Arc<Shared>,
    incoming: [Receiver<Command>; 2],
    ready: SyncSender<Result<(), DispatchError>>,
}

fn run(job: &Job, stop: &AtomicBool, overloaded: &AtomicBool) -> Result<(), DispatchError> {
    let mut room = Room::skyline().map_err(|_| DispatchError::Initialization)?;
    let publish = |r: &Room| {
        let frame = r.capture().map_err(|_| DispatchError::Initialization)?;
        job.shared.latest.lock().unwrap().replace(Arc::new(frame));
        Ok(())
    };
    publish(&room)?;
    job.ready.send(Ok(())).map_err(|_| DispatchError::Closed)?;
    let origin = Instant::now();
    let mut clock = Cadence::default();
    let mut accepted = [None; 2];
    while !stop.load(Ordering::Acquire) && !job.shared.closed.load(Ordering::Acquire) {
        let elapsed = origin.elapsed();
        let debt = clock.debt(elapsed);
        // One second of unserved simulation time suspends new admissions across the pool.
        overloaded.store(debt >= 60, Ordering::Release);
        if debt > 4 {
            job.shared.delayed.fetch_add(1, Ordering::Relaxed);
        }
        for _ in 0..clock.take(elapsed) {
            for id in [FighterId::First, FighterId::Second] {
                let i = id as usize;
                drain(
                    &mut room,
                    id,
                    &job.incoming[i],
                    &job.shared,
                    &mut accepted[i],
                );
            }
            room.step();
            if room.snapshot_due() {
                publish(&room)?;
            }
        }
        let wait = clock.wait(origin.elapsed());
        if wait.is_zero() {
            thread::yield_now();
        } else {
            thread::park_timeout(wait);
        }
    }
    Ok(())
}

fn drain(
    room: &mut Room,
    id: FighterId,
    incoming: &Receiver<Command>,
    shared: &Shared,
    last: &mut Option<Instant>,
) {
    let expiry = Duration::from_millis(250);
    if last.is_some_and(|at| at.elapsed() >= expiry) {
        room.expire(id);
        *last = None;
    }
    // Bound command work even when a sender refills a slot during this drain.
    for command in incoming.try_iter().take(8) {
        match command {
            Command::Input {
                round,
                bytes,
                received,
            } => {
                if received.elapsed() >= expiry || room.submit(id, round, &bytes).is_err() {
                    shared.rejected.fetch_add(1, Ordering::Relaxed);
                } else {
                    *last = Some(received);
                }
            }
            Command::Attach => room.attach(id),
            Command::Detach => {
                room.detach(id);
                *last = None;
            }
        }
    }
}

#[cfg(test)]
mod tests;
