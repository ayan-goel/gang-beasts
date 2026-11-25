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

pub(super) fn force(joint: &GenericJoint, iterations: usize) -> f32 {
    // Rapier writes the final solver substep's impulse. Scale by its duration
    // to sample the transmitted force once per tick. New motors start at zero.
    let impulse = Vector::from(AXES.map(|a| joint.motors[a as usize].impulse));
    impulse.norm() * iterations as f32 / TICK_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PhysicsConfig, combat::GripBudget, physics::PhysicsWorld};

    fn suspend(mass: f32, count: usize, iterations: usize) -> (Vec<f32>, Option<usize>) {
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
                world
                    .impulse_joints
                    .insert(fixed, weight, spring(anchor, anchor, &tuning), true)
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
            if budget.advance(loads, &tuning.costs).contains(&true) {
                return (last, Some(tick));
            }
        }
        (last, None)
    }

    #[test]
    fn forty_kg_weight_settles_on_one_or_two_grips_across_solver_settings() {
        for iterations in [2, 4, 8] {
            for count in [1, 2] {
                let (loads, failed) = suspend(40., count, iterations);
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
    fn overweight_attachment_builds_sustained_force_before_breaking() {
        let (_, failed) = suspend(140., 1, 4);
        assert!(matches!(failed, Some(8..=100)), "break tick {failed:?}");
    }
}
