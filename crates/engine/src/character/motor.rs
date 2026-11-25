use super::JointMotorConfig;
use rapier3d::prelude::*;

pub(super) fn local_torque(
    current: Rotation<Real>,
    target: Rotation<Real>,
    relative_velocity: Vector<Real>,
    free_axes: [bool; 3],
    config: &JointMotorConfig,
) -> Vector<Real> {
    let mut error = (target * current.inverse()).into_inner();
    error.normalize_mut();
    // Canonicalize the hemisphere, including exactly 180 degrees, so q and -q
    // choose the same shortest arc instead of alternating the requested torque.
    if [error.w, error.i, error.j, error.k]
        .into_iter()
        .find(|component| *component != 0.0)
        .is_some_and(|component| component < 0.0)
    {
        error = -error;
    }
    let angle_error = Rotation::new_unchecked(error).scaled_axis();
    let mut torque =
        angle_error * config.stiffness_nm_per_rad - relative_velocity * config.damping_nm_s_per_rad;
    for axis in 0..3 {
        if !free_axes[axis] {
            torque[axis] = 0.0;
        }
    }
    // A joint has one vector budget; independently clamping three axes would
    // allow sqrt(3) times the advertised effort on diagonal errors.
    let norm = torque.cast::<f64>().norm();
    if !norm.is_finite() {
        return Vector::zeros();
    }
    if norm > f64::from(config.max_torque_nm) {
        torque *= (f64::from(config.max_torque_nm) / norm) as f32;
    }
    torque
}
