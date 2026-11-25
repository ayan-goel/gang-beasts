use rapier3d::prelude::*;
use std::num::NonZeroUsize;

use crate::{PhysicsConfig, TICK_SECONDS};

pub(crate) struct PhysicsWorld {
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    gravity: Vector<Real>,
    parameters: IntegrationParameters,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
}

impl PhysicsWorld {
    pub fn new(config: &PhysicsConfig) -> Self {
        Self {
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            gravity: vector![0.0, config.gravity_y, 0.0],
            parameters: IntegrationParameters {
                dt: TICK_SECONDS,
                num_solver_iterations: NonZeroUsize::new(config.solver_iterations)
                    .expect("validated nonzero solver iterations"),
                ..IntegrationParameters::default()
            },
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: BroadPhaseMultiSap::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd: CCDSolver::new(),
        }
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd,
            None,
            &(),
            &(),
        );
    }
}
