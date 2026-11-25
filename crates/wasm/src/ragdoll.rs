use crate::ragdoll_fixture::RagdollFixture;
use brawler_engine::{BodyPart, PhysicsConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LocalRagdoll {
    rig: RagdollFixture,
}

#[wasm_bindgen]
impl LocalRagdoll {
    #[wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        build(configuration, false)
    }

    pub fn standing(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        build(configuration, true)
    }

    pub fn tick(&self) -> u64 {
        self.rig.view().tick()
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
            .view()
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect()
    }

    pub fn joint_states(&self) -> Vec<f32> {
        self.rig
            .view()
            .joint_states()
            .into_iter()
            .flat_map(|joint| joint.to_buffer())
            .collect()
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.rig.collider_lines()
    }

    pub fn definition_json(&self) -> String {
        serde_json::to_string(self.rig.view().definition()).expect("validated character definition")
    }

    pub fn configuration_json(&self) -> String {
        serde_json::to_string(self.rig.view().configuration())
            .expect("validated physics configuration")
    }

    pub fn configure(&mut self, json: &str) -> Result<(), JsValue> {
        let enabled = self.rig.driven().map(|rig| rig.enabled());
        let mut replacement = build(json, enabled.is_some())?;
        if let Some(enabled) = enabled {
            replacement.rig.set_enabled(enabled).map_err(js_error)?;
        }
        *self = replacement;
        Ok(())
    }

    pub fn set_enabled(&mut self, value: JsValue) -> Result<(), JsValue> {
        // Validate before WASM coercion: the string "false" must never enable control.
        let enabled = value
            .as_bool()
            .ok_or_else(|| JsValue::from_str("enabled must be a boolean"))?;
        self.rig.set_enabled(enabled).map_err(js_error)
    }

    pub fn motor_configuration_json(&self) -> Option<String> {
        self.rig.driven().map(|rig| {
            serde_json::to_string(rig.motor_configuration()).expect("validated motor configuration")
        })
    }

    pub fn motor_efforts(&self) -> Vec<f32> {
        self.rig
            .driven()
            .and_then(|rig| rig.solver_motor_efforts())
            .map_or_else(Vec::new, |efforts| efforts.into_iter().flatten().collect())
    }

    pub fn motor_targets(&self) -> Vec<f32> {
        self.rig
            .driven()
            .and_then(|rig| rig.solver_motor_targets())
            .map_or_else(Vec::new, |targets| targets.into_iter().flatten().collect())
    }

    pub fn controller_state(&self) -> Vec<f32> {
        let Some(rig) = self.rig.driven() else {
            return Vec::new();
        };
        let balance = rig
            .balance_state()
            .expect("standing fixture has balance configured");
        vec![
            u8::from(rig.enabled()) as f32,
            u8::from(balance.feet[0]) as f32,
            u8::from(balance.feet[1]) as f32,
            balance.torque[0],
            balance.torque[1],
            balance.torque[2],
        ]
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

fn build(configuration: &str, standing: bool) -> Result<LocalRagdoll, JsValue> {
    let physics = PhysicsConfig::from_json(configuration).map_err(js_error)?;
    Ok(LocalRagdoll {
        rig: RagdollFixture::new(physics, standing).map_err(js_error)?,
    })
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
