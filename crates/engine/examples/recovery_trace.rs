use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    recovery: [f32; 3],
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled()?,
        [0.; 3],
    )?;
    let mut frames = Vec::with_capacity(480);
    for tick in 1..=480 {
        match tick {
            61 => {
                rig.set_enabled(false);
                rig.queue_impulse(BodyPart::Chest, [10., 0., 0.])?;
            }
            181 => rig.set_enabled(true),
            _ => {}
        }
        rig.step();
        let state = rig.state();
        let [force, moment] = state.recovery_loads;
        frames.push(Frame {
            bodies: rig
                .pose()
                .rig()
                .body_states()
                .into_iter()
                .flat_map(|b| b.state.to_buffer())
                .collect(),
            recovery: [state.recovery_strength, force, moment],
        });
    }
    serde_json::to_writer(std::io::stdout().lock(), &frames)?;
    Ok(())
}
