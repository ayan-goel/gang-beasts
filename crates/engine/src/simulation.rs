use rapier3d::prelude::*;

use crate::{
    CommandError, ConfigError, FixtureState, PhysicsConfig, fixture::validate_impulse,
    physics::PhysicsWorld,
};

pub struct Simulation {
    world: PhysicsWorld,
    completed_tick: u64,
    fixture_body: RigidBodyHandle,
    config: PhysicsConfig,
    pending_impulse: Option<[f32; 3]>,
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
            config,
            pending_impulse: None,
        })
    }

    pub fn step(&mut self) {
        if let Some(impulse) = self.pending_impulse.take() {
            self.world.bodies[self.fixture_body].apply_impulse(impulse.into(), true);
        }
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

    pub fn fixture_state(&self) -> FixtureState {
        let body = &self.world.bodies[self.fixture_body];
        FixtureState {
            position: (*body.translation()).into(),
            rotation: body.rotation().coords.into(),
            linear_velocity: (*body.linvel()).into(),
            angular_velocity: (*body.angvel()).into(),
            mass: body.mass(),
        }
    }

    pub fn queue_fixture_impulse(&mut self, impulse: [f32; 3]) -> Result<(), CommandError> {
        validate_impulse(impulse)?;
        if self.pending_impulse.is_some() {
            return Err(CommandError::ImpulsePending);
        }
        self.pending_impulse = Some(impulse);
        Ok(())
    }

    pub fn reset(&mut self) {
        // Rebuilding clears solver caches as well as queued commands, so reset
        // reproduces a fresh fixture rather than merely teleporting the body.
        *self = Self::new(self.config.clone()).expect("stored configuration was validated");
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.world.collider_lines()
    }
}
