use super::CharacterDefinition;
use crate::{PhysicsConfig, physics::PhysicsWorld};
use rapier3d::prelude::*;
use std::ops::Deref;

#[derive(Clone, Copy)]
pub(super) struct RigParts<'a> {
    pub bodies: [RigidBodyHandle; 15],
    pub joints: [ImpulseJointHandle; 14],
    pub definition: &'a CharacterDefinition,
    pub physics: &'a PhysicsConfig,
    pub tick: u64,
}

// The room lends its solver to one controller at a time. Every body and joint
// that the controller may drive is identified by this fighter's own handles.
pub(super) struct RigFrame<'a, W> {
    pub world: W,
    pub parts: RigParts<'a>,
}
pub(super) type ReadRig<'a> = RigFrame<'a, &'a PhysicsWorld>;
pub(super) type WriteRig<'a> = RigFrame<'a, &'a mut PhysicsWorld>;

impl<W: Deref<Target = PhysicsWorld>> RigFrame<'_, W> {
    pub fn read(&self) -> ReadRig<'_> {
        RigFrame {
            world: &self.world,
            parts: self.parts,
        }
    }
}

pub(super) fn body_states(rig: ReadRig) -> Vec<super::RagdollBodyState> {
    use super::{BodyPart, RagdollBodyState};
    use crate::FixtureState;
    BodyPart::ALL
        .into_iter()
        .map(|part| {
            let body = &rig.world.bodies[rig.parts.bodies[part.index()]];
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
