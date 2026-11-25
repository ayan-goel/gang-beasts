use super::PassiveRagdoll;
use rapier3d::prelude::*;

pub(super) struct PracticeTarget {
    pad: ColliderHandle,
    pivot: RigidBodyHandle,
}

impl PracticeTarget {
    pub fn install(rig: &mut PassiveRagdoll) -> Self {
        let world = &mut rig.world;
        let center = vector![0., 1.12, -0.54];
        let suspension = vector![0., 0.95, 0.];
        let pivot = world
            .bodies
            .insert(RigidBodyBuilder::fixed().translation(center + suspension));
        let body = world.bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(center)
                .linear_damping(0.22)
                .angular_damping(0.32),
        );
        let pad = world.colliders.insert_with_parent(
            ColliderBuilder::cuboid(0.42, 0.17, 0.055).mass(2.0),
            body,
            &mut world.bodies,
        );
        // The offset joint lets an off-center hit swing and twist the pad through
        // actual contact while gravity restores it beneath the fixed pivot.
        world.impulse_joints.insert(
            pivot,
            body,
            SphericalJointBuilder::new().local_anchor2(Point::from(suspension)),
            true,
        );
        Self { pad, pivot }
    }

    pub fn state(&self, rig: &PassiveRagdoll) -> [f32; 13] {
        let pad = &rig.world.colliders[self.pad];
        let center = pad.translation();
        let q = pad.rotation().coords;
        let size = pad
            .shape()
            .as_cuboid()
            .expect("practice pad geometry")
            .half_extents;
        let pivot = rig.world.bodies[self.pivot].translation();
        [
            center.x, center.y, center.z, q.x, q.y, q.z, q.w, size.x, size.y, size.z, pivot.x,
            pivot.y, pivot.z,
        ]
    }
}
