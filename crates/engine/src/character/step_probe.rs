use super::{BalanceConfig, BodyPart, rig_frame::ReadRig, support::FootSupport};
use crate::physics::PhysicsWorld;
use rapier3d::prelude::*;

struct FootProbe {
    sole: Point<Real>,
    direction: Vector<Real>,
    half_length: f32,
    half_width: f32,
    reach: f32,
    groups: InteractionGroups,
}

impl FootProbe {
    fn sees_low_tread(&self, world: &PhysicsWorld, support: &BalanceConfig) -> bool {
        let margin = support.maximum_contact_distance_m;
        let filter = QueryFilter::only_fixed()
            .exclude_sensors()
            .groups(self.groups);
        let ray = Ray::new(self.sole + Vector::y() * margin, self.direction);
        let Some((collider, face)) = world.queries.cast_ray_and_get_normal(
            &world.bodies,
            &world.colliders,
            &ray,
            self.half_length + self.reach,
            true,
            filter,
        ) else {
            return false;
        };
        if face.normal.y >= support.minimum_normal_y || face.normal.dot(&self.direction) >= 0.0 {
            return false;
        }
        let edge = self.sole + self.direction * face.time_of_impact;
        let side = self.direction.cross(&Vector::y());
        // A wall, rail or overhang may have the same leading face but cannot carry the foot.
        let landing = |along, across| {
            let origin = edge + self.direction * along + side * across + Vector::y() * self.reach;
            let down = Ray::new(origin, -Vector::y());
            world
                .queries
                .cast_ray_and_get_normal(
                    &world.bodies,
                    &world.colliders,
                    &down,
                    self.reach,
                    true,
                    filter,
                )
                .is_some_and(|(id, hit)| {
                    let rise = self.reach - hit.time_of_impact;
                    id == collider
                        && hit.normal.y >= support.minimum_normal_y
                        && rise > margin
                        && rise <= self.reach * 0.5
                })
        };
        [margin, self.half_length * 2.0 + margin]
            .into_iter()
            .all(|along| {
                [-self.half_width, self.half_width]
                    .into_iter()
                    .all(|across| landing(along, across))
            })
    }
}

pub(super) fn approaching(
    rig: ReadRig<'_>,
    direction: Vector<Real>,
    supports: [Option<FootSupport>; 2],
    config: &BalanceConfig,
) -> bool {
    let Some(fallback) = supports.into_iter().flatten().next() else {
        return false;
    };
    let bind = |part: BodyPart| Vector::from(rig.parts.definition.bodies[part.index()].position);
    let reach = (bind(BodyPart::LeftThigh) - bind(BodyPart::LeftShin)).norm();
    [BodyPart::LeftFoot, BodyPart::RightFoot]
        .into_iter()
        .zip(supports)
        .any(|(part, support)| {
            let foot = &rig.world.bodies[rig.parts.bodies[part.index()]];
            let collider = &rig.world.colliders[foot.colliders()[0]];
            let dims = collider.shape().compute_local_aabb().half_extents();
            let projected =
                |axis: Vector<Real>| dims.dot(&(foot.rotation().inverse() * axis).abs());
            let mut sole = *foot.center_of_mass();
            let support = support.unwrap_or(fallback);
            sole.y -= (sole - support.contact_point).dot(&support.normal) / support.normal.y;
            FootProbe {
                sole,
                direction,
                half_length: projected(direction),
                half_width: projected(direction.cross(&Vector::y())),
                reach,
                groups: collider.collision_groups(),
            }
            .sees_low_tread(rig.world, config)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foothold_requires_a_low_solid_face_with_room_for_the_whole_sole() {
        for (width, depth, height, sensor, expected) in [
            (1.0, 0.8, 0.1, false, true),
            (0.08, 0.8, 0.1, false, false),
            (1.0, 0.12, 0.1, false, false),
            (1.0, 0.8, 0.5, false, false),
            (1.0, 0.8, 0.1, true, false),
        ] {
            let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
            world.colliders.insert(
                ColliderBuilder::cuboid(width / 2., height / 2., depth / 2.)
                    .translation(vector![0., height / 2., -0.3 - depth / 2.])
                    .sensor(sensor),
            );
            world.step();
            let probe = FootProbe {
                sole: Point::origin(),
                direction: -Vector::z(),
                half_length: 0.16,
                half_width: 0.10,
                reach: 0.30,
                groups: InteractionGroups::all(),
            };
            assert_eq!(
                probe.sees_low_tread(&world, &BalanceConfig::bundled().unwrap()),
                expected
            );
        }
    }
}
