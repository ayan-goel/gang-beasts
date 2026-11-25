use crate::CharacterError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SOURCE: &str = include_str!("../../../../assets/maps/skyline/map.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArenaKind {
    Deck,
    Vent,
    Crate,
    Panel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArenaObject {
    pub id: u16,
    pub kind: ArenaKind,
    pub center: [f32; 3],
    pub half: [f32; 3],
    pub mass: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub position: [f32; 3],
    pub yaw: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub warn_tick: u32,
    pub west_tick: u32,
    pub east_tick: u32,
    pub gust_tick: u32,
    pub end_tick: u32,
    pub gust_acceleration: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapDefinition {
    pub schema_version: u8,
    pub content_version: u32,
    pub id: String,
    pub fall_y: f32,
    pub schedule: Schedule,
    pub spawns: [Spawn; 8],
    pub objects: Vec<ArenaObject>,
}

impl MapDefinition {
    pub fn skyline() -> Result<Self, CharacterError> {
        Self::from_json(SOURCE)
    }
    pub fn skyline_hash() -> String {
        format!("{:x}", Sha256::digest(SOURCE.as_bytes()))
    }
    pub fn from_json(data: &str) -> Result<Self, CharacterError> {
        // Bound parse work before allocating authored objects or creating physics handles.
        if data.len() > 32_768 {
            return Err(CharacterError::Invalid("map exceeds 32 KiB"));
        }
        let m: Self = serde_json::from_str(data)?;
        m.validate()?;
        Ok(m)
    }
    fn validate(&self) -> Result<(), CharacterError> {
        let s = &self.schedule;
        if self.schema_version != 1
            || self.content_version == 0
            || self.id != "skyline"
            || !(-24.0..=-6.).contains(&self.fall_y)
            || self.objects.len() != 9
            || !(0 < s.warn_tick
                && s.warn_tick < s.west_tick
                && s.west_tick < s.east_tick
                && s.east_tick < s.gust_tick
                && s.gust_tick < s.end_tick
                && s.end_tick - s.gust_tick <= 1800
                && s.end_tick <= 54000)
            || !(0.1..=15.).contains(&s.gust_acceleration)
        {
            return Err(CharacterError::Invalid(
                "invalid Skyline map header or schedule",
            ));
        }
        // The authored ID order is also the stable wire/render order. Groups 8 and 9
        // carry the scheduled edge events, so a reordered manifest cannot retarget a hazard.
        let roles = [
            ArenaKind::Deck,
            ArenaKind::Vent,
            ArenaKind::Vent,
            ArenaKind::Crate,
            ArenaKind::Crate,
            ArenaKind::Crate,
            ArenaKind::Crate,
            ArenaKind::Panel,
            ArenaKind::Panel,
        ];
        for (i, (o, role)) in self.objects.iter().zip(roles).enumerate() {
            let dynamic = matches!(role, ArenaKind::Crate | ArenaKind::Panel);
            if o.id != i as u16 + 1
                || o.kind != role
                || !o.center.iter().all(|v| v.is_finite() && v.abs() <= 32.)
                || !o.half.iter().all(|v| (0.05..=15.).contains(v))
                || if dynamic {
                    !(1.0..=250.).contains(&o.mass)
                } else {
                    o.mass != 0.
                }
            {
                return Err(CharacterError::Invalid("invalid Skyline object"));
            }
            if self.objects[..i].iter().any(|other| {
                (0..3).all(|a| {
                    (other.center[a] - o.center[a]).abs() + 0.001 < other.half[a] + o.half[a]
                })
            }) {
                return Err(CharacterError::Invalid("overlapping Skyline solids"));
            }
        }
        let deck = &self.objects[0];
        if (deck.center[1] + deck.half[1]).abs() > 0.001 {
            return Err(CharacterError::Invalid("roof top must be at zero"));
        }
        for obj in &self.objects[1..] {
            let [x, y, z] = obj.center;
            let [hx, hy, hz] = obj.half;
            let dx = x - deck.center[0];
            let dz = z - deck.center[2];
            let placed = if obj.kind == ArenaKind::Panel {
                let sign = if obj.id == 8 { -1. } else { 1. };
                (dx * sign - deck.half[0] - hx).abs() < 0.001
                    && (y + hy).abs() < 0.001
                    && dz.abs() + hz <= deck.half[2]
            } else {
                (y - hy).abs() < 0.001
                    && dx.abs() + hx <= deck.half[0]
                    && dz.abs() + hz <= deck.half[2]
            };
            if !placed {
                return Err(CharacterError::Invalid(
                    "Skyline object lacks authored roof support",
                ));
            }
        }
        for (i, spawn) in self.spawns.iter().enumerate() {
            let [x, y, z] = spawn.position;
            let safe = |o: &ArenaObject| {
                (o.center[0] - x).abs() > o.half[0] + 0.5
                    || (o.center[2] - z).abs() > o.half[2] + 0.5
            };
            if !(x.is_finite() && z.is_finite() && spawn.yaw.is_finite())
                || y != 0.
                || (x - deck.center[0]).abs() + 1. >= deck.half[0]
                || (z - deck.center[2]).abs() + 1. >= deck.half[2]
                || self.spawns[..i]
                    .iter()
                    .any(|p| (p.position[0] - x).hypot(p.position[2] - z) < 1.1)
                || !self.objects[1..].iter().all(safe)
            {
                return Err(CharacterError::Invalid("unsafe Skyline spawn"));
            }
        }
        Ok(())
    }
}
