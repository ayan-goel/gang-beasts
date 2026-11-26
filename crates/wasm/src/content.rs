use brawler_engine::{CharacterDefinition, ENGINE_HASH, MapDefinition};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn skyline_content() -> Result<String, JsValue> {
    let map = MapDefinition::skyline().map_err(|err| JsValue::from_str(&err.to_string()))?;
    let fighter =
        CharacterDefinition::bundled().map_err(|err| JsValue::from_str(&err.to_string()))?;
    Ok(serde_json::json!({
        "engineHash": ENGINE_HASH, "mapHash": MapDefinition::tour_hash(),
        "map": map, "fighter": fighter,
    })
    .to_string())
}

#[wasm_bindgen]
pub fn tour_map(seed: u32, round: u32) -> Result<String, JsValue> {
    let map =
        MapDefinition::tour_round(seed, round).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(serde_json::to_string(&map).expect("validated map"))
}
