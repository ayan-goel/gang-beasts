use brawler_engine::{PhysicsConfig, Simulation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PhysicsConfig::from_json(include_str!("../../../config/physics.json"))?;
    let mut simulation = Simulation::new(config)?;
    let mut positions = vec![simulation.fixture_position()];
    for _ in 0..600 {
        simulation.step();
        positions.push(simulation.fixture_position());
    }
    println!("{}", serde_json::to_string(&positions)?);
    Ok(())
}
