use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig =
        LocomotionRagdoll::stairs(PhysicsConfig::default(), LocomotionConfig::bundled()?)?;
    let mut args = std::env::args().skip(1);
    let (settle, walk, end_tick) = match args.next().as_deref() {
        None => (60, 240, 210),
        Some("--landing") => {
            let settle = args
                .next()
                .map(|s| s.parse::<u16>())
                .transpose()?
                .unwrap_or(60);
            let walk = args
                .next()
                .map(|s| s.parse::<u16>())
                .transpose()?
                .unwrap_or(240);
            if walk > 660 {
                return Err("walking duration exceeds 660 ticks".into());
            }
            let (settle, walk) = (usize::from(settle), usize::from(walk));
            (settle, walk, settle + walk + 180)
        }
        Some(_) => {
            return Err(
                "expected --landing [settling ticks] [walking ticks] or no argument".into(),
            );
        }
    };
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let mut frames = Vec::with_capacity(end_tick + 1);
    for tick in 0..=end_tick {
        if tick != 0 {
            if walk > 0 && tick == settle + 1 {
                rig.set_input([0.0, -1.0], false)?;
            }
            if tick == settle + walk + 1 {
                rig.set_input([0.0; 2], false)?;
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
