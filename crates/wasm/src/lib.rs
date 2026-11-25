use brawler_engine::{PhysicsConfig, Simulation};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct LocalSimulation {
    simulation: Simulation,
}

#[wasm_bindgen]
impl LocalSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<LocalSimulation, JsValue> {
        let config = PhysicsConfig::from_json(configuration)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        let simulation =
            Simulation::new(config).map_err(|error| JsValue::from_str(&error.to_string()))?;
        Ok(Self { simulation })
    }

    pub fn step(&mut self) {
        self.simulation.step();
    }

    pub fn tick(&self) -> u64 {
        self.simulation.tick()
    }

    pub fn position(&self) -> Vec<f32> {
        // Return owned data so a subsequent WASM allocation cannot invalidate
        // the renderer's view when linear memory grows.
        self.simulation.fixture_position().to_vec()
    }

    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    pub fn queue_impulse(&mut self, x: f32, y: f32, z: f32) -> Result<(), JsValue> {
        self.simulation
            .queue_fixture_impulse([x, y, z])
            .map_err(|error| JsValue::from_str(&error.to_string()))
    }

    pub fn fixture_state(&self) -> Vec<f32> {
        let state = self.simulation.fixture_state();
        let mut buffer = Vec::with_capacity(14);
        buffer.extend_from_slice(&state.position);
        buffer.extend_from_slice(&state.rotation);
        buffer.extend_from_slice(&state.linear_velocity);
        buffer.extend_from_slice(&state.angular_velocity);
        buffer.push(state.mass);
        buffer
    }

    pub fn collider_lines(&mut self) -> Vec<f32> {
        self.simulation.collider_lines()
    }
}
