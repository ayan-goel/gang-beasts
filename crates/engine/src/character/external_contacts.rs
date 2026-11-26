use super::{BodyPart, PassiveRagdoll};
use rapier3d::prelude::SolverFlags;

impl PassiveRagdoll {
    /// Completed-solve contacts with the environment, in world XYZ, inward normal XYZ and N·s.
    pub fn external_contacts(&self, part: BodyPart) -> Vec<[f32; 7]> {
        let body = &self.world.bodies[self.bodies[part.index()]];
        let mut contacts = Vec::new();
        for &handle in body.colliders() {
            let collider = &self.world.colliders[handle];
            for pair in self.world.narrow_phase.contact_pairs_with(handle) {
                let first = pair.collider1 == handle;
                let other = if first {
                    pair.collider2
                } else {
                    pair.collider1
                };
                if !pair.has_any_active_contact
                    || self.world.colliders[other]
                        .parent()
                        .is_some_and(|body| self.bodies.contains(&body))
                {
                    continue;
                }
                for manifold in &pair.manifolds {
                    if !manifold
                        .data
                        .solver_flags
                        .contains(SolverFlags::COMPUTE_IMPULSES)
                    {
                        continue;
                    }
                    // Rapier orients manifold normals from collider one toward collider two.
                    let normal = if first {
                        -manifold.data.normal
                    } else {
                        manifold.data.normal
                    };
                    for point in manifold.points.iter().filter(|p| p.data.impulse > 0.0) {
                        let local = if first {
                            point.local_p1
                        } else {
                            point.local_p2
                        };
                        let world = collider.position() * local;
                        contacts.push([
                            world.x,
                            world.y,
                            world.z,
                            normal.x,
                            normal.y,
                            normal.z,
                            point.data.impulse,
                        ]);
                    }
                }
            }
        }
        contacts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CharacterDefinition, PhysicsConfig};
    use rapier3d::prelude::*;

    #[test]
    fn isolated_riser_impact_points_back_toward_the_selected_foot() {
        let mut rig = PassiveRagdoll::new(
            PhysicsConfig::default(),
            CharacterDefinition::bundled().unwrap(),
            [0.0, 1.0, 0.0],
        )
        .unwrap();
        rig.world.colliders.insert(
            ColliderBuilder::cuboid(0.07, 0.06, 0.4).translation(vector![0.11, 1.07, -0.65]),
        );
        for handle in rig.bodies {
            rig.world.bodies[handle].set_linvel(vector![0.0, 0.0, -2.0], true);
        }
        for _ in 0..5 {
            rig.step();
            let hits = rig.external_contacts(BodyPart::RightFoot);
            if !hits.is_empty() {
                assert!(
                    hits.iter()
                        .all(|[_, _, z, _, _, nz, impulse]| (z + 0.25).abs() < 0.025
                            && *nz > 0.99
                            && *impulse > 0.0)
                );
                assert!(rig.external_contacts(BodyPart::LeftFoot).is_empty());
                return;
            }
        }
        panic!("right foot never struck the riser");
    }
}
