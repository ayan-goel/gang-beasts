use brawler_engine::{BodyPart, CharacterDefinition, PassiveRagdoll, PhysicsConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LocalRagdoll {
    rig: PassiveRagdoll,
}

#[wasm_bindgen]
impl LocalRagdoll {
    #[wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        let config = PhysicsConfig::from_json(configuration).map_err(js_error)?;
        let definition = CharacterDefinition::bundled().map_err(js_error)?;
        Ok(Self {
            rig: PassiveRagdoll::new(config, definition, [0.0, 2.0, 0.0]).map_err(js_error)?,
        })
    }

    pub fn tick(&self) -> u64 {
        self.rig.tick()
    }

    pub fn step(&mut self) {
        self.rig.step();
    }

    pub fn reset(&mut self) {
        self.rig.reset();
    }

    pub fn body_states(&self) -> Vec<f32> {
        // Owned, batched arrays remain valid after later WASM allocation/free.
        self.rig
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect()
    }

    pub fn joint_states(&self) -> Vec<f32> {
        self.rig
            .joint_states()
            .into_iter()
            .flat_map(|joint| joint.to_buffer())
            .collect()
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.rig.collider_lines()
    }

    pub fn definition_json(&self) -> String {
        serde_json::to_string(self.rig.definition()).expect("validated character definition")
    }

    pub fn configuration_json(&self) -> String {
        serde_json::to_string(self.rig.configuration()).expect("validated physics configuration")
    }

    pub fn configure(&mut self, json: &str) -> Result<(), JsValue> {
        let replacement = Self::new(json)?;
        *self = replacement;
        Ok(())
    }

    pub fn queue_impulse(&mut self, part: JsValue, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
        // Validate the JS number before integer conversion; narrow WASM integer
        // arguments would wrap/truncate invalid indices into a different limb.
        let part = part
            .as_f64()
            .ok_or_else(|| JsValue::from_str("body-part index must be a number"))?;
        if !part.is_finite() || part.fract() != 0.0 || !(0.0..15.0).contains(&part) {
            return Err(JsValue::from_str("invalid body-part index"));
        }
        self.rig
            .queue_impulse(BodyPart::ALL[part as usize], [x, y, z])
            .map_err(js_error)
    }
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
