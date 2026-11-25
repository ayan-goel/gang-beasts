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
    match_serial: u32,
    shared: Arc<Shared>,
    incoming: Vec<Receiver<Command>>,
    ready: SyncSender<Result<(), DispatchError>>,
}

fn run(job: &Job, stop: &AtomicBool, overloaded: &AtomicBool) -> Result<(), DispatchError> {
    let mut room = Room::skyline_in_match(job.incoming.len(), job.match_serial)
        .map_err(|_| DispatchError::Initialization)?;
    let publish = |r: &Room| {
        let frame = r.capture().map_err(|_| DispatchError::Initialization)?;
        job.shared.latest.lock().unwrap().replace(Arc::new(frame));
        Ok(())
    };
    publish(&room)?;
    job.ready.send(Ok(())).map_err(|_| DispatchError::Closed)?;
    let origin = Instant::now();
    let mut clock = Cadence::default();
    let mut accepted = [None; FighterId::ALL.len()];
    let mut owners = [0u64; FighterId::ALL.len()];
    while !stop.load(Ordering::Acquire) && !job.shared.closed.load(Ordering::Acquire) {
        let elapsed = origin.elapsed();
        let debt = clock.debt(elapsed);
        // One second of unserved simulation time suspends new admissions across the pool.
        overloaded.store(debt >= 60, Ordering::Release);
        if debt > 4 {
            job.shared.delayed.fetch_add(1, Ordering::Relaxed);
        }
        for _ in 0..clock.take(elapsed) {
            for &id in room.game().fighter_ids() {
                let i = id as usize;
                let generation = job.shared.owners[i].load(Ordering::Acquire);
                if owners[i] != generation {
                    room.replace_connection(id, generation != 0);
                    owners[i] = generation;
                    accepted[i] = None;
                }
                drain(
                    &mut room,
                    id,
                    &job.incoming[i],
                    &job.shared,
                    &mut accepted[i],
                    generation,
                );
            }
            room.step_with_forfeits(job.shared.forfeits.load(Ordering::Acquire));
            if room.finished_for_lack_of_players() {
                job.shared.return_to_lobby.store(true, Ordering::Release);
                return Ok(());
            }
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
    generation: u64,
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
                generation: sender,
                round,
                bytes,
                received,
            } => {
                if sender != generation
                    || received.elapsed() >= expiry
                    || room.submit(id, round, &bytes).is_err()
                {
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
