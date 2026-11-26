use super::{BodyPart, CharacterDefinition, FighterId, duel_fighter::Fighter};
use crate::{Hand, physics::PhysicsWorld};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct Aim {
    pub fighter: FighterId,
    region: BodyPart,
    alignment: f32,
    distance: f32,
    pub yaw: f32,
    pitch: f32,
}

impl Aim {
    fn preferred(self, other: Self) -> bool {
        self.alignment
            .total_cmp(&other.alignment)
            .reverse()
            .then(self.distance.total_cmp(&other.distance))
            .then(
                (self.fighter as usize, self.region.index())
                    .cmp(&(other.fighter as usize, other.region.index())),
            )
            .is_lt()
    }

    pub fn steer(
        self,
        definition: &CharacterDefinition,
        hand: Hand,
        weight: f32,
        angles: &mut [[f32; 3]; 14],
        rotations: &mut [Rotation<Real>; 14],
    ) {
        if weight <= 0. || (self.pitch == 0. && self.yaw == 0.) {
            return;
        }
        let index = definition
            .joints
            .iter()
            .position(|j| j.child == hand.parts()[0])
            .expect("shoulder joint");
        let limits = definition.joints[index].angular_limits;
        let pose = &mut angles[index];
        pose[0] = (pose[0] + weight * self.pitch).clamp(limits[0][0], limits[0][1]);
        pose[1] = (pose[1] + weight * self.yaw).clamp(limits[1][0], limits[1][1]);
        rotations[index] = Rotation::from_euler_angles(pose[0], pose[1], pose[2]);
    }
}

fn reachable_point(collider: &Collider, ray: &Ray, reach: f32) -> Option<(Point<Real>, bool)> {
    let depth = (collider.translation() - ray.origin.coords).dot(&ray.dir);
    if depth <= 0. {
        return None;
    }
    // An intersecting ray already puts the ordinary punch on this surface.
    if let Some(t) = collider
        .shape()
        .cast_ray(collider.position(), ray, reach, true)
    {
        return Some((ray.point_at(t), true));
    }
    let reference = ray.point_at(depth.min(reach));
    let point = collider
        .shape()
        .project_point(collider.position(), &reference, true)
        .point;
    ((point - ray.origin).norm_squared() <= reach * reach).then_some((point, false))
}

fn blocked(
    world: &PhysicsWorld,
    attacker: &Fighter,
    target: ColliderHandle,
    segment: &Ray,
) -> bool {
    world.colliders.iter().any(|(id, c)| {
        id != target
            && c.is_enabled()
            && !c.is_sensor()
            && !c
                .parent()
                .is_some_and(|p| attacker.bodies.contains(&p) || !world.bodies[p].is_enabled())
            && c.shape()
                .cast_ray(c.position(), segment, 1., true)
                .is_some_and(|t| t < 1. - 1e-5)
    })
}

fn in_clinch(world: &PhysicsWorld, attacker: &Fighter, target: &Fighter, fist_width: f32) -> bool {
    let hips = |fighter: &Fighter| {
        let body = &world.bodies[fighter.bodies[BodyPart::Pelvis.index()]];
        let collider = &world.colliders[body.colliders()[0]];
        (body.is_enabled() && collider.is_enabled() && !collider.is_sensor()).then_some(collider)
    };
    let (Some(a), Some(b)) = (hips(attacker), hips(target)) else {
        return false;
    };
    rapier3d::parry::query::distance(a.position(), a.shape(), b.position(), b.shape())
        .is_ok_and(|gap| gap <= fist_width)
}

pub(super) fn select(
    world: &PhysicsWorld,
    definition: &CharacterDefinition,
    fighters: &[Fighter],
    vitals: &[crate::Vitals],
    owner: FighterId,
    hand: Hand,
) -> Option<Aim> {
    let attacker = &fighters[owner as usize];
    let chest = &world.bodies[attacker.bodies[BodyPart::Chest.index()]];
    let joint = definition
        .joints
        .iter()
        .find(|j| j.child == hand.parts()[0])
        .expect("shoulder joint");
    let rest = |part: BodyPart| Vector::from(definition.bodies[part.index()].position);
    let anchor = Vector::from(joint.anchor);
    let shoulder = chest.position() * Point::from(anchor - rest(BodyPart::Chest));
    let palm = world.colliders[attacker.hand_collider(world, hand)]
        .shape()
        .compute_local_aabb()
        .half_extents()
        .min();
    let length = (anchor - rest(hand.body())).norm() + palm;
    let forward = chest.rotation() * -Vector::z();
    let ray = Ray::new(shoulder, forward);
    let rules = &attacker.hands.tuning;
    let mut best: Option<Aim> = None;
    for (&fighter, opponent) in FighterId::ALL.iter().zip(fighters) {
        if fighter == owner || vitals[fighter as usize].phase != crate::Consciousness::Active {
            continue;
        }
        for region in BodyPart::ALL {
            let body = &world.bodies[opponent.bodies[region.index()]];
            if !body.is_enabled() {
                continue;
            }
            let handle = body.colliders()[0];
            let collider = &world.colliders[handle];
            if !collider.is_enabled() || collider.is_sensor() {
                continue;
            }
            let Some((point, straight)) = reachable_point(collider, &ray, length) else {
                continue;
            };
            let delta = point - shoulder;
            let distance = delta.norm();
            if distance < 1e-5 || distance > length {
                continue;
            }
            let local = chest.rotation().inverse() * delta;
            let alignment = if straight {
                1.
            } else {
                (-local.z / distance).clamp(-1., 1.)
            };
            if alignment < rules.aim_cone_cos {
                continue;
            }
            let span = local.x.hypot(local.y);
            let correction = if span <= palm {
                0.
            } else {
                (span - palm) / span
            };
            let cap = rules.aim_max_rad;
            let candidate = Aim {
                fighter,
                region,
                alignment,
                distance,
                yaw: ((-local.x).atan2(-local.z) * correction).clamp(-cap, cap),
                pitch: (local.y.atan2(local.z.hypot(local.x)) * correction).clamp(-cap, cap),
            };
            if best.is_some_and(|old| !candidate.preferred(old)) {
                continue;
            }
            let line = Ray::new(shoulder, delta);
            if !blocked(world, attacker, handle, &line) {
                best = Some(candidate);
            }
        }
    }
    let mut aim = best?;
    // Redirecting the arm between pressed-together fighters can pull an ordinary punch away.
    if (aim.yaw != 0. || aim.pitch != 0.)
        && in_clinch(world, attacker, &fighters[aim.fighter as usize], 2. * palm)
    {
        aim.yaw = 0.;
        aim.pitch = 0.;
    }
    Some(aim)
}
