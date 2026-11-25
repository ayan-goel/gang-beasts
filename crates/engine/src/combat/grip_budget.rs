use super::grip_tuning::GripCosts;
use crate::TICK_SECONDS;

#[derive(Default)]
struct LoadHistory {
    force_n: f32,
    overload_ticks: u16,
}

pub(crate) struct GripBudget {
    pub stamina: f32,
    hands: [LoadHistory; 2],
}

impl GripBudget {
    pub fn new(costs: &GripCosts) -> Self {
        Self {
            stamina: costs.maximum_stamina,
            hands: Default::default(),
        }
    }

    pub fn force(&self, side: usize) -> f32 {
        self.hands[side].force_n
    }

    pub fn advance(
        &mut self,
        loads: [Option<f32>; 2],
        costs: &GripCosts,
        ledges: [bool; 2],
    ) -> [bool; 2] {
        let count = loads.iter().flatten().count();
        let mut extra = 0.;
        let mut broken = [false; 2];
        for (side, (history, load)) in self.hands.iter_mut().zip(loads).enumerate() {
            // A released hand unloads gradually. Reattaching cannot erase recent sustained strain.
            let alpha = TICK_SECONDS / (costs.smoothing_s + TICK_SECONDS);
            history.force_n += alpha * (load.unwrap_or(0.) - history.force_n);
            let threshold = if ledges[side] {
                costs.ledge_break_force_n
            } else {
                costs.break_force_n
            };
            if load.is_some() && history.force_n >= threshold {
                history.overload_ticks = history.overload_ticks.saturating_add(1);
            } else {
                history.overload_ticks = history.overload_ticks.saturating_sub(1);
            }
            if load.is_some() {
                extra += costs.load_rate * (history.force_n / costs.reference_force_n - 1.).max(0.);
                // A fixed lip arrests the entire falling rig. A longer overload window
                // admits the catch impulse while still breaking a sustained overloaded hold.
                let window = if ledges[side] {
                    costs.ledge_break_after_ticks
                } else {
                    costs.break_after_ticks
                };
                broken[side] = history.overload_ticks >= window;
            }
        }
        let rate = match count {
            0 => costs.regeneration_per_s,
            1 => -costs.one_hand_per_s - extra.min(costs.maximum_extra_per_s),
            _ => -costs.two_hand_per_s - extra.min(costs.maximum_extra_per_s),
        };
        self.stamina = (self.stamina + rate * TICK_SECONDS).clamp(0., costs.maximum_stamina);
        if self.stamina == 0. {
            return loads.map(|load| load.is_some());
        }
        broken
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::GripTuning;

    #[test]
    fn base_drain_is_per_owner_and_total_load_cost_is_bounded() {
        let costs = GripTuning::bundled().unwrap().costs;
        for (loads, expected) in [([Some(0.), None], 96.), ([Some(0.); 2], 93.)] {
            let mut budget = GripBudget::new(&costs);
            for _ in 0..60 {
                assert_eq!(budget.advance(loads, &costs, [false; 2]), [false; 2]);
            }
            assert!((budget.stamina - expected).abs() < 0.001);
            for _ in 0..60 {
                budget.advance([None; 2], &costs, [false; 2]);
            }
            assert_eq!(budget.stamina, 100.);
        }
        let mut budget = GripBudget::new(&costs);
        for _ in 0..60 {
            budget.advance([Some(1200.); 2], &costs, [false; 2]);
        }
        assert!(budget.stamina >= 100. - 7. - costs.maximum_extra_per_s - 0.001);
        assert!(budget.stamina < 93.);
    }

    #[test]
    fn spikes_decay_and_sustained_overload_breaks_only_the_loaded_hand() {
        let costs = GripTuning::bundled().unwrap().costs;
        let mut budget = GripBudget::new(&costs);
        let high = [Some(1200.), Some(390.)];
        assert_eq!(budget.advance(high, &costs, [false; 2]), [false; 2]);
        for _ in 0..16 {
            assert_eq!(
                budget.advance([Some(0.); 2], &costs, [false; 2]),
                [false; 2]
            );
        }
        for _ in 0..60 {
            if budget.advance(high, &costs, [false; 2])[0] {
                assert!(!budget.advance([None, Some(390.)], &costs, [false; 2])[1]);
                assert_eq!(budget.advance(high, &costs, [false; 2]), [true, false]);
                return;
            }
        }
        panic!("sustained overload did not release the loaded hand");
    }
}
