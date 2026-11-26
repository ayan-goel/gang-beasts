use super::*;

#[test]
fn a_thrown_crate_cannot_cross_a_thin_solid_in_one_tick() {
    for sign in [-1., 1.] {
        let mut world = PhysicsWorld::new(&crate::PhysicsConfig::default());
        let arena = Skyline::insert(&mut world).unwrap();
        let crate_handle = arena.bodies[5];
        let body = &mut world.bodies[crate_handle];
        body.set_translation(vector![0., 4., 0.], true);
        body.set_linvel(vector![sign * 90., 0., 0.], true);
        world
            .colliders
            .insert(ColliderBuilder::cuboid(0.01, 2., 2.).translation(vector![sign * 0.7, 4., 0.]));
        for tick in 1..=12 {
            world.step();
            let x = sign * world.bodies[crate_handle].translation().x;
            assert!(x <= 0.7, "tick {tick}: crate reached {x}");
        }
    }
}
