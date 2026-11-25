use super::*;

fn release_foot_placement(
    stance_pitch: f32,
    yaw: f32,
    offset: Vector<Real>,
    spin: f32,
    translation: Vector<Real>,
) -> (f32, f32, f32) {
    let (mut rig, deck) = rotating_deck(0.0, 0.0);
    let heading = Rotation::from_axis_angle(&Vector::y_axis(), yaw);
    for _ in 0..60 {
        rig.step();
    }
    let forward = heading * -Vector::z();
    rig.set_input([forward.x, forward.z], true).unwrap();
    for _ in 0..15 {
        rig.step();
    }
    let definition = rig.pose.rig.definition.clone();
    let bind = |p: BodyPart| Vector::from(definition.bodies[p.index()].position);
    let mut poses = BodyPart::ALL.map(|p| Isometry::translation(bind(p).x, bind(p).y, bind(p).z));
    for (parts, pitch) in [
        (
            [
                BodyPart::RightThigh,
                BodyPart::RightShin,
                BodyPart::RightFoot,
            ],
            [stance_pitch, 0., -stance_pitch],
        ),
        (
            [BodyPart::LeftThigh, BodyPart::LeftShin, BodyPart::LeftFoot],
            [0.8, -1.3, 0.5],
        ),
    ] {
        for (p, pitch) in parts.into_iter().zip(pitch) {
            let j = definition.joints.iter().find(|j| j.child == p).unwrap();
            let origin = Vector::from(j.anchor);
            let parent = poses[j.parent.index()];
            let rotation = parent.rotation * Rotation::from_axis_angle(&Vector::x_axis(), pitch);
            let center = parent.translation.vector + parent.rotation * (origin - bind(j.parent))
                - rotation * (origin - bind(p));
            poses[p.index()] = Isometry::from_parts(Translation::from(center), rotation);
        }
    }
    let elevation = bind(BodyPart::RightFoot).y - poses[BodyPart::RightFoot.index()].translation.y;
    let frame = Isometry::from_parts(Translation::from(offset + Vector::y() * elevation), heading);
    let poses = poses.map(|pose| frame * pose);
    let center = BodyPart::ALL
        .into_iter()
        .map(|p| poses[p.index()].translation.vector * p.mass())
        .sum::<Vector<Real>>()
        / 40.;
    let stance = poses[BodyPart::RightFoot.index()].translation.vector;
    let stance_error = forward.dot(&(center - stance));
    let present = |rig: &mut LocomotionRagdoll| {
        let p = &mut rig.pose.rig;
        let world = &mut p.world;
        let support = &mut world.bodies[deck];
        // The COM lies on the deck's spin axis so baseline centrifugal assistance is zero.
        support.set_position(
            Isometry::from_parts(
                Translation::from(vector![center.x, -0.5, center.z]),
                heading,
            ),
            true,
        );
        support.set_linvel(translation, true);
        support.set_angvel(Vector::y() * spin, true);
        for (h, pose) in p.bodies.into_iter().zip(poses) {
            let b = &mut world.bodies[h];
            b.set_position(pose, true);
            b.set_linvel(
                translation + (Vector::y() * spin).cross(&(pose.translation.vector - center)),
                true,
            );
            b.set_angvel(Vector::y() * spin, true);
        }
    };
    present(&mut rig);
    // Rebuild candidates for the articulated poses, then restore the controlled pre-solver state.
    rig.pose.rig.world.step();
    present(&mut rig);
    rig.set_input([0.0; 2], false).unwrap();
    let before = rig.pose.rig.bodies.map(|h| {
        (
            *rig.pose.rig.world.bodies[h].linvel(),
            *rig.pose.rig.world.bodies[h].angvel(),
        )
    });
    rig.controller.advance(&mut rig.pose.rig, true);
    assert_eq!(rig.state().feet, [false, true]);
    let mut impulse = 0.;
    for (h, (v, w)) in rig.pose.rig.bodies.into_iter().zip(before) {
        let b = &rig.pose.rig.world.bodies[h];
        impulse += (b.linvel() - v).norm() * b.mass();
        assert!((b.angvel() - w).norm() < 0.00001);
    }
    assert!(
        impulse < 0.0001,
        "unexpected standalone world impulse {impulse}"
    );
    let a = rig.state().targets;
    let localrot = |i: usize| Rotation::from_euler_angles(a[i][0], a[i][1], a[i][2]);
    let hip = definition
        .joints
        .iter()
        .position(|j| j.child == BodyPart::LeftThigh)
        .unwrap();
    let knee = definition
        .joints
        .iter()
        .position(|j| j.child == BodyPart::LeftShin)
        .unwrap();
    let ankle = definition
        .joints
        .iter()
        .position(|j| j.child == BodyPart::LeftFoot)
        .unwrap();
    let upper =
        Vector::from(definition.joints[knee].anchor) - Vector::from(definition.joints[hip].anchor);
    let lower = Vector::from(definition.joints[ankle].anchor)
        - Vector::from(definition.joints[knee].anchor);
    let target = localrot(hip) * (upper + localrot(knee) * lower);
    let stance_hip = definition
        .joints
        .iter()
        .position(|j| j.child == BodyPart::RightThigh)
        .unwrap();
    let stance_knee = definition
        .joints
        .iter()
        .position(|j| j.child == BodyPart::RightShin)
        .unwrap();
    let stance_target = localrot(stance_hip) * (upper + localrot(stance_knee) * lower);
    (stance_error, -target.z, -stance_target.z)
}

#[test]
fn neutral_placement_responds_to_mirrored_stance_error_with_com_velocities_matched() {
    let mut observations = Vec::new();
    for (yaw, origin, spin, v) in [
        (0., Vector::zeros(), 0., Vector::zeros()),
        (1.1, vector![2., 0., -3.], 0.4, vector![0.8, 0., -0.4]),
    ] {
        let front = release_foot_placement(-0.5, yaw, origin, spin, v);
        let rear = release_foot_placement(0.5, yaw, origin, spin, v);
        let centered = release_foot_placement(0.0, yaw, origin, spin, v);
        observations.push((front, centered, rear));
    }
    for (ahead, centered, behind) in &observations {
        // Both displaced COMs lie outside the 0.16 m sagittal half-length of the rounded sole.
        assert!(ahead.0 > 0.16 && centered.0.abs() < 0.16 && behind.0 < -0.16);
        assert!((ahead.2 - behind.2).abs() < 0.00001);
        assert!(
            ahead.1 > centered.1 + 0.0001 && behind.1 < centered.1 - 0.0001,
            "swing targets ignore signed stance offset: {ahead:?} {behind:?}"
        );
    }
    let ((a, _, b), (c, _, d)) = (observations[0], observations[1]);
    assert!((a.1 - c.1).abs() < 0.00001 && (b.1 - d.1).abs() < 0.00001);
}
