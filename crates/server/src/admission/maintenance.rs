use super::directory::Directory;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

pub(super) const GRACE: Duration = Duration::from_secs(20);

pub(super) fn start(directory: &Arc<Mutex<Directory>>) {
    // Dead rooms must release their worker even when the last browser never sends another request.
    let directory = Arc::downgrade(directory);
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_millis(100));
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            let Some(directory) = directory.upgrade() else {
                return;
            };
            directory.lock().unwrap().prune(Instant::now());
        }
    });
}
