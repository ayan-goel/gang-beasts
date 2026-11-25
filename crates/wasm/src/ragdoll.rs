use crate::ragdoll_fixture::{RagdollFixture, RigKind};
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
        build(configuration, RigKind::Passive)
    }

    pub fn standing(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        build(configuration, RigKind::Standing)
    }

    pub fn locomotion(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        build(configuration, RigKind::Locomotion)
    }
    pub fn stairs(configuration: &str) -> Result<LocalRagdoll, JsValue> {
        build(configuration, RigKind::Stairs)
    }

    pub fn terrain_boxes(&self) -> Vec<f32> {
        self.rig
            .view()
            .terrain_boxes()
            .into_iter()
            .flatten()
            .collect()
    }

    pub fn set_movement(
        &mut self,
        x: JsValue,
        z: JsValue,
        running: JsValue,
    ) -> Result<(), JsValue> {
        // Reject JS coercions and f64 values that cannot fit a finite engine f32
        // before replacing persistent movement intent.
        let number = |value: JsValue| -> Result<f32, JsValue> {
            let value = value
                .as_f64()
                .ok_or_else(|| js_error("movement axes must be numbers"))?;
            if !value.is_finite() || value.abs() > f32::MAX as f64 {
                return Err(js_error("movement axis outside finite f32 range"));
            }
            Ok(value as f32)
        };
        let direction = [number(x)?, number(z)?];
        let running = running
            .as_bool()
            .ok_or_else(|| js_error("running must be a boolean"))?;
        self.rig
            .walking_mut()
            .map_err(js_error)?
            .set_input(direction, running)
            .map_err(js_error)
    }
    pub fn set_jump(&mut self, held: JsValue) -> Result<(), JsValue> {
        let held = held
            .as_bool()
            .ok_or_else(|| js_error("jump state must be a boolean"))?;
        self.rig.walking_mut().map_err(js_error)?.set_jump(held);
        Ok(())
    }

    pub fn clear_jump_input(&mut self) -> Result<(), JsValue> {
        self.rig.walking_mut().map_err(js_error)?.clear_jump_input();
        Ok(())
    }

    pub fn movement_state(&self) -> Vec<f32> {
        self.rig
            .walking()
            .map_or_else(Vec::new, |rig| rig.state().to_buffer().to_vec())
    }
    pub fn recovery_state(&self) -> Vec<f32> {
        self.rig.walking().map_or_else(Vec::new, |rig| {
            let state = rig.state();
            let [force, moment] = state.recovery_loads;
            vec![state.recovery_strength, force, moment]
        })
    }
    pub fn stopping_moment(&self) -> Vec<f32> {
        self.rig
            .walking()
            .map_or_else(Vec::new, |rig| rig.state().stopping_moment.to_vec())
    }
    pub fn support_moment(&self) -> Vec<f32> {
        self.rig
            .walking()
            .map_or_else(Vec::new, |rig| rig.state().support_moment.to_vec())
    }
    pub fn pose_targets(&self) -> Vec<f32> {
        if let Some(rig) = self.rig.walking() {
            return rig.pose_targets().into_iter().flatten().collect();
        }
        if self.rig.driven().is_some() {
            return [[0.0, 0.0, 0.0, 1.0]; 14].into_iter().flatten().collect();
        }
        Vec::new()
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
        let mut replacement = build(json, self.rig.kind())?;
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

fn build(configuration: &str, kind: RigKind) -> Result<LocalRagdoll, JsValue> {
    let physics = PhysicsConfig::from_json(configuration).map_err(js_error)?;
    Ok(LocalRagdoll {
        rig: RagdollFixture::new(physics, kind).map_err(js_error)?,
    })
}

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}
