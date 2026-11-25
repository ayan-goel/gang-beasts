//! Headless Skyline room advanced by its owning simulation worker.

mod controls;
mod inbox;
mod room;

pub use inbox::SubmitError;
pub use room::Room;

mod snapshot;
pub use snapshot::RoomFrame;
