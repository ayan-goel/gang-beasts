use super::CharacterDefinition;

pub(super) fn toward_bind(
    previous: [[f32; 3]; 14],
    definition: &CharacterDefinition,
    duration_s: f32,
) -> [[f32; 3]; 14] {
    // Bound local-angle changes by the anatomical range over the release time.
    // Motor strength cannot set this rate: a relaxed slope still has ankle targets.
    std::array::from_fn(|joint| {
        std::array::from_fn(|axis| {
            let [low, high] = definition.joints[joint].angular_limits[axis];
            let step = (high - low) * crate::TICK_SECONDS / duration_s;
            let value = previous[joint][axis];
            value - value.clamp(-step, step)
        })
    })
}
