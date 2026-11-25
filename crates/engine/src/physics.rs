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
    pub narrow_phase: NarrowPhase,
    pub impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd: CCDSolver,
    debug_renderer: DebugRenderPipeline,
}

impl PhysicsWorld {
    #[cfg(test)]
    pub fn remove_collider(&mut self, handle: ColliderHandle) -> Option<Collider> {
        self.colliders
            .remove(handle, &mut self.islands, &mut self.bodies, true)
    }

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
            debug_renderer: DebugRenderPipeline::new(
                DebugRenderStyle::default(),
                DebugRenderMode::COLLIDER_SHAPES,
            ),
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

    pub fn collider_lines(&mut self) -> Vec<f32> {
        let mut lines = crate::debug::DebugLines::default();
        self.debug_renderer.render(
            &mut lines,
            &self.bodies,
            &self.colliders,
            &self.impulse_joints,
            &self.multibody_joints,
            &self.narrow_phase,
        );
        lines.0
    }
}
