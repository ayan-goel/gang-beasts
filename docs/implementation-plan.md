# MVP implementation plan

Status: in progress. All release gates in specification §33 remain required.

Each numbered slice uses failing behavioral tests, minimal implementation, appropriate native/browser verification, documentation, and an atomic commit. The terrain investigation remains open alongside jump and recovery. Both use the verified foot-support contract. This follows the core movement order in specification §32.1 and retains the stair acceptance tests. Split further whenever a slice cannot be verified independently.

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
| 06c2b2 — verified fixtures | Varied stopping phases and support-frame cleanup | Bounded force/moment accounting, flat/forward-slope phase grid, short deck stops, impact compliance and native/WASM traces |
| 06c2b3 — verified native fixtures | Rotating-support traversal and carry moment | Held/stop grids, twenty-second idle observation, measured force/moment limits, lifecycle and diagnostics |
| 06c2b4a — verified | Native backward/cross-slope traversal | 48 slope cases, stopping and overpowering fixtures, native/WASM flat traces |
| 06c2b4b — shared native/browser fixture verified, ascent failing | Low stairs and terrain transitions | Ascend three physical treads, cross flat/slope boundaries and handle distinct support normals |
| 06c3 | Browser terrain and human movement assessment | Matching scenes/traces, visible contact transitions and human playability |
| 07a — verified local fixtures | Jump, flight and landing | Single bounded takeoff, coyote and buffer boundaries, finite air control, native/WASM agreement and accessible controls |
| 07b — verified local fixtures | Fallen recovery | Face-up, face-down and side get-up, obstruction and restraint, clear handoff to supported standing, native/WASM shove trace |
| 08a — verified local fixtures | Independent strikes and held reach | Per-hand timing, bounded motors, physical momentum transfer and browser controls |
| 08a2 — verified local fixture | Strike practice scene | Either fist swings a suspended target through physical contact, with matching native/WASM traces, reset and tuning replacement |
| 08a3 — verified native | Two independently controlled physical fighters | One solver, reciprocal fist contact, independent movement/jump and full reset |
| 08b1 — verified native | Physical punch damage events | Positive fist contact, pre-solve normal point speed, region scaling, attack ownership, whiffs and simultaneous punches |
| 08b2 — verified local fixture | Playable two-fighter sparring | Both fighters visible and controllable, native/WASM agreement, hit diagnostics |
| 08c | Target selection and headbutts | Reachable directional targets, wall obstruction, head contact and independent cooldown |
| 09a — verified native fixture | Opponent grips | Surface proximity, occlusion, reciprocal holds and momentum-preserving release |
| 09b — verified browser fixture | Playable opponent grips | Both rigs and all four grip slots agree across native and WASM, with keyboard release and blur cleanup |
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
