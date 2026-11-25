use super::PassiveRagdoll;
use rapier3d::prelude::*;

pub(super) fn install(rig: &mut PassiveRagdoll) {
    let colliders = &mut rig.world.colliders;
    // Preserve the original regression's collider order. Different contact ordering
    // can change the motion after a foot hits a riser.
    for (_, collider) in colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    colliders.insert(ColliderBuilder::cuboid(30.0, 0.5, 30.0).translation(vector![0.0, -0.5, 0.0]));
    for (front, back, height) in [(-1.0, -1.8, 0.1), (-1.8, -2.6, 0.2), (-2.6, -12.0, 0.3)] {
        colliders.insert(
            ColliderBuilder::cuboid(3.0, height / 2.0, (front - back) / 2.0).translation(vector![
                0.0,
                height / 2.0,
                (front + back) / 2.0
            ]),
        );
    }
}
