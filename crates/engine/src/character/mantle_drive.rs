use super::{BodyPart, CharacterDefinition};
use crate::{TICK_SECONDS, arena::Landing, combat::MantleTuning, physics::PhysicsWorld};
use rapier3d::prelude::*;

pub(super) fn drive(
    world: &mut PhysicsWorld,
    bodies: &[RigidBodyHandle; 15],
    landing: &Landing,
    crossing: bool,
    cap: f32,
    tuning: MantleTuning,
    gravity_y: f32,
) {
    let hip = &world.bodies[bodies[BodyPart::Pelvis.index()]];
    let (depth, height) = if crossing {
        (0.78, 0.18)
    } else {
        (-0.32, 0.45)
    };
    let target = landing.edge + landing.inward * depth + Vector::y() * height;
    let velocity = ((target - hip.center_of_mass()) * 5.).cap_magnitude(tuning.speed_m_s);
    let mass: f32 = bodies.iter().map(|&h| world.bodies[h].mass()).sum();
    let force = ((velocity - hip.linvel()) * 8. - Vector::y() * gravity_y) * mass;
    let impulse = force.cap_magnitude(cap) * TICK_SECONDS;
    // Load the torso while rising. Once it clears the rim, bring the shins inward
    // so the body can transfer weight off the anchored hand onto the roof.
    let shares = if crossing {
        [0.45, 0.15, 0.2, 0.2]
    } else {
        [0.75, 0.25, 0., 0.]
    };
    for (part, share) in [
        BodyPart::Pelvis,
        BodyPart::Chest,
        BodyPart::LeftShin,
        BodyPart::RightShin,
    ]
    .into_iter()
    .zip(shares)
    {
        world.bodies[bodies[part.index()]].apply_impulse(impulse * share, true);
    }
}

pub(super) fn tuck(
    definition: &CharacterDefinition,
    targets: &mut [Rotation<Real>; 14],
    bracing: [bool; 2],
) {
    use BodyPart::*;
    for (index, joint) in definition.joints.iter().enumerate() {
        let pitch: f32 = match joint.child {
            LeftThigh | RightThigh => 1.1,
            LeftShin | RightShin => -1.8,
            LeftUpperArm if bracing[0] => 1.3,
            RightUpperArm if bracing[1] => 1.3,
            LeftForearm if bracing[0] => 1.8,
            RightForearm if bracing[1] => 1.8,
            _ => continue,
        };
        let [min, max] = joint.angular_limits[0];
        targets[index] = Rotation::from_euler_angles(pitch.clamp(min, max), 0., 0.);
    }
}
