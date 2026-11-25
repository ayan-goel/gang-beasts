use super::*;
use rapier3d::prelude::*;

struct SettlingDeck {
    rig: LocomotionRagdoll,
    frames: [RigidBodyHandle; 2],
    tiles: [ColliderHandle; 2],
    bind: [Isometry<Real>; 15],
    decay: f32,
}

impl SettlingDeck {
    fn new() -> Self {
        let config = LocomotionConfig::bundled().unwrap();
        let mut rig =
            LocomotionRagdoll::new(PhysicsConfig::default(), config.clone(), [0.0; 3]).unwrap();
        let world = &mut rig.pose.rig.world;
        for (_, collider) in world.colliders.iter_mut() {
            if collider.parent().is_none() {
                collider.set_enabled(false);
            }
        }
        let frames = [0, 1].map(|_| {
            world
                .bodies
                .insert(RigidBodyBuilder::kinematic_velocity_based())
        });
        let tiles = [-1.0, 1.0].map(|x| {
            world.colliders.insert_with_parent(
                ColliderBuilder::cuboid(0.99, 0.5, 3.0).translation(vector![x, -0.5, 0.0]),
                frames[0],
                &mut world.bodies,
            )
        });
        for _ in 0..60 {
            rig.step();
        }
        rig.set_input([0.0, -1.0], false).unwrap();
        rig.step();
        rig.set_input([0.0; 2], false).unwrap();
        for _ in 0..config.braking_ticks {
            rig.step();
        }
        assert_eq!(rig.state().motor_strength, 1.0);
        let bind = rig
            .pose
            .rig
            .bodies
            .map(|h| *rig.pose.rig.world.bodies[h].position());
        Self {
            rig,
            frames,
            tiles,
            bind,
            decay: crate::TICK_SECONDS / config.motor_release_s,
        }
    }

    fn sample(
        &mut self,
        tile_frames: [usize; 2],
        deck_yaw: f32,
        root_yaw: f32,
        deck_rates: [f32; 2],
        root_rate: f32,
        should_settle: bool,
    ) {
        let config = BalanceConfig::bundled().unwrap();
        let before = self.rig.state().motor_strength;
        let rig = &mut self.rig.pose.rig;
        let world = &mut rig.world;
        for (tile, frame) in self.tiles.into_iter().zip(tile_frames) {
            world
                .colliders
                .set_parent(tile, Some(self.frames[frame]), &mut world.bodies);
        }
        for (handle, rate) in self.frames.into_iter().zip(deck_rates) {
            let body = &mut world.bodies[handle];
            body.set_rotation(Rotation::from_axis_angle(&Vector::y_axis(), deck_yaw), true);
            let spin = Vector::y() * rate;
            body.set_angvel(spin, true);
            body.set_linvel(spin.cross(&body.center_of_mass().coords), true);
        }
        let enabled = self.tiles.map(|tile| world.colliders[tile].is_enabled());
        let count = enabled.iter().filter(|v| **v).count();
        let speed = tile_frames
            .iter()
            .zip(enabled)
            .filter(|(_, on)| *on)
            .map(|(frame, _)| deck_rates[*frame])
            .sum::<f32>()
            / count as f32;
        let rotation = Isometry::rotation(Vector::y() * root_yaw);
        for (handle, bind) in rig.bodies.into_iter().zip(self.bind) {
            let body = &mut world.bodies[handle];
            body.set_position(rotation * bind, true);
            body.set_angvel(Vector::y() * root_rate, true);
            // Matching the mean deck field at COM isolates angular eligibility from the speed gate.
            body.set_linvel(
                (Vector::y() * speed).cross(&body.center_of_mass().coords),
                true,
            );
        }
        let frames_at_feet = [BodyPart::LeftFoot, BodyPart::RightFoot].map(|part| {
            super::super::support::foot_support(
                world,
                rig.bodies[part.index()],
                &rig.bodies,
                &config,
            )
            .and_then(|foot| foot.body)
        });
        assert_eq!(
            frames_at_feet,
            core::array::from_fn(|i| enabled[i].then_some(self.frames[tile_frames[i]]))
        );
        self.rig.step();
        assert_eq!(
            self.rig.state().motor_strength,
            if should_settle {
                before - self.decay
            } else {
                before
            },
            "tick={} frames={tile_frames:?} deck_yaw={deck_yaw} root_yaw={root_yaw} rates={deck_rates:?} root_rate={root_rate}",
            self.rig.pose.rig.tick()
        );
    }
}

#[test]
fn settling_observes_completed_shared_deck_rotation_during_velocity_changes() {
    let mut fixture = SettlingDeck::new();
    let ordinary = 0.2 * crate::TICK_SECONDS;
    fixture.sample([0, 0], 0.0, 0.0, [0.2, 0.0], 0.2, true);
    fixture.sample([0, 0], ordinary, ordinary, [1.2, 0.0], 1.2, true);
    fixture.sample(
        [0, 0],
        ordinary + 1.2 * crate::TICK_SECONDS,
        ordinary,
        [0.0, 0.0],
        0.0,
        false,
    );
}

#[test]
fn mixed_and_new_support_bodies_use_current_relative_spin_for_settling() {
    let mut fixture = SettlingDeck::new();
    fixture.sample([0, 0], 0.0, 0.0, [0.0, 0.0], 0.0, true);
    // The new deck has the same orientation. An old pose interval would report a false zero.
    fixture.sample([1, 1], 0.0, 0.0, [0.0, 0.0], 0.8, false);
    fixture.sample([1, 1], 0.0, 0.0, [0.0, 0.0], 0.8, true);
    fixture.sample([0, 1], 0.0, 0.0, [1.0, -0.2], 0.4, true);
    fixture.sample([0, 1], 0.0, 0.0, [1.0, -0.2], 0.0, false);
    fixture.sample([1, 1], 0.0, 0.0, [0.0, 0.0], 0.8, false);
    fixture.sample([1, 1], 0.0, 0.0, [0.0, 0.0], 0.8, true);
    fixture.rig.pose.rig.world.colliders[fixture.tiles[1]].set_enabled(false);
    fixture.sample([1, 1], 0.0, 0.0, [0.0, 0.0], 0.8, true);
}

#[test]
fn distinct_fixed_deck_tiles_share_the_world_settling_frame() {
    let mut fixture = SettlingDeck::new();
    for handle in fixture.frames {
        fixture.rig.pose.rig.world.bodies[handle].set_body_type(RigidBodyType::Fixed, true);
    }
    fixture.sample([0, 0], 0.0, 0.0, [0.0, 0.0], 0.0, true);
    fixture.sample([0, 1], 0.0, 0.0, [0.0, 0.0], 0.8, true);
}
