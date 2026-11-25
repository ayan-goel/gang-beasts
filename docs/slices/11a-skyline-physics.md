# Skyline physics and map data

The original roof in `assets/maps/skyline/map.json` has a 20 × 20 m central deck and two 2 m wide exterior walkways. Their offsets form an asymmetric 24 × 20 m footprint. All walking surfaces start at Y = 0. Two fixed ventilation housings and four pushable 7–10 kg crates occupy the main deck. Eight authored spawns have distinct positions, clearance from solids and a margin from the roof edge.

The manifest fixes object IDs and render order. The parser validates the version, dimensions, masses, overlap, support, spawn clearance and hazard deadlines before allocating physics handles. The manifest contributes to the engine fingerprint and has a separate SHA-256 content identity.

The arena warns at tick 5400. The west walkway detaches at 7200 and the east walkway at 9000. Detachment makes the original body dynamic so it keeps its local anchors. Sudden death starts at 10800 with a lateral gust ramping over three seconds to 9 m/s². The configured final round deadline is tick 12600. Fallen arena objects stop simulating 16 m below the elimination plane. Reset recreates bodies and the schedule from the manifest.

The broad fall sensor and pelvis-center comparison are separate observations. The center crossing −12 m decides a fall, including outside the sensor footprint. Sensor overlap alone can come from the surface of a pelvis that is still above the plane.

## Verification

Run `cargo test --release --locked -p brawler-engine --test skyline` and `cargo test --release --locked -p brawler-engine --lib arena::tests`. Fixtures cover manifest validation, eight spawn clearances, real crate impulses, fixed vents, timed release of both physical panels, sensor edge cases and full reset. Multiplayer room ownership and ledge traversal remain separate gates.
