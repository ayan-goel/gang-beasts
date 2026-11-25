# MVP implementation plan

Status: in progress. All release gates in specification §33 remain required.

Each numbered slice uses failing behavioral tests, minimal implementation, appropriate native/browser verification, documentation, and an atomic commit. Dependencies follow the order below. Split further whenever a slice cannot be verified independently.

| Slice | Deliverable | Exit evidence |
| --- | --- | --- |
| 01 — verified | Validated fixed-tick native physics fixture | Falling body settles; reproducible reset; invalid config rejected; fmt/Clippy/tests |
| 02 — verified | Same fixture through WASM and Three.js | Browser loads actual WASM; native/browser fixture agreement; build and browser tests |
| 03 — verified | Playground controls and diagnostics | Pause, single step, reset, impulse and timing work; debug geometry is accurate |
| 03b — verified | Fixture recording/replay and configuration editing | Replayed tick-stamped commands reproduce native/WASM state; invalid imported data is bounded/rejected; exported tuning matches the running world |
| 04a — verified | Native fifteen-body passive ragdoll | 40 kg/14 joints, collision filtering, finite falls and aligned anchors |
| 04b1 — verified | Ragdoll WASM contract | Shared native/WASM body and joint-frame trace, owned buffers, validated limb indices |
| 04b2 — verified | Passive rig in browser playground | Scenario selection, visible collider geometry and joint frames, reset/impulse controls |
| 05a — verified | Native bounded relative pose motors | Quaternion/frame correctness, conserved actuation momentum, per-joint torque caps, finite driven drop |
| 05b1 — verified | Native support-aware standing | 60-second stand, valid foot support, bounded effort, external pushes, reset/disable |
| 05b2 — verified | Driven WASM rig and motor inspection | Native/WASM standing trace; visible motor targets, support/effort readings and browser controls |
| 06a — verified | Native assisted walking/running | Direction/speed, repeated alternating swings, bounded impulse/effort, stop, wall, moving-support and ledge fixtures |
| 06b — verified | Locomotion WASM/playground | Native/browser trace agreement, movement controls and visible gait/turn/stop inspection |
| 06c1 — verified | Native moving-support transitions | Rotation/translation start and stop, partial transfer, impact response and departure momentum |
| 06c2a — verified | Native uphill/downhill gait and braking | 60-second slope stands, walk/run/stop, impacts, bounded targets and absolute impulse budget |
| 06c2b1 — verified | Reversal braking and keyboard transitions | Eight alternating walk/run legs, bounded effort, neutral gaps, lifecycle and native/WASM traces |
| 06c2b2 | Cross-slope, steps and stop-phase control | Cross-slope traversal, tread clearance, terrain transitions and stopping at varied phases |
| 06c3 | Browser terrain and human movement assessment | Matching scenes/traces, visible contact transitions and human playability |
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
