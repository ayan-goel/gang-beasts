mod encode;

pub const ARENA_BASE: u32 = 1 << 16;
pub const MAX_BODIES: usize = 256;
pub const LIMBS: usize = 15;

pub fn limb_id(slot: usize, part: usize) -> u32 {
    (1 + 16 * slot + part) as u32
}

#[derive(Debug, Clone)]
pub struct BodyPose {
    pub id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Default, Debug, Clone)]
pub struct FighterState {
    pub consciousness: u8,
    pub score: u8,
    pub hands: [u8; 2],
    pub health: f32,
    pub stamina: f32,
    pub grips: [u32; 2],
}

pub struct Snapshot {
    pub tick: u32,
    pub applied: Option<u32>,
    pub round: u64,
    pub number: u32,
    pub arena_tick: u32,
    pub phase: u8,
    pub winner: Option<u8>,
    pub remaining: u16,
    pub bodies: Vec<BodyPose>,
    pub fighters: Vec<FighterState>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Match,
    Body,
    Fighter,
    Grip,
}

impl Snapshot {
    fn validate(&self) -> Result<(), SnapshotError> {
        let count = self.fighters.len();
        if !(2..=8).contains(&count)
            || self.phase > 3
            || self.number == 0
            || self.round == 0
            || self
                .winner
                .is_some_and(|s| usize::from(s) >= count || self.phase < 2)
        {
            return Err(SnapshotError::Match);
        }
        if !(count * LIMBS..=MAX_BODIES).contains(&self.bodies.len()) {
            return Err(SnapshotError::Body);
        }
        let mut last = 0;
        for (i, b) in self.bodies.iter().enumerate() {
            let valid_id = if i < count * LIMBS {
                b.id == limb_id(i / LIMBS, i % LIMBS)
            } else {
                b.id > ARENA_BASE && b.id <= ARENA_BASE + u32::from(u16::MAX)
            };
            let norm = b
                .rotation
                .iter()
                .map(|&v| f64::from(v).powi(2))
                .sum::<f64>();
            if !valid_id
                || b.id <= last
                || !(0.998..=1.002).contains(&norm)
                || !b.position.iter().all(|v| v.is_finite())
            {
                return Err(SnapshotError::Body);
            }
            last = b.id;
        }
        for (slot, fighter) in self.fighters.iter().enumerate() {
            if fighter.consciousness > 3
                || ![fighter.health, fighter.stamina]
                    .into_iter()
                    .all(|v| (0.0..=100.).contains(&v))
                || fighter.hands.iter().any(|&phase| phase > 5)
            {
                return Err(SnapshotError::Fighter);
            }
            for (&target, phase) in fighter.grips.iter().zip(fighter.hands) {
                if (target != 0) != (phase == 5)
                    || (target != 0
                        && ((limb_id(slot, 0)..=limb_id(slot, LIMBS - 1)).contains(&target)
                            || self.bodies.binary_search_by_key(&target, |b| b.id).is_err()))
                {
                    return Err(SnapshotError::Grip);
                }
            }
        }
        Ok(())
    }
}
