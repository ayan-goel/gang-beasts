use super::*;
use crate::{Consciousness, Hand, MeleePhase};

fn fighters_in_ring(hand: Hand) -> Duel {
    let mut game = Duel::skyline_match_with_players(PhysicsConfig::default(), 8).unwrap();
    for _ in 0..180 {
        game.step();
    }
    assert_eq!(game.match_state().unwrap().phase, MeleePhase::Playing);
    let yaw_offset = match hand {
        Hand::Right => -0.8,
        Hand::Left => 0.8,
    };
    for (index, fighter) in game.fighters.iter().enumerate() {
        let angle = index as f32 * std::f32::consts::FRAC_PI_4;
        let center = vector![angle.sin(), 0., angle.cos()];
        // Turn each rig toward its next neighbor so ordinary reach can close the final link.
        let heading = Rotation::from_axis_angle(&Vector::y_axis(), angle + yaw_offset);
        for (&handle, limb) in fighter.bodies.iter().zip(&game.definition.bodies) {
            let position = heading * Vector::from(limb.position) + center;
            game.world.bodies[handle].set_position(
                Isometry::from_parts(Translation::from(position), heading),
                true,
            );
        }
    }
    game
}

#[derive(Default, Debug)]
struct Peaks {
    speed: f32,
    angular_speed: f32,
    energy: f32,
    anatomical_error: f32,
    grip_error: f32,
    grip_load: f32,
}

impl Peaks {
    fn observe(&mut self, game: &Duel, hand: Hand) -> f32 {
        let mut energy = 0.;
        for (&id, fighter) in FighterId::ALL.iter().zip(&game.fighters) {
            assert_ne!(game.vitals(id).phase, Consciousness::Eliminated);
            let parts = fighter.parts(&game.definition, &game.physics, game.tick);
            for handle in parts.bodies {
                let body = &game.world.bodies[handle];
                assert!(
                    body.is_enabled()
                        && body
                            .translation()
                            .iter()
                            .chain(body.rotation().coords.iter())
                            .chain(body.linvel())
                            .chain(body.angvel())
                            .all(|v| v.is_finite())
                );
                energy += body.kinetic_energy();
                self.speed = self.speed.max(body.linvel().norm());
                self.angular_speed = self.angular_speed.max(body.angvel().norm());
            }
            for handle in parts.joints {
                let joint = game.world.impulse_joints.get(handle).unwrap();
                let world_anchor =
                    |body: RigidBodyHandle, local| game.world.bodies[body].position() * local;
                let error = nalgebra::distance(
                    &world_anchor(joint.body1, joint.data.local_anchor1()),
                    &world_anchor(joint.body2, joint.data.local_anchor2()),
                );
                assert!(error.is_finite());
                self.anatomical_error = self.anatomical_error.max(error);
            }
            if let Some(grip) = game.grip(id, hand) {
                let fist = fighter.bodies[hand.body().index()];
                let other = game.fighters[grip.target as usize].bodies[grip.region.index()];
                let a = game.world.bodies[fist].position() * Point::from(grip.hand_anchor);
                let b = game.world.bodies[other].position() * Point::from(grip.target_anchor);
                let error = nalgebra::distance(&a, &b);
                assert!(grip.load_n.is_finite() && error.is_finite());
                self.grip_load = self.grip_load.max(grip.load_n);
                self.grip_error = self.grip_error.max(error);
            }
        }
        assert!(energy.is_finite());
        self.energy = self.energy.max(energy);
        energy
    }
}

#[test]
fn eight_fighters_form_one_closed_loop_then_release_every_constraint() {
    for hand in [Hand::Right, Hand::Left] {
        solve_ring(hand);
    }
}

fn solve_ring(hand: Hand) {
    let mut game = fighters_in_ring(hand);
    for owner in FighterId::ALL {
        game.set_hand(owner, hand, true);
    }
    let mut peaks = Peaks::default();
    let mut acquired = None;
    let mut pulling_energy = 0.;
    let mut resting_energy = 0.;
    for tick in 1..=2340 {
        if tick == 120 || tick == 165 {
            for id in FighterId::ALL {
                let angle = id as usize as f32 * std::f32::consts::FRAC_PI_4;
                let direction = if tick == 120 {
                    [angle.sin(), angle.cos()]
                } else {
                    [0.; 2]
                };
                game.set_input(id, direction, false).unwrap();
            }
        }
        if tick == 540 {
            for owner in FighterId::ALL {
                game.set_hand(owner, hand, false);
            }
        }
        game.step();
        let energy = peaks.observe(&game, hand);
        if (120..165).contains(&tick) {
            pulling_energy += energy;
        }
        if tick > 2280 {
            resting_energy += energy;
        }
        if (60..540).contains(&tick) {
            let created = FighterId::ALL.map(|owner| {
                let next_slot = match hand {
                    Hand::Right => 1,
                    Hand::Left => 7,
                };
                let next = FighterId::ALL[(owner as usize + next_slot) % 8];
                let grip = game
                    .grip(owner, hand)
                    .expect("every fighter must keep its neighbor");
                assert_eq!(grip.target, next);
                grip.created_tick
            });
            assert_eq!(created, *acquired.get_or_insert(created));
            assert_eq!(game.world.impulse_joints.iter().count(), 8 * 15);
        }
        if tick >= 540 {
            assert_eq!(game.world.impulse_joints.iter().count(), 8 * 14);
            assert!(
                FighterId::ALL
                    .into_iter()
                    .all(|id| game.grip(id, hand).is_none())
            );
        }
    }
    let mean_pulling = pulling_energy / 45.;
    let mean_rest = resting_energy / 60.;
    eprintln!(
        "{hand:?}: acquired {acquired:?}, {peaks:?}, pulling {mean_pulling} J, at rest {mean_rest} J"
    );
    assert!(
        peaks.grip_load > 100.,
        "outward movement must load the ring"
    );
    // Anatomical anchors use the spec tolerance. Grip springs allow force-limited travel.
    assert!(peaks.anatomical_error < 0.05 && peaks.grip_error < 0.31);
    assert!(peaks.speed < 12. && peaks.angular_speed < 100. && peaks.energy < 1000.);
    // Compare full-roster kinetic energy after all get-ups with the loaded interval.
    assert!(mean_rest < 1. && mean_rest < mean_pulling * 0.01);
}
