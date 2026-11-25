use rapier3d::prelude::*;

use crate::{ConfigError, PhysicsConfig, physics::PhysicsWorld};

pub struct Simulation {
    world: PhysicsWorld,
    completed_tick: u64,
    fixture_body: RigidBodyHandle,
}

impl Simulation {
    pub fn new(config: PhysicsConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        let mut world = PhysicsWorld::new(&config);
        world
            .colliders
            .insert(ColliderBuilder::cuboid(6.0, 0.5, 6.0).translation(vector![0.0, -0.5, 0.0]));
        let fixture_body = world
            .bodies
            .insert(RigidBodyBuilder::dynamic().translation(vector![0.0, 4.0, 0.0]));
        world.colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.5, 0.5, 0.5)
                .mass(1.0)
                .restitution(0.0),
            fixture_body,
            &mut world.bodies,
        );
        Ok(Self {
            world,
            completed_tick: 0,
            fixture_body,
        })
    }

    pub fn step(&mut self) {
        self.world.step();
        // Events and render state identify the completed step; tick zero is the
        // initial fixture, avoiding ambiguous pre-step/post-step replay labels.
        self.completed_tick += 1;
    }

    pub fn tick(&self) -> u64 {
        self.completed_tick
    }

    pub fn fixture_position(&self) -> [f32; 3] {
        (*self.world.bodies[self.fixture_body].translation()).into()
    }
}
