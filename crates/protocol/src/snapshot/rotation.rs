pub(super) fn pack(rotation: [f32; 4]) -> [u8; 7] {
    let mut largest = 0;
    for i in 1..4 {
        if rotation[i].abs() > rotation[largest].abs() {
            largest = i;
        }
    }
    // A quaternion and its negation describe the same orientation. Making the omitted
    // component positive lets the receiver recover its sign without an extra field.
    let norm = rotation
        .iter()
        .map(|v| f64::from(*v).powi(2))
        .sum::<f64>()
        .sqrt();
    let scale = 32767. * f64::from(rotation[largest].signum()) / norm;
    let mut bytes = [0; 7];
    bytes[0] = largest as u8;
    let mut offset = 1;
    for (i, value) in rotation.into_iter().enumerate() {
        if i != largest {
            let code = (f64::from(value) * scale).round() as i16;
            bytes[offset..offset + 2].copy_from_slice(&code.to_le_bytes());
            offset += 2;
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantized_rotations_stay_within_a_hundredth_of_a_degree() {
        let mut state = 1_u32;
        for _ in 0..50_000 {
            let source: [f64; 4] = std::array::from_fn(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                f64::from(state as i32) / f64::from(i32::MAX)
            });
            let norm = source.iter().map(|v| v * v).sum::<f64>().sqrt();
            let unit = source.map(|v| v / norm);
            let bytes = pack(unit.map(|v| (v * 1.0004) as f32));
            let omitted = bytes[0] as usize;
            let mut codes = bytes[1..].chunks_exact(2);
            let mut restored = [0.; 4];
            for (i, v) in restored.iter_mut().enumerate() {
                if i != omitted {
                    let code = i16::from_le_bytes(codes.next().unwrap().try_into().unwrap());
                    *v = f64::from(code) / 32767.;
                }
            }
            restored[omitted] = (1. - restored.iter().map(|v| v * v).sum::<f64>()).sqrt();
            assert!(
                restored
                    .iter()
                    .all(|v| v.abs() <= restored[omitted] + 3. / 32767.)
            );
            let dot = unit.iter().zip(restored).map(|(a, b)| a * b).sum::<f64>();
            assert!((2. * dot.abs().min(1.).acos()).to_degrees() < 0.01);
        }
    }
}
