use crate::CharacterError;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GripTuning {
    schema_version: u8,
    pub reach_gap_m: f32,
    pub stiffness_n_m: f32,
    pub damping_n_s_m: f32,
    pub maximum_force_n: f32,
}

impl GripTuning {
    pub fn bundled() -> Result<Self, CharacterError> {
        let config: Self = serde_json::from_str(include_str!("../../../../config/grip.json"))?;
        if config.schema_version != 1
            || !(0.0..=0.04).contains(&config.reach_gap_m)
            || !(1.0..=10000.).contains(&config.stiffness_n_m)
            || !(1.0..=500.).contains(&config.damping_n_s_m)
            || !(1.0..=3000.).contains(&config.maximum_force_n)
        {
            return Err(CharacterError::Invalid(
                "grip tuning outside supported range",
            ));
        }
        Ok(config)
    }
}
