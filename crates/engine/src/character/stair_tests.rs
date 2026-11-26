use super::*;

fn approach(settle_ticks: usize, yaw: f32) -> (LocomotionRagdoll, Vector<Real>) {
    let mut rig = LocomotionRagdoll::stairs(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
    )
    .unwrap();
    let heading = Rotation::from_axis_angle(&Vector::y_axis(), yaw);
    let direction = heading * -Vector::z();
    if yaw != 0.0 {
        let world = &mut rig.pose.rig.world;
        let rotation = Isometry::from_parts(Translation::identity(), heading);
        for handle in rig.pose.rig.bodies {
            let body = &mut world.bodies[handle];
            body.set_position(rotation * body.position(), true);
        }
        for (_, collider) in world.colliders.iter_mut() {
            if collider.parent().is_none() {
                collider.set_position(rotation * collider.position());
            }
        }
    }
    for _ in 0..settle_ticks {
        rig.step();
    }
    rig.set_input([direction.x, direction.z], false).unwrap();
    (rig, direction)
}

fn advance_upright(rig: &mut LocomotionRagdoll, ticks: usize) {
    let pelvis = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
    let mut previous = *rig.pose.rig.world.bodies[pelvis].translation();
    for _ in 0..ticks {
        rig.step();
        bounded(rig, Vector::y());
        let position = *rig.pose.rig.world.bodies[pelvis].translation();
        assert!(
            (position - previous).norm() < 0.15,
            "pelvis displacement at tick {}",
            rig.pose.rig.tick()
        );
        previous = position;
    }
}

#[test]
fn walking_climbs_three_low_risers_without_teleporting() {
    let (mut rig, _) = approach(60, 0.0);
    advance_upright(&mut rig, 150);
    let pelvis = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
    let position = rig.pose.rig.world.bodies[pelvis].translation();
    assert!(
        position.z < -3.0 && position.y > 0.85,
        "final pelvis {position:?}"
    );
}

#[test]
fn walking_continues_over_the_landing_and_stops_on_the_top_tread() {
    for settle_ticks in [59, 60, 61, 69, 75, 84] {
        let (rig, direction) = approach(settle_ticks, 0.0);
        stopped_on_landing(rig, direction);
    }
}

#[test]
fn climbing_and_stopping_follow_the_rotated_treads() {
    for quarter_turns in 1..=3 {
        let (rig, direction) = approach(60, std::f32::consts::FRAC_PI_2 * quarter_turns as f32);
        stopped_on_landing(rig, direction);
    }
}

fn stopped_on_landing(mut rig: LocomotionRagdoll, direction: Vector<Real>) {
    // A tipping torso can cross the last edge while the feet are still behind it.
    // Continue for another 1.5 s, then require a supported stop on the landing.
    advance_upright(&mut rig, 240);
    rig.set_input([0.0, 0.0], false).unwrap();
    advance_upright(&mut rig, 180);
    let pelvis = &rig.pose.rig.world.bodies[rig.pose.rig.bodies[BodyPart::Pelvis.index()]];
    assert!(pelvis.translation().dot(&direction) > 3.0 && pelvis.translation().y > 0.85);
    assert!(pelvis.linvel().norm() < 0.2);
    let config = BalanceConfig::bundled().unwrap();
    for part in [BodyPart::LeftFoot, BodyPart::RightFoot] {
        let support = crate::character::support::foot_support(
            &rig.pose.rig.world,
            rig.pose.rig.bodies[part.index()],
            &rig.pose.rig.bodies,
            &config,
        )
        .expect("both stationary feet must carry support");
        assert!((support.contact_point.y - 0.30).abs() < 0.015);
    }
}
