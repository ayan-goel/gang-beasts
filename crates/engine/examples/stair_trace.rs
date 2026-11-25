use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig =
        LocomotionRagdoll::stairs(PhysicsConfig::default(), LocomotionConfig::bundled()?)?;
    let mut frames = Vec::with_capacity(211);
    for tick in 0..=210 {
        if tick != 0 {
            if tick == 61 {
                rig.set_input([0.0, -1.0], false)?;
            }
            rig.step();
        }
        let bodies: Vec<_> = rig
            .pose()
            .rig()
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect();
        frames.push(bodies);
    }
    serde_json::to_writer(std::io::stdout().lock(), &frames)?;
    Ok(())
}
