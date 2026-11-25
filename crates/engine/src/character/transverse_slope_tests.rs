use super::*;
use std::f32::consts::FRAC_PI_2;

#[derive(Clone, Copy, Debug)]
enum Travel {
    Backward,
    CrossSlope,
}

fn check_pose(rig: &LocomotionRagdoll, normal: Vector<Real>) {
    bounded(rig, normal);
    let separation = crate::character::gait::geometry_tests::foot_separation(
        &rig.pose.rig.definition,
        rig.state().targets,
    );
    assert!(
        separation >= -0.00001,
        "tick {} desired foot penetration {}",
        rig.pose.rig.tick(),
        -separation
    );
}

fn traverse(travel: Travel, degrees: f32, quarter_turns: u32, running: bool) {
    let heading = quarter_turns as f32 * FRAC_PI_2;
    let terrain_heading = match travel {
        Travel::Backward => heading,
        Travel::CrossSlope => heading + FRAC_PI_2,
    };
    let (mut rig, normal, facing) = slope_with_headings(degrees, heading, terrain_heading);
    let direction = match travel {
        Travel::Backward => -facing,
        Travel::CrossSlope => {
            // A contour has no vertical rise along the command, even after rotating the fixture.
            assert!(normal.dot(&facing).abs() < 0.00001);
            facing
        }
    };
    eprintln!("{travel:?}, slope={degrees}, quarter_turns={quarter_turns}, running={running}");
    for _ in 0..120 {
        rig.step();
        check_pose(&rig, normal);
    }
    let pelvis = rig.pose.rig.bodies[BodyPart::Pelvis.index()];
    let start = *rig.pose.rig.world.bodies[pelvis].translation();
    rig.set_input([direction.x, direction.z], running).unwrap();
    for _ in 0..240 {
        rig.step();
        check_pose(&rig, normal);
    }
    let travel_distance = (rig.pose.rig.world.bodies[pelvis].translation() - start).dot(&direction);
    let requested_speed = if running { 4.5 } else { 2.5 };
    assert!(
        travel_distance >= 0.7 * requested_speed * 4.0,
        "distance={travel_distance}"
    );

    rig.set_input([0.0; 2], false).unwrap();
    for _ in 0..360 {
        rig.step();
        check_pose(&rig, normal);
    }
    assert!(rig.pose.rig.world.bodies[pelvis].linvel().norm() < 0.2);
    assert_eq!(rig.state().motor_strength, 0.0);
    assert_eq!(rig.state().stopping_moment, [0.0; 3]);
    assert_eq!(rig.state().support_moment, [0.0; 3]);
}

fn check_directions(travel: Travel) {
    let mut failures = Vec::new();
    for degrees in [-15.0, 15.0] {
        for quarter_turns in 0..4 {
            for running in [false, true] {
                // Run every independent fixture so a first failure cannot hide
                // orientation-dependent regressions elsewhere in the matrix.
                if std::panic::catch_unwind(|| traverse(travel, degrees, quarter_turns, running))
                    .is_err()
                {
                    failures.push(format!("{degrees}/{quarter_turns}/{running}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{travel:?}: failed {}/16 cases: {failures:?}",
        failures.len()
    );
}

#[test]
fn backward_slope_travel_and_release_preserve_posture_and_progress() {
    check_directions(Travel::Backward);
}

#[test]
fn cross_slope_travel_and_release_preserve_posture_and_progress() {
    check_directions(Travel::CrossSlope);
}
