//! Headless Skyline room advanced by its owning simulation worker.

mod controls;
mod inbox;
mod room;

pub use inbox::SubmitError;
pub use room::Room;

mod snapshot;
pub use snapshot::RoomFrame;

mod worker;
pub use worker::{DispatchError, RoomClient, WorkerPool};

mod admission;
mod api;
pub use api::{api, api_for_origin};
