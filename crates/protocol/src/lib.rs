//! Native endpoints share explicit, versioned browser game records.

mod input;
pub use input::{Button, INPUT_BYTES, INPUT_KIND, InputError, InputFrame};
pub const VERSION: u8 = 1;

pub fn is_newer(candidate: u32, previous: u32) -> bool {
    // Exactly half a u32 cycle is ambiguous. Neither endpoint can establish ordering there.
    let distance = candidate.wrapping_sub(previous);
    distance != 0 && distance < (1 << 31)
}

pub mod lobby;
pub mod snapshot;
