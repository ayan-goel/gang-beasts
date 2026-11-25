use super::{BodyPart, CharacterDefinition, CharacterError, rig::insert_rig};
use crate::{
    CommandError, FixtureState, PhysicsConfig, fixture::validate_impulse, physics::PhysicsWorld,
};
use rapier3d::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RagdollBodyState {
    pub part: BodyPart,
    pub state: FixtureState,
}

pub struct RagdollJointState {
    pub parent: BodyPart,
    pub child: BodyPart,
    pub anchor_error: f32,
    pub contacts_enabled: bool,
}

pub struct PassiveRagdoll {
    world: PhysicsWorld,
    bodies: [RigidBodyHandle; 15],
    definition: CharacterDefinition,
    physics: PhysicsConfig,
    spawn: [f32; 3],
    completed_tick: u64,
    pending_impulse: Option<(BodyPart, [f32; 3])>,
}

impl PassiveRagdoll {
    pub fn new(
        physics: PhysicsConfig,
        definition: CharacterDefinition,
        spawn: [f32; 3],
    ) -> Result<Self, CharacterError> {
        physics.validate()?;
        definition.validate()?;
        // Keep this diagnostic drop above its finite platform. Map/player spawn
        // validation will have its own geometry contract when arenas are added.
        if !spawn.iter().all(|value| value.is_finite())
            || spawn[0].abs() > 4.0
            || spawn[2].abs() > 4.0
            || !(0.0..=10.0).contains(&spawn[1])
        {
            return Err(CharacterError::Invalid(
                "spawn outside diagnostic drop bounds",
            ));
        }
        definition.validate_spawn(spawn)?;
        let mut world = PhysicsWorld::new(&physics);
        world
            .colliders
            .insert(ColliderBuilder::cuboid(6.0, 0.5, 6.0).translation(vector![0.0, -0.5, 0.0]));
        let bodies = insert_rig(&mut world, &definition, spawn);
        Ok(Self {
            world,
            bodies,
            definition,
            physics,
            spawn,
            completed_tick: 0,
            pending_impulse: None,
        })
    }

    pub fn tick(&self) -> u64 {
        self.completed_tick
    }

    pub fn step(&mut self) {
        if let Some((part, impulse)) = self.pending_impulse.take() {
            self.world.bodies[self.bodies[part.index()]].apply_impulse(impulse.into(), true);
        }
        self.world.step();
        self.completed_tick += 1;
    }

    pub fn queue_impulse(&mut self, part: BodyPart, impulse: [f32; 3]) -> Result<(), CommandError> {
        validate_impulse(impulse)?;
        if self.pending_impulse.is_some() {
            return Err(CommandError::ImpulsePending);
        }
        self.pending_impulse = Some((part, impulse));
        Ok(())
    }

    pub fn counts(&self) -> (usize, usize, usize) {
        (
            self.world.bodies.len(),
            self.world.colliders.len(),
            self.world.impulse_joints.len(),
        )
    }

    pub fn body_states(&self) -> Vec<RagdollBodyState> {
        BodyPart::ALL
            .into_iter()
            .map(|part| {
                let body = &self.world.bodies[self.bodies[part.index()]];
                RagdollBodyState {
                    part,
                    state: FixtureState {
                        position: (*body.translation()).into(),
                        rotation: body.rotation().coords.into(),
                        linear_velocity: (*body.linvel()).into(),
                        angular_velocity: (*body.angvel()).into(),
                        mass: body.mass(),
                    },
                }
            })
            .collect()
    }

    pub fn joint_states(&self) -> Vec<RagdollJointState> {
        self.world
            .impulse_joints
            .iter()
            .map(|(_, joint)| {
                let anchor1 =
                    self.world.bodies[joint.body1].position() * joint.data.local_anchor1();
                let anchor2 =
                    self.world.bodies[joint.body2].position() * joint.data.local_anchor2();
                RagdollJointState {
                    parent: BodyPart::ALL[self
                        .bodies
                        .iter()
                        .position(|handle| *handle == joint.body1)
                        .expect("owned anatomical body")],
                    child: BodyPart::ALL[self
                        .bodies
                        .iter()
                        .position(|handle| *handle == joint.body2)
                        .expect("owned anatomical body")],
                    anchor_error: (anchor1 - anchor2).norm(),
                    contacts_enabled: joint.data.contacts_enabled(),
                }
            })
            .collect()
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.world.collider_lines()
    }

    pub fn reset(&mut self) {
        // Rebuild solver caches and constraints, not just visible body poses.
        *self = Self::new(self.physics.clone(), self.definition.clone(), self.spawn)
            .expect("validated rig configuration");
    }
}
