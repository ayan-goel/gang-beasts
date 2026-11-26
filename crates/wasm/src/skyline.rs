use super::{Duel, FighterId, LocalDuel, PhysicsConfig, fighter, index};
use brawler_engine::Hand;
use serde_json::json;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl LocalDuel {
    pub fn tour_match(configuration: &str, seed: u32, wins: u8) -> Result<Self, JsValue> {
        let physics = PhysicsConfig::from_json(configuration)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            duel: Duel::tour_match(physics, 2, (u64::from(seed) << 32) | 1, wins)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        })
    }
    pub fn skyline_match(configuration: &str) -> Result<Self, JsValue> {
        let physics = PhysicsConfig::from_json(configuration)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self {
            duel: Duel::skyline_match(physics).map_err(|e| JsValue::from_str(&e.to_string()))?,
        })
    }
    pub fn arena_grip(&self, who: JsValue, side: JsValue) -> Result<Option<u16>, JsValue> {
        Ok(self
            .duel
            .arena_grip(fighter(who)?, [Hand::Left, Hand::Right][index(side)?]))
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
        let map = self.duel.map_definition()?;
        let s = &map.schedule;
        let deadline = if map.tour.is_some() || tick >= u64::from(s.gust_tick) {
            s.end_tick
        } else {
            s.gust_tick
        };
        let visible = [FighterId::First, FighterId::Second]
            .map(|id| self.duel.vitals(id).phase != brawler_engine::Consciousness::Eliminated);
        Some(json!({"arenaTick":tick,"phase":m.phase,"round":m.round_number,"epoch":m.round_id.to_string(),"remaining":m.remaining_ticks,
            "scores":m.scores(),"winner":m.winner,"mapId":map.id,"wins":m.wins(),"breath":[self.duel.breath_remaining(FighterId::First),self.duel.breath_remaining(FighterId::Second)],"visible":visible,
            "timeLeft":u64::from(deadline).saturating_sub(tick),
            "hazard":if tick>=s.gust_tick.into(){"gust"}else if tick>=s.east_tick.into(){"gust_warning"}else if tick>=s.west_tick.into(){"east_warning"}else if tick>=s.warn_tick.into(){"west_warning"}else{"quiet"}
        }).to_string())
    }
}
