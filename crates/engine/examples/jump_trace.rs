use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled()?,
        [0.; 3],
    )?;
    let mut frames = Vec::with_capacity(220);
    for tick in 1..=220 {
        match tick {
            61 | 160 => rig.set_jump(true),
            70 => rig.set_input([1., 0.], false)?,
            90 => rig.set_input([0.; 2], false)?,
            155 => rig.set_jump(false),
            _ => {}
        }
        rig.step();
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
