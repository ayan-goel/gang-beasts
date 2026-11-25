use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    movement: [f32; 11],
}

#[derive(Serialize)]
struct Case {
    running: bool,
    gap: u32,
    frames: Vec<Frame>,
}

fn frame(rig: &LocomotionRagdoll) -> Frame {
    Frame {
        bodies: rig
            .pose()
            .rig()
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect(),
        movement: rig.state().to_buffer(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    for running in [false, true] {
        for gap in [0, 1, 5] {
            let mut rig = LocomotionRagdoll::new(
                PhysicsConfig::default(),
                LocomotionConfig::bundled()?,
                [0.0; 3],
            )?;
            let mut frames = vec![frame(&rig)];
            for step in 0..420 {
                if step >= 60 {
                    let leg = (step - 60) / 45;
                    let tick = (step - 60) % 45;
                    let sign = if leg % 2 == 0 { 1.0 } else { -1.0 };
                    if leg > 0 && tick == 0 {
                        rig.set_input([0.0; 2], false)?;
                    }
                    let request = if leg > 0 && tick < gap { 0.0 } else { sign };
                    rig.set_input([request, 0.0], running && request != 0.0)?;
                }
                rig.step();
                frames.push(frame(&rig));
            }
            cases.push(Case {
                running,
                gap,
                frames,
            });
        }
    }
    serde_json::to_writer(std::io::stdout().lock(), &cases)?;
    Ok(())
}
