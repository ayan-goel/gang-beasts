use brawler_engine::FixtureSession;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut json = String::new();
    io::stdin().take(262_145).read_to_string(&mut json)?;
    let mut session = FixtureSession::from_recording(&json)?;
    let mut trace = vec![session.simulation().fixture_state().to_buffer()];
    while session.step() {
        trace.push(session.simulation().fixture_state().to_buffer());
    }
    println!("{}", serde_json::to_string(&trace)?);
    Ok(())
}
