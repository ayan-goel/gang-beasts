use super::{Duel, FighterId, LocalDuel, PhysicsConfig};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl LocalDuel {
    pub fn skyline_match(configuration: &str) -> Result<Self, JsValue> {
        let physics = PhysicsConfig::from_json(configuration)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            duel: Duel::skyline_match(physics).map_err(|e| JsValue::from_str(&e.to_string()))?,
        })
    }
    pub fn map_json(&self) -> Option<String> {
        self.duel
            .map_definition()
            .map(|m| serde_json::to_string(m).expect("validated map"))
    }
    pub fn arena_poses(&self) -> Vec<f32> {
        self.duel
            .arena_states()
            .into_iter()
            .flat_map(|s| {
                let [x, y, z] = s.position;
                let [qx, qy, qz, qw] = s.rotation;
                [x, y, z, qx, qy, qz, qw, if s.visible { 1. } else { 0. }]
            })
            .collect()
    }
    pub fn match_json(&self) -> Option<String> {
        let m = self.duel.match_state()?;
        let tick = self.duel.tick();
        let s = &self.duel.map_definition()?.schedule;
        let visible = [FighterId::First, FighterId::Second]
            .map(|id| self.duel.vitals(id).phase != brawler_engine::Consciousness::Eliminated);
        Some(json!({"phase":m.phase,"round":m.round_number,"epoch":m.round_id.to_string(),"remaining":m.remaining_ticks,
            "scores":m.scores(),"winner":m.winner,"visible":visible,
            "timeLeft":u64::from(if tick<u64::from(s.gust_tick){s.gust_tick}else{s.end_tick}).saturating_sub(tick),
            "hazard":if tick>=s.gust_tick.into(){"gust"}else if tick>=s.east_tick.into(){"gust_warning"}else if tick>=s.west_tick.into(){"east_warning"}else if tick>=s.warn_tick.into(){"west_warning"}else{"quiet"}
        }).to_string())
    }
}
