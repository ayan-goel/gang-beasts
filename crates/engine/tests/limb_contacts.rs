use brawler_engine::{BodyPart, LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

#[test]
fn ground_impulses_point_up_from_the_loaded_sole_and_clear_after_reset() {
    let mut rig = LocomotionRagdoll::stairs(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
    )
    .unwrap();
    let contacts = |rig: &LocomotionRagdoll, part| rig.pose().rig().external_contacts(part);
    assert!(contacts(&rig, BodyPart::RightFoot).is_empty());
    for _ in 0..60 {
        rig.step();
    }
    let standing = contacts(&rig, BodyPart::RightFoot);
    assert!(!standing.is_empty());
    for [_, y, _, _, ny, _, impulse] in standing {
        assert!(y.abs() < 0.02 && ny > 0.99 && impulse > 0.0);
    }
    assert!(contacts(&rig, BodyPart::Head).is_empty());
    rig.reset();
    assert!(contacts(&rig, BodyPart::RightFoot).is_empty());
}
