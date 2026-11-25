use super::*;
use rapier3d::prelude::*;

fn fallen(rotation: Rotation<Real>) -> LocomotionRagdoll {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
        [0.; 3],
    )
    .unwrap();
    rig.set_enabled(false);
    let world = &mut rig.pose.rig.world;
    let mut lowest = f32::INFINITY;
    for handle in rig.pose.rig.bodies {
        let body = &world.bodies[handle];
        let pose = Isometry::from_parts(
            Translation::from(rotation * body.translation()),
            rotation * body.rotation(),
        );
        let collider = &world.colliders[body.colliders()[0]];
        lowest = lowest.min(
            collider
                .shape()
                .compute_aabb(&(pose * collider.position_wrt_parent().unwrap()))
                .mins
                .y,
        );
        world.bodies[handle].set_position(pose, true);
    }
    for handle in rig.pose.rig.bodies {
        let body = &mut world.bodies[handle];
        body.set_translation(*body.translation() + Vector::y() * (0.02 - lowest), true);
    }
    for _ in 0..90 {
        rig.step();
    }
    assert!(rig.pose.rig.body_states()[0].state.position[1] < 0.65);
    rig.set_enabled(true);
    rig
}

fn verify_recovery(rotation: Rotation<Real>) {
    let mut rig = fallen(rotation);
    let parts = [BodyPart::Pelvis, BodyPart::Chest, BodyPart::Head];
    for tick in 1..=360 {
        let before = rig.pose.rig.body_states();
        rig.step();
        for (old, new) in before.iter().zip(rig.pose.rig.body_states()) {
            assert!(new.state.to_buffer().iter().all(|v| v.is_finite()));
            assert!(
                (Vector::from(old.state.position) - Vector::from(new.state.position)).norm() < 0.16,
                "sudden displacement at {tick}"
            );
        }
        assert!(
            rig.pose
                .rig
                .joint_states()
                .iter()
                .all(|j| j.anchor_error < 0.05)
        );
        if tick >= 300 {
            assert_eq!(rig.state().feet, [true, true], "support at {tick}");
            for part in parts {
                let body = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[part.index()]];
                assert!(
                    (body.rotation() * Vector::y()).y > 0.94,
                    "{part:?} at {tick}"
                );
            }
            assert!(rig.pose.rig.body_states()[0].state.position[1] > 1.2);
        }
    }
}

#[test]
fn recovers_face_up() {
    verify_recovery(Rotation::from_axis_angle(
        &Vector::x_axis(),
        std::f32::consts::FRAC_PI_2,
    ));
}
#[test]
fn recovers_face_down() {
    verify_recovery(Rotation::from_axis_angle(
        &Vector::x_axis(),
        -std::f32::consts::FRAC_PI_2,
    ));
}
#[test]
fn recovers_left_and_right_sides() {
    for sign in [-1., 1.] {
        verify_recovery(Rotation::from_axis_angle(
            &Vector::z_axis(),
            sign * std::f32::consts::FRAC_PI_2,
        ));
    }
}

#[test]
fn obstruction_bounds_each_attempt_and_releasing_the_ceiling_allows_another() {
    let mut rig = fallen(Rotation::from_axis_angle(
        &Vector::x_axis(),
        std::f32::consts::FRAC_PI_2,
    ));
    let ceiling = rig
        .pose
        .rig
        .world
        .colliders
        .insert(ColliderBuilder::cuboid(8., 0.1, 8.).translation(vector![0., 0.70, 0.]));
    let mut active = 0;
    let mut rested_after_effort = false;
    for _ in 0..390 {
        rig.step();
        let strength = rig.state().recovery_strength;
        assert!(rig.state().recovery_loads[0] <= 600. * strength + 1e-3);
        assert!(rig.state().recovery_loads[1] <= 360. * strength + 1e-3);
        if strength > 0. {
            active += 1;
        } else if active > 0 {
            rested_after_effort = true;
        }
        for handle in rig.pose.rig.bodies {
            let world = &rig.pose.rig.world;
            let body = &world.bodies[handle];
            let collider = &world.colliders[body.colliders()[0]];
            let top = collider
                .shape()
                .compute_aabb(&(body.position() * collider.position_wrt_parent().unwrap()))
                .maxs
                .y;
            assert!(
                top < 0.65,
                "ceiling penetration at tick {}: {top}",
                rig.pose.rig.tick()
            );
        }
    }
    assert!(active > 30 && rested_after_effort);
    rig.pose.rig.world.colliders[ceiling].set_enabled(false);
    for _ in 0..420 {
        rig.step();
    }
    assert!(rig.pose.rig.body_states()[0].state.position[1] > 1.2);
    assert_eq!(rig.state().feet, [true, true]);
    assert_eq!(rig.state().recovery_strength, 0.);
}

#[test]
fn losing_the_floor_or_disabling_control_removes_recovery_effort_immediately() {
    for disable in [true, false] {
        let mut rig = fallen(Rotation::from_axis_angle(
            &Vector::z_axis(),
            std::f32::consts::FRAC_PI_2,
        ));
        for _ in 0..180 {
            rig.step();
            if rig.state().recovery_strength > 0. {
                break;
            }
        }
        assert!(rig.state().recovery_loads[0] > 0.);
        if disable {
            rig.set_enabled(false);
        } else {
            for (_, collider) in rig.pose.rig.world.colliders.iter_mut() {
                if collider.parent().is_none() {
                    collider.set_enabled(false);
                }
            }
        }
        for _ in 0..30 {
            rig.step();
            assert_eq!(rig.state().recovery_strength, 0.);
            assert_eq!(rig.state().recovery_loads, [0.; 2]);
        }
    }
}

#[test]
fn measured_recovery_pair_has_zero_net_impulse_and_respects_both_caps() {
    let mut rig = fallen(Rotation::from_axis_angle(
        &Vector::x_axis(),
        -std::f32::consts::FRAC_PI_2,
    ));
    let config = LocomotionConfig::bundled().unwrap().recovery;
    let before = rig.pose.rig.body_states();
    let [force, moment] =
        crate::character::recovery_drive::apply(&mut rig.pose.rig.frame(), 0.5, &config);
    let mut net = Vector::zeros();
    let mut torque = Vector::zeros();
    let mut absolute = 0.;
    for (old, new) in before.iter().zip(rig.pose.rig.body_states()) {
        let impulse = (Vector::from(new.state.linear_velocity)
            - Vector::from(old.state.linear_velocity))
            * new.state.mass;
        net += impulse;
        torque += Vector::from(old.state.position).cross(&impulse);
        absolute += impulse.norm();
        assert_eq!(old.state.position, new.state.position);
        assert_eq!(old.state.rotation, new.state.rotation);
    }
    assert!(net.norm() < 1e-4);
    assert!(force > 0. && force <= 300.001 && moment <= 180.001);
    assert!((absolute - 2. * force * crate::TICK_SECONDS).abs() < 1e-4);
    assert!((torque.norm() - moment * crate::TICK_SECONDS).abs() < 1e-4);
}

#[test]
fn a_constrained_pelvis_stays_held_until_the_constraint_is_removed() {
    let mut rig = fallen(Rotation::from_axis_angle(
        &Vector::z_axis(),
        std::f32::consts::FRAC_PI_2,
    ));
    let pelvis = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
    let world = &mut rig.pose.rig.world;
    let anchored_pose = *world.bodies[pelvis].position();
    let handhold = world
        .bodies
        .insert(RigidBodyBuilder::fixed().position(anchored_pose));
    let restraint = world
        .impulse_joints
        .insert(handhold, pelvis, FixedJointBuilder::new(), true);
    let mut attempted = false;
    for _ in 0..180 {
        rig.step();
        attempted |= rig.state().recovery_strength > 0.;
        let actual = rig.pose.rig.world.bodies[pelvis].position();
        assert!((actual.translation.vector - anchored_pose.translation.vector).norm() < 0.05);
        assert!(actual.rotation.angle_to(&anchored_pose.rotation) < 0.1);
    }
    assert!(attempted);
    rig.pose
        .rig
        .world
        .impulse_joints
        .remove(restraint, true)
        .unwrap();
    for _ in 0..480 {
        rig.step();
    }
    assert!(rig.pose.rig.body_states()[0].state.position[1] > 1.2);
    assert_eq!(rig.state().feet, [true, true]);
}

#[test]
fn a_free_dynamic_support_cannot_authorize_the_ground_assist() {
    let mut rig = fallen(Rotation::from_axis_angle(
        &Vector::x_axis(),
        std::f32::consts::FRAC_PI_2,
    ));
    let world = &mut rig.pose.rig.world;
    for (_, collider) in world.colliders.iter_mut() {
        if collider.parent().is_none() {
            collider.set_enabled(false);
        }
    }
    let prop = world.bodies.insert(
        RigidBodyBuilder::dynamic()
            .translation(vector![0., -0.5, 0.])
            .lock_translations()
            .lock_rotations(),
    );
    world.colliders.insert_with_parent(
        ColliderBuilder::cuboid(4., 0.5, 4.),
        prop,
        &mut world.bodies,
    );
    for _ in 0..120 {
        rig.step();
        assert_eq!(rig.state().recovery_strength, 0.);
    }
}
