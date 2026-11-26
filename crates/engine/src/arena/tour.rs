use super::{ArenaKind, MapDefinition, Skyline};
use crate::{CharacterError, physics::PhysicsWorld};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

const PACK: &str = include_str!("../../../../assets/maps/tour.json");

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionKind {
    Slide,
    Roll,
    Sink,
    Sweep,
    Heave,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub object: u16,
    pub kind: MotionKind,
    pub axis: [f32; 3],
    pub amplitude: f32,
    pub period: u32,
    pub start: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TourStage {
    pub name: String,
    pub theme: String,
    pub twist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub water: Option<f32>,
    pub motions: Vec<Motion>,
}

impl TourStage {
    pub(super) fn validate(&self, map: &MapDefinition) -> Result<(), CharacterError> {
        let s = &map.schedule;
        if map.schema_version != 1
            || map.content_version == 0
            || map.id == "skyline"
            || map.id.is_empty()
            || self.name.is_empty()
            || self.theme.is_empty()
            || self.twist.is_empty()
            || !(2..=32).contains(&map.objects.len())
            || !(-40.0..=-5.).contains(&map.fall_y)
            || !(s.warn_tick < s.west_tick
                && s.west_tick < s.east_tick
                && s.east_tick < s.gust_tick
                && s.gust_tick < s.end_tick
                && s.end_tick <= 54000)
            || self
                .water
                .is_some_and(|w| !w.is_finite() || w <= map.fall_y || w > 12.)
        {
            return Err(CharacterError::Invalid("invalid tour stage header"));
        }
        for (i, obj) in map.objects.iter().enumerate() {
            // The parent target pose is resolved in wire order before the child.
            // Dynamic props remain solver-owned and cannot anchor or inherit authored motion.
            if let Some(parent) = obj.parent {
                if parent == 0
                    || usize::from(parent) > i
                    || obj.kind == ArenaKind::Crate
                    || map.objects[usize::from(parent) - 1].kind == ArenaKind::Crate
                {
                    return Err(CharacterError::Invalid("invalid tour attachment"));
                }
            }
            // Object IDs address the same collision body, grip target and snapshot slot.
            if obj.id as usize != i + 1
                || !obj.center.iter().all(|v| v.is_finite() && v.abs() <= 64.)
                || !obj.half.iter().all(|v| (0.04..=32.).contains(v))
                || !(0.0..=500.).contains(&obj.mass)
                || (obj.kind == ArenaKind::Crate && obj.mass <= 0.)
            {
                return Err(CharacterError::Invalid("invalid tour solid"));
            }
        }
        for (i, spawn) in map.spawns.iter().enumerate() {
            let [x, y, z] = spawn.position;
            let supported = map.objects.iter().any(|o| {
                matches!(o.kind, ArenaKind::Deck | ArenaKind::Panel | ArenaKind::Vent)
                    && (o.center[1] + o.half[1] - y).abs() < 0.06
                    && (o.center[0] - x).abs() + 0.15 < o.half[0]
                    && (o.center[2] - z).abs() + 0.15 < o.half[2]
            });
            if !spawn.yaw.is_finite()
                || !supported
                || map.spawns[..i].iter().any(|other| {
                    let d = Vector::from(other.position) - Vector::from(spawn.position);
                    d.norm_squared() < 0.8 * 0.8
                })
            {
                return Err(CharacterError::Invalid("invalid tour spawn"));
            }
        }
        for m in &self.motions {
            if m.object == 0
                || m.object as usize > map.objects.len()
                || m.period == 0
                || m.start >= s.end_tick
                || !m.amplitude.is_finite()
                || m.amplitude.abs() > 100.
                || !(0.97..=1.03).contains(&Vector::from(m.axis).norm_squared())
            {
                return Err(CharacterError::Invalid("invalid tour motion"));
            }
        }
        Ok(())
    }
}

impl MapDefinition {
    pub fn tour_catalog() -> Result<&'static [Self], CharacterError> {
        static CATALOG: OnceLock<Result<Vec<MapDefinition>, &'static str>> = OnceLock::new();
        CATALOG
            .get_or_init(|| {
                let mut catalog: Vec<MapDefinition> =
                    serde_json::from_str(PACK).map_err(|_| "invalid bundled tour JSON")?;
                catalog.insert(0, Self::skyline().map_err(|_| "invalid Skyline")?);
                for (i, stage) in catalog.iter().enumerate() {
                    stage.validate().map_err(|err| match err {
                        CharacterError::Invalid(msg) => msg,
                        _ => "invalid tour stage",
                    })?;
                    if catalog[..i].iter().any(|p| p.id == stage.id) {
                        return Err("duplicate tour stage");
                    }
                }
                Ok(catalog)
            })
            .as_deref()
            .map_err(|msg| CharacterError::Invalid(msg))
    }
    pub fn tour_hash() -> String {
        let mut hash = Sha256::new();
        hash.update(Self::skyline_hash());
        hash.update(PACK.as_bytes());
        format!("{:x}", hash.finalize())
    }
    pub fn tour_round(seed: u32, round: u32) -> Result<Self, CharacterError> {
        if round == 0 {
            return Err(CharacterError::Invalid("series rounds start at one"));
        }
        let catalog = Self::tour_catalog()?;
        Ok(catalog[stage_index(seed, round, catalog.len())].clone())
    }
}

fn stage_index(seed: u32, round: u32, count: usize) -> usize {
    let offset = (round - 1) as usize;
    let cycle = offset / count;
    let mut state = u64::from(seed) ^ (cycle as u64).wrapping_mul(0x632be59bd9b4e019);
    let mut next = || {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = (state ^ (state >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    };
    let mut cards: Vec<_> = (0..count).collect();
    for i in (1..count).rev() {
        let j = (next() % (i + 1) as u64) as usize;
        cards.swap(i, j);
    }
    // Use the same parity for both boundary cards of a bag, alternating parity
    // between bags. Every stage is still available as the first card of a series.
    if count >= 4 {
        let parity = ((seed & 1) as usize + cycle % 2) % 2;
        let first = cards.iter().position(|id| id % 2 == parity).unwrap();
        cards.swap(0, first);
        let last = cards[1..].iter().rposition(|id| id % 2 == parity).unwrap() + 1;
        cards.swap(count - 1, last);
    }
    cards[offset % count]
}

impl Skyline {
    pub(super) fn move_stage(&self, world: &mut PhysicsWorld, tick: u64) {
        let stage = self.definition.tour.as_ref().unwrap();
        // Composing all motions from the authored pose keeps a pitching deck and its
        // lateral travel deterministic after round changes and prevents cumulative drift.
        for (obj, &handle) in self.definition.objects.iter().zip(&self.bodies) {
            if !world.bodies[handle].is_kinematic() {
                continue;
            }
            let mut pose = Isometry::translation(obj.center[0], obj.center[1], obj.center[2]);
            for m in stage
                .motions
                .iter()
                .filter(|m| m.object == obj.id && tick >= u64::from(m.start))
            {
                let elapsed = (tick - u64::from(m.start)) as f32 / m.period as f32;
                let phase = std::f32::consts::TAU * elapsed;
                let axis = Vector::from(m.axis);
                match m.kind {
                    MotionKind::Roll | MotionKind::Sweep => {
                        let angle = if matches!(m.kind, MotionKind::Roll) {
                            phase.sin()
                        } else {
                            elapsed
                        };
                        pose.rotation *= Rotation::from_scaled_axis(axis * (angle * m.amplitude));
                    }
                    MotionKind::Sink => {
                        let t = elapsed.min(1.0);
                        pose.translation.vector += axis * (m.amplitude * t * t * (3. - 2. * t));
                    }
                    MotionKind::Slide | MotionKind::Heave => {
                        pose.translation.vector += axis * (phase.sin() * m.amplitude)
                    }
                }
            }
            if let Some(parent) = obj.parent {
                let idx = usize::from(parent) - 1;
                let support = &world.bodies[self.bodies[idx]];
                // Authored centers share world space at rest. Transform only the
                // relative offset and local animation by the parent's upcoming pose.
                pose.translation.vector -= Vector::from(self.definition.objects[idx].center);
                pose = support.next_position() * pose;
            }
            world.bodies[handle].set_next_kinematic_position(pose);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhysicsConfig;
    use std::collections::HashSet;

    #[test]
    fn shuffled_tour_bags_include_every_stage_once_and_do_not_repeat_at_the_seam() {
        let catalog = MapDefinition::tour_catalog().unwrap();
        assert_eq!(catalog.len(), 22);
        let mut openings = HashSet::new();
        for seed in 0..100 {
            openings.insert(stage_index(seed, 1, catalog.len()));
            let sequence: Vec<_> = (1..=66)
                .map(|r| stage_index(seed, r, catalog.len()))
                .collect();
            for bag in sequence.chunks(22) {
                assert_eq!(bag.iter().collect::<HashSet<_>>().len(), 22);
            }
            assert!(sequence.windows(2).all(|pair| pair[0] != pair[1]));
        }
        assert_eq!(openings.len(), 22);
    }

    #[test]
    fn rigid_trawler_and_airplane_fixtures_follow_their_parent_through_the_solve() {
        for (id, tag) in [
            ("night-trawler", "wheelhouse"),
            ("cloud-courier", "cockpit"),
            ("cloud-courier", "tail-fin"),
            ("cloud-courier", "tailplane"),
        ] {
            let map = MapDefinition::tour_catalog()
                .unwrap()
                .iter()
                .find(|m| m.id == id)
                .unwrap()
                .clone();
            let child = map
                .objects
                .iter()
                .find(|o| o.art.as_deref() == Some(tag))
                .unwrap();
            let pi = usize::from(child.parent.expect("articulated fixture requires a parent")) - 1;
            let ci = usize::from(child.id) - 1;
            let offset = Vector::from(child.center) - Vector::from(map.objects[pi].center);
            let mut world = PhysicsWorld::new(&PhysicsConfig::default());
            let mut arena = Skyline::from_map(&mut world, map).unwrap();
            for tick in [1, 1040, 1160, 2120, 2460] {
                arena.step(&mut world, tick);
                world.step();
                let parent = &world.bodies[arena.bodies[pi]];
                let child = &world.bodies[arena.bodies[ci]];
                let relative = parent.position().inverse() * child.position();
                assert!(
                    (relative.translation.vector - offset).norm() < 1e-4,
                    "{tag} detached"
                );
                assert!(
                    relative.rotation.angle() < 1e-5,
                    "{tag} rotated away from deck"
                );
                assert_eq!(
                    world.colliders[child.colliders()[0]].position(),
                    child.position()
                );
            }
        }
    }

    #[test]
    fn child_animation_uses_the_parents_rotating_axes_and_rejects_cycles() {
        let mut map = MapDefinition::tour_catalog()
            .unwrap()
            .iter()
            .find(|m| m.id == "night-trawler")
            .unwrap()
            .clone();
        let ci = map
            .objects
            .iter()
            .position(|o| o.art.as_deref() == Some("wheelhouse"))
            .unwrap();
        let child = map.objects[ci].id;
        let offset = Vector::from(map.objects[ci].center) - Vector::from(map.objects[0].center);
        map.objects[ci].parent = Some(child);
        assert!(map.validate().is_err());
        map.objects[ci].parent = Some(5); // The loose fish box cannot carry a scripted child.
        assert!(map.validate().is_err());
        map.objects[ci].parent = Some(1);
        let motions = &mut map.tour.as_mut().unwrap().motions;
        motions.clear();
        for (object, kind, axis, amplitude) in [
            (1, MotionKind::Roll, [0., 0., 1.], 0.5),
            (1, MotionKind::Slide, [0., 0., 1.], 1.2),
            (child, MotionKind::Heave, [0., 1., 0.], 0.4),
        ] {
            motions.push(Motion {
                object,
                kind,
                axis,
                amplitude,
                period: 120,
                start: 0,
            });
        }
        map.validate().unwrap();
        let mut world = PhysicsWorld::new(&PhysicsConfig::default());
        let mut arena = Skyline::from_map(&mut world, map).unwrap();
        arena.step(&mut world, 30);
        world.step();
        let parent = &world.bodies[arena.bodies[0]];
        let child = &world.bodies[arena.bodies[ci]];
        let relative = parent.position().inverse() * child.position();
        assert!((relative.translation.vector - (offset + Vector::y() * 0.4)).norm() < 1e-4);
        assert!(relative.rotation.angle() < 1e-5);
    }

    #[test]
    fn each_authored_motion_reaches_the_rapier_collider_and_is_exported_as_a_pose() {
        for map in &MapDefinition::tour_catalog().unwrap()[1..] {
            let mut world = PhysicsWorld::new(&PhysicsConfig::default());
            let mut arena = Skyline::from_map(&mut world, map.clone()).unwrap();
            let motions = &map.tour.as_ref().unwrap().motions;
            assert!(!motions.is_empty(), "{}", map.id);
            for motion in motions {
                let tick = motion.start + motion.period / 4;
                arena.step(&mut world, u64::from(tick));
                world.step();
                let idx = usize::from(motion.object) - 1;
                let state = &arena.states(&world)[idx];
                let shift =
                    (Vector::from(state.position) - Vector::from(map.objects[idx].center)).norm();
                assert!(
                    shift > 0.01 || state.rotation[3].abs() < 0.99999,
                    "{} motion {:?} at {tick}",
                    map.id,
                    motion.kind
                );
                let body = &world.bodies[arena.bodies[idx]];
                let collider = &world.colliders[body.colliders()[0]];
                assert_eq!(collider.position(), body.position());
            }
        }
    }
}
