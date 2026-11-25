use brawler_engine::{LocomotionConfig, LocomotionRagdoll, PhysicsConfig};

#[test]
fn low_stairs_have_a_level_approach_three_adjacent_treads_and_a_long_landing() {
    let mut rig = LocomotionRagdoll::stairs(
        PhysicsConfig::default(),
        LocomotionConfig::bundled().unwrap(),
    )
    .unwrap();
    let boxes = rig.pose().rig().terrain_boxes();
    assert_eq!(boxes.len(), 4);
    assert_eq!(boxes[0], [0.0, -0.5, 0.0, 30.0, 0.5, 30.0]);
    for (shape, (front, back, top)) in
        boxes[1..]
            .iter()
            .zip([(-1.0, -1.8, 0.1), (-1.8, -2.6, 0.2), (-2.6, -12.0, 0.3)])
    {
        let [x, y, z, dx, dy, dz] = *shape;
        assert_eq!((x, dx), (0.0, 3.0));
        assert!((z + dz - front).abs() < 1e-6);
        assert!((z - dz - back).abs() < 1e-6);
        assert!((y + dy - top).abs() < 1e-6);
        assert_eq!(y - dy, 0.0);
    }
    rig.step();
    rig.reset();
    assert_eq!(rig.pose().rig().terrain_boxes(), boxes);
}
