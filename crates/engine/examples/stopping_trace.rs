use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    movement: [f32; 11],
    moment: [f32; 3],
}
#[derive(Serialize)]
struct Case {
    direction: [f32; 2],
    running: bool,
    duration: usize,
    reverse: usize,
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
        moment: rig.state().stopping_moment,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cases = Vec::new();
    for (direction, running, duration, reverse) in [
        ([1.0, 0.0], true, 45, 0),
        ([1.0, 0.0], true, 45, 5),
        ([-1.0, 0.0], true, 57, 0),
        ([0.0, -1.0], true, 51, 0),
        ([0.0, 1.0], true, 63, 0),
        ([1.0, 0.0], false, 72, 0),
        ([-1.0, 0.0], false, 54, 0),
        ([0.0, -1.0], false, 63, 0),
    ] {
        let mut rig = LocomotionRagdoll::new(
            PhysicsConfig::default(),
            LocomotionConfig::bundled()?,
            [0.0; 3],
        )?;
        let mut frames = vec![frame(&rig)];
        for step in 0..(60 + duration + reverse + 360) {
            if step == 60 {
                rig.set_input(direction, running)?;
            }
            if step == 60 + duration {
                rig.set_input(
                    if reverse == 0 {
                        [0.0; 2]
                    } else {
                        direction.map(|v| -v)
                    },
                    reverse > 0 && running,
                )?;
            }
            if step == 60 + duration + reverse {
                rig.set_input([0.0; 2], false)?;
            }
            rig.step();
            frames.push(frame(&rig));
        }
        cases.push(Case {
            direction,
            running,
            duration,
            reverse,
            frames,
        });
    }
    serde_json::to_writer(std::io::stdout().lock(), &cases)?;
    Ok(())
}
