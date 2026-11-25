use brawler_engine::MotorConfig;
use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};
use serde::Serialize;

#[derive(Serialize)]
struct Frame {
    bodies: Vec<f32>,
    joints: Vec<f32>,
    efforts: Vec<f32>,
    targets: Vec<f32>,
    poses: Vec<f32>,
    movement: [f32; 11],
    motors: MotorConfig,
}
fn frame(rig: &LocomotionRagdoll) -> Frame {
    let pose = rig.pose();
    Frame {
        bodies: pose
            .rig()
            .body_states()
            .into_iter()
            .flat_map(|b| b.state.to_buffer())
            .collect::<Vec<_>>(),
        joints: pose
            .rig()
            .joint_states()
            .into_iter()
            .flat_map(|j| j.to_buffer())
            .collect::<Vec<_>>(),
        efforts: pose
            .solver_motor_efforts()
            .unwrap()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        targets: pose
            .solver_motor_targets()
            .unwrap()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        poses: rig.pose_targets().into_iter().flatten().collect::<Vec<_>>(),
        movement: rig.state().to_buffer(),
        motors: pose.motor_configuration().clone(),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rig = LocomotionRagdoll::new(
        PhysicsConfig::default(),
        LocomotionConfig::bundled()?,
        [0.0; 3],
    )?;
    let mut frames = vec![frame(&rig)];
    for step in 1..=480 {
        if step == 61 {
            rig.set_input([1.0, 0.0], false)?;
        }
        if [121, 241, 421].contains(&step) {
            rig.set_input([0.0; 2], false)?;
        }
        if step == 181 {
            rig.set_input([0.0, -1.0], true)?;
        }
        if [271, 272].contains(&step) {
            rig.queue_impulse(BodyPart::Chest, [10.0, 0.0, 0.0])?;
        }
        if step == 301 {
            rig.set_enabled(false);
        }
        if step == 321 {
            rig.set_enabled(true);
        }
        if step == 341 {
            rig.reset();
        }
        if step == 361 {
            rig.set_input([0.0, -1.0], false)?;
        }
        rig.step();
        frames.push(frame(&rig));
    }
    serde_json::to_writer(std::io::stdout().lock(), &frames)?;
    Ok(())
}
