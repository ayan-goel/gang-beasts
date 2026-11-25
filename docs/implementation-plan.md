# MVP implementation plan

Status: in progress. All release gates in specification §33 remain required.

Each numbered slice uses failing behavioral tests, minimal implementation, appropriate native/browser verification, documentation, and an atomic commit. Dependencies follow the order below. Split further whenever a slice cannot be verified independently.

| Slice | Deliverable | Exit evidence |
| --- | --- | --- |
| 01 — verified | Validated fixed-tick native physics fixture | Falling body settles; reproducible reset; invalid config rejected; fmt/Clippy/tests |
| 02 — verified | Same fixture through WASM and Three.js | Browser loads actual WASM; native/browser fixture agreement; build and browser tests |
| 03 — verified | Playground controls and diagnostics | Pause, single step, reset, impulse and timing work; debug geometry is accurate |
| 03b — verified | Fixture recording/replay and configuration editing | Replayed tick-stamped commands reproduce native/WASM state; invalid imported data is bounded/rejected; exported tuning matches the running world |
| 04 | Fifteen-body passive ragdoll | 40 kg/14 joints, collision filtering, finite falls and aligned anchors |
| 05 | Bounded motors and standing | Upright controller yields to pushes; 60-second stand; visible motor targets |
| 06 | Walking/running and support | Direction/speed/wall/slope/moving-support fixtures and browser inspection |
| 07 | Jump, fall and recovery | Buffered edge consumed once; total impulse bounded; physical get-up |
| 08 | Independent strikes and contact damage | Real contact, attack deduplication, no resting damage |
| 09 | Grips, stamina and release | Per-hand constraints, valid anchors, load failure, cyclic stress cleanup |
| 10 | Lift, throw, knockout and rescue | Momentum preserved; bounded assistance; KO and ledge cases verified |
| 11 | Skyline and group camera | Spawns, props, panels, warnings, camera bounds and local round |
| 12 | Melee match lifecycle | Simultaneous elimination, scoring, full reset and rematch |
| 13 | Rust/TS binary contracts | Golden inputs/snapshots, malformed records, sequence wrap and lifecycle |
| 14 | Dedicated room server | Single owner, bounded queues, authoritative inputs, scheduling tests |
| 15 | Two-to-eight online clients | Interpolation, resync, backpressure and latency matrix |
| 16 | Private sessions and lobby | Link/code, ready/start, auth, reconnect, host transfer, late joining |
| 17 | Original presentation and audio | Original skinned characters, effects, warnings, licensed asset manifest |
| 18 | Settings, accessibility and product flows | Input isolation, keyboard navigation, loading/errors, quality/audio settings |
| 19 | Performance and endurance | Declared hardware measurements, 60-minute soak, 100 resets, eight-player stress |
| 20 | Deployment and release audit | CI, container, compatibility manifest, real two-client smoke and all §33 gates |

Physics feel and grip stability are early risks. Network responsiveness must pass measured latency and human playability gates. Native/WASM deterministic parity is a testable property, not an assumption. No future slice is complete merely because a module exists.
