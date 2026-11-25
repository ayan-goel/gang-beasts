use crate::{CharacterError, TICK_SECONDS};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HealthRules {
    schema_version: u8,
    max_health: f32,
    regen_delay_ticks: u16,
    regen_points_per_s: f32,
    first_ko_ticks: u16,
    extra_ko_ticks: u16,
    max_ko_ticks: u16,
    wake_health: f32,
    wake_ticks: u16,
}

impl HealthRules {
    pub fn bundled() -> Result<Self, CharacterError> {
        let rules: Self =
            serde_json::from_str(include_str!("../../../../config/consciousness.json"))?;
        if rules.schema_version != 1
            || !(1.0..=1000.).contains(&rules.max_health)
            || !(0.1..=rules.max_health).contains(&rules.wake_health)
            || !(0.0..=100.).contains(&rules.regen_points_per_s)
            || !(1..=3600).contains(&rules.regen_delay_ticks)
            || !(1..=3600).contains(&rules.max_ko_ticks)
            || !(1..=rules.max_ko_ticks).contains(&rules.first_ko_ticks)
            || rules.extra_ko_ticks > rules.max_ko_ticks
            || !(1..=180).contains(&rules.wake_ticks)
        {
            return Err(CharacterError::Invalid(
                "consciousness tuning outside supported range",
            ));
        }
        Ok(rules)
    }

    fn ko_ticks(&self, count: u32) -> u64 {
        (u64::from(self.first_ko_ticks)
            + u64::from(count.saturating_sub(1)) * u64::from(self.extra_ko_ticks))
        .min(self.max_ko_ticks.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Consciousness {
    Active,
    KnockedOut,
    Recovering,
    Eliminated,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vitals {
    pub health: f32,
    pub phase: Consciousness,
    pub knockouts: u32,
    pub since_tick: u64,
    last_damage: u64,
}

impl Vitals {
    pub(crate) fn new(rules: &HealthRules) -> Self {
        Self {
            health: rules.max_health,
            phase: Consciousness::Active,
            knockouts: 0,
            since_tick: 0,
            last_damage: 0,
        }
    }

    pub(crate) fn waking_effort(self, tick: u64, rules: &HealthRules) -> f32 {
        (tick.saturating_sub(self.since_tick) as f32 / f32::from(rules.wake_ticks)).min(1.)
    }

    pub(crate) fn advance(
        &mut self,
        tick: u64,
        damage: f32,
        rules: &HealthRules,
    ) -> Option<Consciousness> {
        let elapsed = tick - self.since_tick;
        let next = match self.phase {
            Consciousness::Active if damage > 0. => {
                self.last_damage = tick;
                self.health = (self.health - damage).max(0.);
                if self.health > 0. {
                    return None;
                }
                self.knockouts = self.knockouts.saturating_add(1);
                Consciousness::KnockedOut
            }
            Consciousness::Active => {
                if tick - self.last_damage >= rules.regen_delay_ticks.into() {
                    self.health = (self.health + rules.regen_points_per_s * TICK_SECONDS)
                        .min(rules.max_health);
                }
                return None;
            }
            // Contacts during the down and wake phases cannot restart either deadline.
            Consciousness::KnockedOut if elapsed >= rules.ko_ticks(self.knockouts) => {
                self.health = rules.wake_health;
                Consciousness::Recovering
            }
            Consciousness::Recovering if elapsed >= rules.wake_ticks.into() => {
                Consciousness::Active
            }
            _ => return None,
        };
        self.phase = next;
        self.since_tick = tick;
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_knockouts_have_bounded_distinct_deadlines_even_under_continuous_hits() {
        let rules = HealthRules::bundled().unwrap();
        let mut vitals = Vitals::new(&rules);
        let mut tick = 10;
        for n in 1..=13 {
            assert_eq!(
                vitals.advance(tick, 100., &rules),
                Some(Consciousness::KnockedOut)
            );
            assert_eq!(vitals.knockouts, n);
            let wake = tick + (180 + 60 * u64::from(n - 1)).min(720);
            for t in tick + 1..wake {
                assert_eq!(vitals.advance(t, 35., &rules), None);
            }
            assert_eq!(vitals.health, 0.);
            assert_eq!(
                vitals.advance(wake, 35., &rules),
                Some(Consciousness::Recovering)
            );
            assert_eq!(vitals.health, 60.);
            for t in wake + 1..wake + 30 {
                assert_eq!(vitals.advance(t, 35., &rules), None);
                assert_eq!(vitals.waking_effort(t, &rules), (t - wake) as f32 / 30.);
            }
            assert_eq!(
                vitals.advance(wake + 30, 35., &rules),
                Some(Consciousness::Active)
            );
            tick = wake + 31;
        }
    }
}
