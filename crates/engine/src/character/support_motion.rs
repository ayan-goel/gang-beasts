use rapier3d::prelude::*;

#[derive(Default)]
pub(super) struct SupportMotion {
    reference_velocity: Vector<Real>,
}

impl SupportMotion {
    pub fn advance(
        &mut self,
        mut surface_velocity: Vector<Real>,
        predicted_velocity: Vector<Real>,
        maximum_velocity_change: Real,
    ) -> Vector<Real> {
        surface_velocity.y = 0.0;
        let change =
            (surface_velocity - self.reference_velocity).cap_magnitude(maximum_velocity_change);
        self.reference_velocity += change;
        if change.norm_squared() == 0.0 {
            return Vector::zeros();
        }
        // Track deck acceleration without stiffening damping against impacts.
        // Contacts and idle damping may already supply some or all of this change.
        let missing = ((self.reference_velocity - predicted_velocity).dot(&change)
            / change.norm_squared())
        .clamp(0.0, 1.0);
        change * missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_start_and_stop_consume_one_bounded_velocity_change() {
        let mut support = SupportMotion::default();
        let mut velocity = Vector::zeros();
        for target in [vector![0.8, 0.0, 0.0], Vector::zeros()] {
            for _ in 0..8 {
                let change = support.advance(target, velocity, 0.2);
                assert!(change.norm() <= 0.20001);
                velocity += change;
            }
            assert!((velocity - target).norm() < 1e-6);
        }
    }

    #[test]
    fn contact_transfer_is_not_duplicated_and_relative_impacts_are_not_cancelled() {
        let mut support = SupportMotion::default();
        let surface = vector![0.8, 0.0, 0.0];
        for _ in 0..8 {
            assert_eq!(support.advance(surface, surface, 0.2), Vector::zeros());
        }
        assert_eq!(
            support.advance(surface, vector![-1.0, 0.0, 2.0], 0.2),
            Vector::zeros()
        );
        assert_eq!(
            support.advance(Vector::zeros(), Vector::zeros(), 0.2),
            Vector::zeros()
        );
    }

    #[test]
    fn contacts_are_subtracted_from_each_bounded_start_and_stop_increment() {
        let mut support = SupportMotion::default();
        let change = support.advance(vector![0.8, 0.0, 0.0], vector![0.15, 0.0, 0.0], 0.2);
        assert!(
            (change.x - 0.05).abs() < 1e-6,
            "start correction {change:?}"
        );
        for _ in 0..3 {
            support.advance(vector![0.8, 0.0, 0.0], vector![0.8, 0.0, 0.0], 0.2);
        }
        let change = support.advance(Vector::zeros(), vector![0.65, 0.0, 0.0], 0.2);
        assert!((change.x + 0.05).abs() < 1e-6, "stop correction {change:?}");
    }

    #[test]
    fn partial_contact_transfer_only_supplies_the_remaining_gap_without_vertical_lift() {
        let mut support = SupportMotion::default();
        for _ in 0..3 {
            support.advance(vector![0.8, 0.0, 0.0], vector![0.8, 0.0, 0.0], 0.2);
        }
        let change = support.advance(vector![0.8, 3.0, 0.0], vector![0.75, -1.0, 2.0], 0.2);
        assert!((change.x - 0.05).abs() < 1e-6);
        assert_eq!(change.y, 0.0);
        assert_eq!(change.z, 0.0);
    }
}
