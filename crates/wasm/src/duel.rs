use brawler_engine::{Duel, FighterId, Hand, PhysicsConfig};
use serde_json::json;
use wasm_bindgen::prelude::*;

fn index(value: JsValue) -> Result<usize, JsValue> {
    match value.as_f64() {
        Some(0.0) => Ok(0),
        Some(1.0) => Ok(1),
        _ => Err(JsValue::from_str("expected index 0 or 1")),
    }
}
fn fighter(value: JsValue) -> Result<FighterId, JsValue> {
    Ok([FighterId::First, FighterId::Second][index(value)?])
}
fn boolean(value: JsValue) -> Result<bool, JsValue> {
    value
        .as_bool()
        .ok_or_else(|| JsValue::from_str("expected a boolean"))
}
fn axis(value: JsValue) -> Result<f32, JsValue> {
    value
        .as_f64()
        .map(|n| n as f32)
        .filter(|n| n.is_finite())
        .ok_or_else(|| JsValue::from_str("expected a finite movement axis"))
}

#[wasm_bindgen]
pub struct LocalDuel {
    duel: Duel,
}

#[wasm_bindgen]
impl LocalDuel {
    #[wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<Self, JsValue> {
        let config = PhysicsConfig::from_json(configuration)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let duel = Duel::new(config).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { duel })
    }
    pub fn step(&mut self) {
        self.duel.step();
    }
    pub fn tick(&self) -> u64 {
        self.duel.tick()
    }
    pub fn reset(&mut self) {
        self.duel.reset();
    }
    pub fn definition_json(&self) -> String {
        serde_json::to_string(self.duel.definition()).expect("validated fighter definition")
    }
    pub fn body_states(&self, who: JsValue) -> Result<Vec<f32>, JsValue> {
        Ok(self
            .duel
            .body_states(fighter(who)?)
            .into_iter()
            .flat_map(|b| b.state.to_buffer())
            .collect())
    }
    pub fn set_input(
        &mut self,
        who: JsValue,
        x: JsValue,
        z: JsValue,
        run: JsValue,
    ) -> Result<(), JsValue> {
        self.duel
            .set_input(fighter(who)?, [axis(x)?, axis(z)?], boolean(run)?)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    pub fn set_lift(&mut self, who: JsValue, held: JsValue) -> Result<(), JsValue> {
        self.duel.set_lift(fighter(who)?, boolean(held)?);
        Ok(())
    }
    pub fn set_jump(&mut self, who: JsValue, held: JsValue) -> Result<(), JsValue> {
        self.duel.set_jump(fighter(who)?, boolean(held)?);
        Ok(())
    }
    pub fn set_hand(&mut self, who: JsValue, side: JsValue, held: JsValue) -> Result<(), JsValue> {
        self.duel.set_hand(
            fighter(who)?,
            [Hand::Left, Hand::Right][index(side)?],
            boolean(held)?,
        );
        Ok(())
    }
    pub fn condition_json(&self, who: JsValue) -> Result<String, JsValue> {
        let v = self.duel.vitals(fighter(who)?);
        Ok(json!({"phase":v.phase,"health":v.health,"knockouts":v.knockouts,"sinceTick":v.since_tick.to_string()}).to_string())
    }
    pub fn condition_events_json(&self) -> Option<String> {
        let changes = self.duel.consciousness_changes();
        if changes.is_empty() {
            return None;
        }
        let records: Vec<_> = changes
            .iter()
            .map(|(id, phase)| (*id as u8, phase))
            .collect();
        Some(serde_json::to_string(&records).expect("finite state transitions"))
    }
    pub fn stamina(&self, who: JsValue) -> Result<f32, JsValue> {
        Ok(self.duel.stamina(fighter(who)?))
    }
    pub fn grip_region(&self, who: JsValue, side: JsValue) -> Result<Option<u8>, JsValue> {
        let grip = self
            .duel
            .grip(fighter(who)?, [Hand::Left, Hand::Right][index(side)?]);
        Ok(grip.map(|g| g.region.index() as u8))
    }
    pub fn clear_jump(&mut self, who: JsValue) -> Result<(), JsValue> {
        self.duel.clear_jump(fighter(who)?);
        Ok(())
    }
    pub fn clear_hands(&mut self, who: JsValue) -> Result<(), JsValue> {
        self.duel.clear_hands(fighter(who)?);
        Ok(())
    }
    pub fn hits_json(&self) -> Option<String> {
        let hits = self.duel.hits();
        if hits.is_empty() {
            return None;
        }
        // Local IDs and completed ticks remain exact beyond the JS integer range.
        let records:Vec<_>=hits.iter().map(|h|json!({
            "tick":h.tick.to_string(), "attackId":h.attack_id.to_string(),
            "attacker":h.attacker as usize,"defender":h.defender as usize, "hand":h.hand as usize,
            "region":h.region,"damage":h.damage,"energyJ":h.energy_j,"closingSpeed":h.closing_speed,
        })).collect();
        Some(serde_json::to_string(&records).expect("finite physical hit records"))
    }
}

#[path = "skyline.rs"]
mod skyline;
