use crate::{TICK_SECONDS, combat::GripTuning};
use rapier3d::prelude::*;

const AXES: [JointAxis; 3] = [JointAxis::LinX, JointAxis::LinY, JointAxis::LinZ];

pub(super) fn spring(hand: Point<Real>, target: Point<Real>, tuning: &GripTuning) -> GenericJoint {
    let mut joint = GenericJointBuilder::new(JointAxesMask::empty())
        .local_anchor1(hand)
        .local_anchor2(target);
    // Per-axis ceilings bound the resultant. Free angular axes let held limbs twist in closed loops.
    for axis in AXES {
        joint = joint
            .motor_model(axis, MotorModel::ForceBased)
            .motor_position(axis, 0., tuning.stiffness_n_m, tuning.damping_n_s_m)
            .motor_max_force(axis, tuning.maximum_force_n / 3.0_f32.sqrt());
    }
    joint.build()
}

pub(super) fn limit_slip(joint: &mut GenericJoint, slack: f32) {
    // A palm can roll against the edge, but an attached wrist cannot stretch metres away from it.
    for axis in AXES {
        joint.set_limits(axis, [-slack, slack]);
    }
}

pub(super) fn force(joint: &GenericJoint, iterations: usize) -> f32 {
    // Rapier writes the final solver substep's impulse. Scale by its duration
    // to sample the transmitted force once per tick. New motors start at zero.
    let impulse = Vector::from(
        AXES.map(|a| joint.motors[a as usize].impulse + joint.limits[a as usize].impulse),
    );
    impulse.norm() * iterations as f32 / TICK_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhysicsConfig, combat::GripBudget, physics::PhysicsWorld};

    fn suspend(
        mass: f32,
        count: usize,
        iterations: usize,
        ledge: bool,
    ) -> (Vec<f32>, Option<usize>) {
        let config = PhysicsConfig {
            solver_iterations: iterations,
            ..PhysicsConfig::default()
        };
        let mut world = PhysicsWorld::new(&config);
        let fixed = world.bodies.insert(RigidBodyBuilder::fixed());
        let weight = world
            .bodies
            .insert(RigidBodyBuilder::dynamic().can_sleep(false));
        world.colliders.insert_with_parent(
            ColliderBuilder::ball(0.2).mass(mass),
            weight,
            &mut world.bodies,
        );
        let tuning = GripTuning::bundled().unwrap();
        let mut budget = GripBudget::new(&tuning.costs);
        let joints: Vec<_> = (0..count)
            .map(|i| {
                let x = (i as f32 - (count - 1) as f32 * 0.5) * 0.3;
                let anchor = point![x, 0., 0.];
                let mut joint = spring(anchor, anchor, &tuning);
                if ledge {
                    limit_slip(&mut joint, tuning.reach_gap_m);
                }
                world.impulse_joints.insert(fixed, weight, joint, true)
            })
            .collect();
        let mut last = Vec::new();
        for tick in 1..=600 {
            world.step();
            last = joints
                .iter()
                .map(|&j| force(&world.impulse_joints.get(j).unwrap().data, iterations))
                .collect();
            let loads = [last.first().copied(), last.get(1).copied()];
            if budget
                .advance(loads, &tuning.costs, [ledge; 2])
                .contains(&true)
            {
                return (last, Some(tick));
            }
        }
        (last, None)
    }

    #[test]
    fn forty_kg_weight_settles_on_one_or_two_grips_across_solver_settings() {
        for iterations in [2, 4, 8] {
            for count in [1, 2] {
                let (loads, failed) = suspend(40., count, iterations, false);
                assert_eq!(failed, None);
                for load in loads {
                    let expected = 40. * 9.81 / count as f32;
                    assert!(
                        (load - expected).abs() < 4.,
                        "sample {load} N, expected {expected} N"
                    );
                }
            }
        }
    }

    #[test]
    fn roof_lips_support_one_hanging_fighter_and_release_a_sustained_double_load() {
        let (normal, broke) = suspend(40., 1, 4, true);
        assert_eq!(broke, None);
        assert!((normal[0] - 392.4).abs() < 4.);
        let (_, broke) = suspend(80., 1, 4, true);
        let window = usize::from(GripTuning::bundled().unwrap().costs.ledge_break_after_ticks);
        assert!(
            broke.is_some_and(|tick| (window..=90).contains(&tick)),
            "break tick {broke:?}"
        );
    }

    #[test]
    fn overweight_attachment_builds_sustained_force_before_breaking() {
        let (_, failed) = suspend(140., 1, 4, false);
        assert!(matches!(failed, Some(8..=100)), "break tick {failed:?}");
    }
}
