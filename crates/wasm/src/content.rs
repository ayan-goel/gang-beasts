use brawler_engine::{CharacterDefinition, ENGINE_HASH, MapDefinition};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn skyline_content() -> Result<String, JsValue> {
    let map = MapDefinition::skyline().map_err(|err| JsValue::from_str(&err.to_string()))?;
    let fighter =
        CharacterDefinition::bundled().map_err(|err| JsValue::from_str(&err.to_string()))?;
    Ok(serde_json::json!({
        "engineHash": ENGINE_HASH, "mapHash": MapDefinition::skyline_hash(),
        "map": map, "fighter": fighter,
    })
    .to_string())
}
