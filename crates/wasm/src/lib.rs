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
}
