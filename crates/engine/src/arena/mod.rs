mod definition;
mod grip;
mod landing;
mod lip;
use crate::{CharacterError, TICK_SECONDS, physics::PhysicsWorld};
pub use definition::{ArenaKind, MapDefinition};
pub(crate) use landing::Landing;
use rapier3d::prelude::*;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkylineEvent {
    EdgeWarning,
    PanelDetached(u16),
    GustStarted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArenaBodyState {
    pub id: u16,
    pub visible: bool,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

pub(crate) struct Skyline {
    pub definition: MapDefinition,
    pub events: Vec<SkylineEvent>,
    bodies: Vec<RigidBodyHandle>,
    pub fall_sensor: ColliderHandle,
}

fn insert_object(world: &mut PhysicsWorld, obj: &definition::ArenaObject) -> RigidBodyHandle {
    let builder = if obj.kind == ArenaKind::Crate {
        RigidBodyBuilder::dynamic()
    } else {
        RigidBodyBuilder::fixed()
    };
    let body = world.bodies.insert(builder.translation(obj.center.into()));
    let [x, y, z] = obj.half;
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(x, y, z)
            .mass(obj.mass)
            .friction(0.7),
        body,
        &mut world.bodies,
    );
    if obj.kind == ArenaKind::Crate {
        world.track_fast_body(body, x.min(y).min(z));
    }
    body
}

impl Skyline {
    pub fn insert(world: &mut PhysicsWorld) -> Result<Self, CharacterError> {
        let definition = MapDefinition::skyline()?;
        let bodies: Vec<_> = definition
            .objects
            .iter()
            .map(|obj| insert_object(world, obj))
            .collect();
        let core = &definition.foundation;
        let [x, y, z] = core.half;
        let local = Vector::from(core.center) - Vector::from(definition.objects[0].center);
        world.colliders.insert_with_parent(
            ColliderBuilder::cuboid(x, y, z).translation(local),
            bodies[0],
            &mut world.bodies,
        );
        let fall_sensor = world.colliders.insert(
            ColliderBuilder::cuboid(40., 2., 40.)
                .translation(vector![0., definition.fall_y - 2., 0.])
                .sensor(true),
        );
        Ok(Self {
            definition,
            bodies,
            events: Vec::with_capacity(1),
            fall_sensor,
        })
    }

    #[cfg(test)]
    pub(crate) fn add_test_crate(
        &mut self,
        world: &mut PhysicsWorld,
        center: [f32; 3],
    ) -> RigidBodyHandle {
        let mut obj = self
            .definition
            .objects
            .iter()
            .find(|obj| obj.kind == ArenaKind::Crate)
            .unwrap()
            .clone();
        obj.id = u16::try_from(self.bodies.len() + 1).unwrap();
        obj.center = center;
        let body = insert_object(world, &obj);
        self.bodies.push(body);
        self.definition.objects.push(obj);
        body
    }

    fn panel_deadlines(&self) -> [(u16, u64); 2] {
        let s = &self.definition.schedule;
        [(8, s.west_tick.into()), (9, s.east_tick.into())]
    }
    pub fn step(&mut self, world: &mut PhysicsWorld, tick: u64) {
        self.events.clear();
        let s = &self.definition.schedule;
        if tick == u64::from(s.warn_tick) {
            self.events.push(SkylineEvent::EdgeWarning);
        }
        for (id, deadline) in self.panel_deadlines() {
            if tick == deadline {
                // Keep the body and its local anchors alive when the roof connection fails.
                world.bodies[self.bodies[usize::from(id - 1)]]
                    .set_body_type(RigidBodyType::Dynamic, true);
                self.events.push(SkylineEvent::PanelDetached(id));
            }
        }
        if tick == u64::from(s.gust_tick) {
            self.events.push(SkylineEvent::GustStarted);
        }
        let gust_ticks = tick.saturating_sub(s.gust_tick.into());
        if gust_ticks > 0 {
            let dv = (gust_ticks.min(180) as f32 / 180.) * s.gust_acceleration * TICK_SECONDS;
            for (_, body) in world
                .bodies
                .iter_mut()
                .filter(|(_, b)| b.is_dynamic() && b.is_enabled())
            {
                body.apply_impulse(Vector::x() * (body.mass() * dv), true);
            }
        }
        for &handle in &self.bodies {
            let b = &mut world.bodies[handle];
            if b.is_enabled() && b.translation().y < self.definition.fall_y - 16. {
                b.set_enabled(false);
            }
        }
    }
    pub fn states(&self, world: &PhysicsWorld) -> Vec<ArenaBodyState> {
        self.bodies
            .iter()
            .zip(&self.definition.objects)
            .map(|(&handle, obj)| {
                let b = &world.bodies[handle];
                ArenaBodyState {
                    id: obj.id,
                    visible: b.is_enabled(),
                    position: (*b.translation()).into(),
                    rotation: b.rotation().coords.into(),
                }
            })
            .collect()
    }
    pub fn fall_state(&self, world: &PhysicsWorld, root: RigidBodyHandle) -> FallState {
        // Sensor overlap is useful for broad-phase reporting. The authoritative
        // center test still handles a high-speed fall beyond the sensor footprint.
        FallState {
            sensor_overlap: world.bodies[root]
                .colliders()
                .iter()
                .any(|&c| world.narrow_phase.intersection_pair(self.fall_sensor, c) == Some(true)),
            below_plane: world.bodies[root].center_of_mass().y < self.definition.fall_y,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallState {
    pub sensor_overlap: bool,
    pub below_plane: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sensor_overlap_alone_is_insufficient_and_fast_wide_falls_still_count() {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let arena = Skyline::insert(&mut world).unwrap();
        let body = world
            .bodies
            .insert(RigidBodyBuilder::dynamic().translation(vector![13., -11.9, 0.]));
        world
            .colliders
            .insert_with_parent(ColliderBuilder::ball(0.5), body, &mut world.bodies);
        world.step();
        let near = arena.fall_state(&world, body);
        assert!(near.sensor_overlap && !near.below_plane);
        world.bodies[body].set_translation(vector![50., -16., 0.], true);
        world.step();
        let far = arena.fall_state(&world, body);
        assert!(far.below_plane && !far.sensor_overlap);
    }
    #[test]
    fn final_gust_ramps_to_a_bounded_mass_independent_acceleration() {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let mut arena = Skyline::insert(&mut world).unwrap();
        let ids = [2., 40.].map(|mass| {
            let body = world
                .bodies
                .insert(RigidBodyBuilder::dynamic().translation(vector![mass, 20., 0.]));
            world.colliders.insert_with_parent(
                ColliderBuilder::ball(0.1).mass(mass),
                body,
                &mut world.bodies,
            );
            body
        });
        world.step();
        arena.step(&mut world, 10800);
        assert_eq!(arena.events, [SkylineEvent::GustStarted]);
        for (tick, expected) in [(10890, 4.5), (10980, 9.), (11040, 9.)] {
            let previous = ids.map(|id| world.bodies[id].linvel().x);
            arena.step(&mut world, tick);
            for (id, old) in ids.into_iter().zip(previous) {
                assert!(
                    ((world.bodies[id].linvel().x - old) / TICK_SECONDS - expected).abs() < 1e-4
                );
            }
        }
    }

    #[test]
    fn physical_crates_respond_to_impulse_while_vents_remain_anchored() {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let arena = Skyline::insert(&mut world).unwrap();
        let (vent, box_id) = (arena.bodies[1], arena.bodies[3]);
        let before = [vent, box_id].map(|h| *world.bodies[h].translation());
        for id in [vent, box_id] {
            world.bodies[id].apply_impulse(vector![18., 0., 0.], true);
        }
        for _ in 0..20 {
            world.step();
        }
        assert_eq!(before[0], *world.bodies[vent].translation());
        assert!(world.bodies[box_id].translation().x > before[1].x + 0.15);
    }
}

#[cfg(test)]
mod fast_crate_tests;
