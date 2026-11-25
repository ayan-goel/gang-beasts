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
    pub costs: GripCosts,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GripCosts {
    pub maximum_stamina: f32,
    pub one_hand_per_s: f32,
    pub two_hand_per_s: f32,
    pub regeneration_per_s: f32,
    pub reference_force_n: f32,
    pub load_rate: f32,
    pub maximum_extra_per_s: f32,
    pub break_force_n: f32,
    pub break_after_ticks: u16,
    pub smoothing_s: f32,
}

impl GripCosts {
    fn valid(&self, ceiling: f32) -> bool {
        (1.0..=1000.).contains(&self.maximum_stamina)
            && (0.01..=100.).contains(&self.one_hand_per_s)
            && (self.one_hand_per_s..=100.).contains(&self.two_hand_per_s)
            && (0.01..=100.).contains(&self.regeneration_per_s)
            && (1.0..=ceiling).contains(&self.reference_force_n)
            && (0.0..=100.).contains(&self.load_rate)
            && (0.0..=100.).contains(&self.maximum_extra_per_s)
            && (self.reference_force_n..=ceiling).contains(&self.break_force_n)
            && (2..=120).contains(&self.break_after_ticks)
            && (0.001..=1.0).contains(&self.smoothing_s)
    }
}

impl GripTuning {
    pub fn bundled() -> Result<Self, CharacterError> {
        let config: Self = serde_json::from_str(include_str!("../../../../config/grip.json"))?;
        if config.schema_version != 1
            || !config.costs.valid(config.maximum_force_n)
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
