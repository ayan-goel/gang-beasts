use brawler_engine::{BodyPart, CharacterDefinition, PassiveRagdoll, PhysicsConfig};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    joints: Vec<f32>,
}

fn frame(rig: &PassiveRagdoll) -> Frame {
    Frame {
        bodies: rig
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect(),
        joints: rig
            .joint_states()
            .into_iter()
            .flat_map(|joint| joint.to_buffer())
            .collect(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let physics = PhysicsConfig::from_json(include_str!("../../../config/physics.json"))?;
    let mut rig = PassiveRagdoll::new(physics, CharacterDefinition::bundled()?, [0.0, 2.0, 0.0])?;
    let mut frames = vec![frame(&rig)];
    for tick in 1..=240 {
        if tick == 1 {
            rig.queue_impulse(BodyPart::Head, [3.0, 0.0, 1.0])?;
        }
        if tick == 90 {
            rig.queue_impulse(BodyPart::LeftHand, [0.0, 2.0, 1.0])?;
        }
        rig.step();
        frames.push(frame(&rig));
    }
    serde_json::to_writer(std::io::stdout().lock(), &frames)?;
    Ok(())
}
