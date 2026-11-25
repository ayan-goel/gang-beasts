use brawler_engine::{
    BalanceConfig, BodyPart, CharacterDefinition, MotorConfig, PhysicsConfig, PoseRagdoll,
};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    joints: Vec<f32>,
    efforts: Vec<f32>,
    targets: Vec<f32>,
    control: Vec<f32>,
}

fn frame(rig: &PoseRagdoll) -> Frame {
    let balance = rig.balance_state().unwrap();
    Frame {
        bodies: rig
            .rig()
            .body_states()
            .into_iter()
            .flat_map(|body| body.state.to_buffer())
            .collect(),
        joints: rig
            .rig()
            .joint_states()
            .into_iter()
            .flat_map(|joint| joint.to_buffer())
            .collect(),
        efforts: rig
            .solver_motor_efforts()
            .unwrap()
            .into_iter()
            .flatten()
            .collect(),
        targets: rig
            .solver_motor_targets()
            .unwrap()
            .into_iter()
            .flatten()
            .collect(),
        control: vec![
            u8::from(rig.enabled()) as f32,
            u8::from(balance.feet[0]) as f32,
            u8::from(balance.feet[1]) as f32,
            balance.torque[0],
            balance.torque[1],
            balance.torque[2],
        ],
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig = PoseRagdoll::new(
        PhysicsConfig::from_json(include_str!("../../../config/physics.json"))?,
        CharacterDefinition::bundled()?,
        MotorConfig::standing()?,
        [0.0; 3],
    )?;
    rig.configure_balance(Some(BalanceConfig::bundled()?))?;
    let mut frames = vec![frame(&rig)];
    for step in 1..=600 {
        if (121..=123).contains(&step) {
            rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])?;
        }
        if step == 360 {
            rig.set_enabled(false);
        }
        if step == 420 {
            rig.set_enabled(true);
        }
        if step == 480 {
            rig.reset();
        }
        rig.step();
        frames.push(frame(&rig));
    }
    serde_json::to_writer(std::io::stdout().lock(), &frames)?;
    Ok(())
}
