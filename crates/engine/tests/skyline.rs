use brawler_engine::{ArenaKind, Duel, MapDefinition, PhysicsConfig, SkylineEvent};

#[test]
fn skyline_contains_all_authored_objects_and_eight_distinct_safe_spawn_points() {
    let d = Duel::skyline(PhysicsConfig::default()).unwrap();
    let layout = d.map_definition().unwrap();
    assert_eq!(layout.id, "skyline");
    assert_eq!(layout.spawns.len(), 8);
    for (kind, n) in [
        (ArenaKind::Deck, 1),
        (ArenaKind::Vent, 2),
        (ArenaKind::Crate, 4),
        (ArenaKind::Panel, 2),
    ] {
        assert_eq!(layout.objects.iter().filter(|o| o.kind == kind).count(), n);
    }
    assert_eq!(d.arena_states().len(), 9);
    let mut data = serde_json::to_value(layout).unwrap();
    data["spawns"][1] = data["spawns"][0].clone();
    assert!(MapDefinition::from_json(&data.to_string()).is_err());
}

#[test]
fn skyline_panels_warn_detach_once_fall_and_reset_with_the_crates() {
    let mut d = Duel::skyline(PhysicsConfig::default()).unwrap();
    let initial = d.arena_states();
    let mut events = Vec::new();
    for _ in 0..=9005 {
        d.step();
        for event in d.arena_events() {
            events.push((d.tick(), *event));
        }
    }
    assert_eq!(
        events,
        [
            (5400, SkylineEvent::EdgeWarning),
            (7200, SkylineEvent::PanelDetached(8)),
            (9000, SkylineEvent::PanelDetached(9))
        ]
    );
    let after = d.arena_states();
    assert!(after.iter().find(|s| s.id == 8).unwrap().position[1] < -12.);
    assert!(after.iter().find(|s| s.id == 9).unwrap().position[1] < initial[8].position[1]);
    d.reset();
    assert_eq!(initial, d.arena_states());
    assert!(d.arena_events().is_empty());
}

#[test]
fn malformed_map_geometry_cannot_create_floating_props_or_misdirect_panel_failure() {
    let valid = serde_json::to_value(MapDefinition::skyline().unwrap()).unwrap();
    for (pointer, wrong) in [
        ("/objects/1/center/1", serde_json::json!(25.)),
        ("/objects/3/mass", serde_json::json!(-1)),
        ("/objects/7/center/0", serde_json::json!(-13.)),
        ("/objects/8/id", serde_json::json!(8)),
        ("/schedule/east_tick", serde_json::json!(6500)),
        ("/spawns/0/position/0", serde_json::json!(20)),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(pointer).unwrap() = wrong;
        assert!(
            MapDefinition::from_json(&bad.to_string()).is_err(),
            "{pointer}"
        );
    }
    assert_eq!(MapDefinition::skyline_hash().len(), 64);
}
