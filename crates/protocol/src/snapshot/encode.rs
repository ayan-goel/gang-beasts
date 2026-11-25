use super::{Snapshot, SnapshotError};
use crate::VERSION;

pub struct SharedSnapshot(Vec<u8>);

impl SharedSnapshot {
    pub fn for_recipient(&self, applied: Option<u32>) -> Vec<u8> {
        let mut bytes = self.0.clone();
        // Only the full/ack flags and recipient sequence vary between recipients.
        bytes[2..4].copy_from_slice(&flags(applied).to_le_bytes());
        bytes[8..12].copy_from_slice(&applied.unwrap_or(0).to_le_bytes());
        bytes
    }
}

fn flags(applied: Option<u32>) -> u16 {
    if applied.is_some() { 3 } else { 1 }
}

impl Snapshot {
    pub fn encode_shared(&self) -> Result<SharedSnapshot, SnapshotError> {
        Ok(SharedSnapshot(self.encode()?))
    }

    pub fn encode(&self) -> Result<Vec<u8>, SnapshotError> {
        self.validate()?;
        let gameplay = 24 + 24 * self.fighters.len();
        let mut bytes = Vec::with_capacity(20 + 32 * self.bodies.len() + gameplay);
        bytes.extend([2, VERSION]);
        bytes.extend(flags(self.applied).to_le_bytes());
        for n in [self.tick, self.applied.unwrap_or(0), 0] {
            bytes.extend(n.to_le_bytes());
        }
        bytes.extend((self.bodies.len() as u16).to_le_bytes());
        bytes.extend((gameplay as u16).to_le_bytes());
        for b in &self.bodies {
            bytes.extend(b.id.to_le_bytes());
            for v in b.position.into_iter().chain(b.rotation) {
                bytes.extend(v.to_le_bytes());
            }
        }
        bytes.extend([
            1,
            self.phase,
            self.winner.unwrap_or(255),
            self.fighters.len() as u8,
        ]);
        bytes.extend(self.remaining.to_le_bytes());
        bytes.extend([0; 2]);
        bytes.extend(self.round.to_le_bytes());
        bytes.extend(self.number.to_le_bytes());
        bytes.extend(self.arena_tick.to_le_bytes());
        for (slot, f) in self.fighters.iter().enumerate() {
            bytes.extend([
                slot as u8,
                f.consciousness,
                f.score,
                f.hands[0],
                f.hands[1],
                0,
                0,
                0,
            ]);
            for v in [f.health, f.stamina] {
                bytes.extend(v.to_le_bytes());
            }
            for target in f.grips {
                bytes.extend(target.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}
