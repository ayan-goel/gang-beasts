use brawler_engine::{FixtureSession, PhysicsConfig};
use wasm_bindgen::prelude::*;
mod ragdoll;
pub use ragdoll::LocalRagdoll;

#[wasm_bindgen]
pub struct LocalSimulation {
    session: FixtureSession,
}

#[wasm_bindgen]
impl LocalSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<LocalSimulation, JsValue> {
        let config = PhysicsConfig::from_json(configuration)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let session =
            FixtureSession::new(config).map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { session })
    }

    pub fn step(&mut self) -> bool {
        self.session.step()
    }

    pub fn tick(&self) -> u64 {
        self.session.simulation().tick()
    }

    pub fn position(&self) -> Vec<f32> {
        // Return owned data so a subsequent WASM allocation cannot invalidate
        // the renderer's view when linear memory grows.
        self.session.simulation().fixture_position().to_vec()
    }

    pub fn reset(&mut self) {
        self.session.reset();
    }

    pub fn queue_impulse(&mut self, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
        self.session
            .queue_impulse([x, y, z])
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn fixture_state(&self) -> Vec<f32> {
        self.session
            .simulation()
            .fixture_state()
            .to_buffer()
            .to_vec()
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.session.collider_lines()
    }

    pub fn configure(&mut self, json: &str) -> Result<(), JsValue> {
        self.session
            .configure(json)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn configuration_json(&self) -> String {
        self.session.configuration_json()
    }

    pub fn import_recording(&mut self, json: &str) -> Result<(), JsValue> {
        self.session
            .import_recording(json)
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn export_recording(&self) -> Result<String, JsValue> {
        self.session
            .export_recording()
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn replay_end(&self) -> Option<u32> {
        self.session.replay_end()
    }

    pub fn recording_available(&self) -> bool {
        self.session.recording_available()
    }
}
