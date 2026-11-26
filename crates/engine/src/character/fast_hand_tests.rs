use super::*;
use crate::{BodyPart, PhysicsConfig};

fn approach(hand: BodyPart, direction: f32, speed: f32, solid: bool, offset: f32) {
    let mut world = PhysicsWorld::new(&PhysicsConfig::default());
    let definition = CharacterDefinition::bundled().unwrap();
    let (limbs, joints) = insert_rig(&mut world, &definition, [0.; 3]);
    let fist = limbs[hand.index()];
    let x = world.bodies[fist].translation().x;
    let plane = if speed < 9. { 0.093 } else { 0.115 };
    world.colliders.insert(
        ColliderBuilder::cuboid(0.09, 0.025, 0.01)
            .translation(vector![x + offset, 0.66, direction * plane])
            .sensor(!solid),
    );
    for h in limbs {
        world.bodies[h].set_linvel(vector![0., 0., direction * speed], true);
    }
    let contact_expected = solid && offset == 0.;
    let mut peak_error = 0.0_f32;
    let initial_momentum = 40. * speed;
    let mut peak_momentum_loss = 0.0_f32;
    let mut previous = *world.bodies[fist].translation();
    for tick in 1..=8 {
        world.step();
        let body = &world.bodies[fist];
        let p = *body.translation();
        let center = direction * p.z;
        if tick == 1 {
            if contact_expected {
                assert!(
                    center <= plane - 0.01,
                    "{hand:?} {speed} m/s, sign {direction}: center reached {center}"
                );
            } else {
                assert!(
                    (center - speed * crate::TICK_SECONDS).abs() < 0.003,
                    "free passage {center}"
                );
            }
        }
        if contact_expected && direction * previous.z < plane && center >= plane {
            let t = (direction * plane - previous.z) / (p.z - previous.z);
            let crossing = previous + (p - previous) * t;
            assert!(
                (crossing.x - x).abs() > 0.09 || (crossing.y - 0.66).abs() > 0.025,
                "crossed solid at {crossing:?}"
            );
        }
        // Only the barrier can change total horizontal momentum in this airborne rig.
        let momentum: f32 = limbs
            .iter()
            .map(|&h| {
                let b = &world.bodies[h];
                b.mass() * direction * b.linvel().z
            })
            .sum();
        peak_momentum_loss = peak_momentum_loss.max(initial_momentum - momentum);
        for handle in joints {
            let joint = world.impulse_joints.get(handle).unwrap();
            let a = world.bodies[joint.body1].position() * joint.data.local_anchor1();
            let b = world.bodies[joint.body2].position() * joint.data.local_anchor2();
            peak_error = peak_error.max((a - b).norm());
        }
        previous = p;
    }
    eprintln!(
        "{hand:?} {speed} m/s sign {direction} solid {solid} offset {offset}: peak error {peak_error}, external impulse {peak_momentum_loss}"
    );
    assert!(peak_error < 0.05);
    assert_eq!(peak_momentum_loss > 0.5, contact_expected);
}

#[test]
fn a_fast_articulated_hand_stops_at_a_thin_solid_from_either_side() {
    for hand in [BodyPart::LeftHand, BodyPart::RightHand] {
        for direction in [-1., 1.] {
            for speed in [7.5, 15., 20.] {
                approach(hand, direction, speed, true, 0.);
            }
        }
    }
}

#[test]
fn fast_hands_pass_through_sensors_and_clear_a_solid_outside_their_path() {
    approach(BodyPart::LeftHand, -1., 15., false, 0.);
    approach(BodyPart::RightHand, 1., 15., true, 0.4);
}
