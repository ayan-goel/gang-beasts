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
| 06c2b4b — ascent and supported stop verified on the default stairs, five start timings still fail | Low stairs and terrain transitions | Ascend three physical treads, cross flat/slope boundaries and handle distinct support normals |
| 06c2b4c — verified | Selected-limb contact inspection | Native solver load and witness direction, browser body selection and reset |
| 06c3 | Browser terrain and human movement assessment | Matching scenes/traces, visible contact transitions and human playability |
| 07a — verified local fixtures | Jump, flight and landing | Single bounded takeoff, coyote and buffer boundaries, finite air control, native/WASM agreement and accessible controls |
| 07b — verified local fixtures | Fallen recovery | Face-up, face-down and side get-up, obstruction and restraint, clear handoff to supported standing, native/WASM shove trace |
| 08a — verified local fixtures | Independent strikes and held reach | Per-hand timing, bounded motors, physical momentum transfer and browser controls |
| 08a2 — verified local fixture | Strike practice scene | Either fist swings a suspended target through physical contact, with matching native/WASM traces, reset and tuning replacement |
| 08a3 — verified native | Two independently controlled physical fighters | One solver, reciprocal fist contact, independent movement/jump and full reset |
| 08b1 — verified native | Physical punch damage events | Positive fist contact, pre-solve normal point speed, region scaling, attack ownership, whiffs and simultaneous punches |
| 08b2 — verified local fixture | Playable two-fighter sparring | Both fighters visible and controllable, native/WASM agreement, hit diagnostics |
| 08c1 — physical headbutts | Local and online head strikes | Bounded neck and chest motion, physical damage, obstruction, isolated cooldown and stamina |
| 08c2 — directional punches | Aim each punch at a reachable opponent surface | Stable ranking, occlusion, bounded steering, physical hits and native/WASM replay |
| 09a — verified native fixture | Opponent grips | Surface proximity, occlusion, reciprocal holds and momentum-preserving release |
| 09b — verified browser fixture | Playable opponent grips | Both rigs and all four grip slots agree across native and WASM, with keyboard release and blur cleanup |
| 09c — verified local fixtures | Grip stamina and overload | Owner-specific stamina, calibrated suspended weights, exhaustion, sustained overload and release |
| 09d | World grip targets and lifecycle | Props and fixed holds, reverse lookup on destruction, multi-party stress and cleanup |
| 09e — online hand actions | Identify each grip and strike independently with the free hand | Mirrored physical hits, shared limb IDs, moving crate, release and reconnect |
| 09f — stamina feedback | Show each fighter’s available stamina through a hold and recovery | Two room clients see a sustained grip tire and release, and local crate grips use the same meter |
| 10a — verified local encounter | Lift and release in motion | Valid support, per-hand force budget, all target colliders airborne, momentum-preserving release and matching browser trace |
| 10b — local punch and recovery fixtures | Knockout and recovery | Physical impacts, bounded unconscious time, grip ownership, motor ramp and get-up |
| 10c1 — local arena grips | Physical prop and edge grips | Walk to a crate, lift and release it, preserve panel attachment on detachment and clean up retired bodies |
| 10c2 — local ledge catches | Catch and hang from the roof | A falling-walkway input sequence catches the roof and release yields a ring-out |
| 10c3 — local rescue | One- and two-hand mantle onto Skyline | Physical catch, clear landing, supported release, interruption, falling panel and passenger load; network and human assessment pending |
| 10d — Skyline throws | Private-room throw controls and counter-grips | Two-client airborne momentum, matched physical resistance and an east-edge browser ring-out |
| 10e — ledge rescue | Make nearby catches recoverable and show the climber’s pull-up input | Eight catch timings, double-load failure and two browser views of the same rescue |
| [10f — west walkway practice](slices/10f-ledge-practice.md) | Repeat a real rescue with immediate player control over both hands and the pull | Twenty reach/pull timings, ordinary keyboard recovery, voluntary release, miss and retry |
| 11a — verified native | Skyline map and physics | Eight safe spawns, physical crates, vents, warning, detachable walkways and fall threshold |
| 11 | Skyline and group camera | Spawns, props, panels, warnings, camera bounds and local round |
| 12a — verified native local match | Two-player Skyline melee | Countdown, physical eliminations, draws, first-to-three, full reset and rematch |
| 12b — playable browser match | Local Skyline game screen | Group framing, hazard warnings, keyboard ring-out, three rounds and rematch |
| 12c — local phase clock | Preserve local countdown and results duration at low frame rates | Slow-render keyboard ring-out, bounded physics work and correct round reset |
| 12d — camera turns | Rotate the shared local view or a personal online view | Smooth 45-degree turns, held movement and independent network inputs |
| 12e — camera framing | Fit full fighters inside the safe screen area | Per-limb bounds, 15% inset, owned-fighter priority and portrait resizing |
| 12f — Skyline occlusion | Show the concealed parts of the fighter through opaque scenery | West-edge hang, orbit, narrow viewport, moving crate and private-room views |
| 12g — round and match results | Identify wins and draws in local and online Skyline | Same-tick physical falls, duplicate verdicts, first to three and all eight scores after reset |
| 12h — spectator camera | Let an eliminated fighter or waiting spectator follow a survivor | Independent orbit, target handoff after a fall, group view and respawn focus |
| 12i — camera clearance | Keep a clear camera path over moving and fixed geometry | Rotated and retired obstacles, safe framing, stable controls and a two-player vent approach |
| 12 | Melee match lifecycle | Simultaneous elimination, scoring, full reset and rematch |
| 13a — input wire contract | Browser sampling and native codec | Shared 18-byte golden record, edge ordering, malformed input, bounded burst and wraparound |
| 13b — full snapshot codec | Rust encoder and browser decoder | Independent binary fixture, stable body and grip IDs, bounded validation, exact epoch and optional input acknowledgement |
| 13 | Rust/TS binary contracts | Golden inputs/snapshots, malformed records, sequence wrap and lifecycle |
| 14a — verified headless room | Apply input records to two native Skyline fighters | Ordered actions, applied sequence, bounded inbox, stale grip release and round reset |
| 14b — headless state publication | Capture one full world image per publication tick | Recipient-specific applied acknowledgements and a native-to-Chromium capture through grip, score, round reset and panel retirement |
| 14c — verified native workers | Fixed-size worker pool and live snapshot delivery | Bounded catch-up, independent rooms, stale commands, shutdown and clean reuse |
| 14d — verified HTTP service | Axum/Tokio listener with liveness, worker readiness and map catalog | In-process router and real loopback requests against the native binary |
| 14 | Dedicated room server | Single owner, bounded queues, authoritative inputs, scheduling tests |
| 15a — online startup | Two ready players enter one native match | Host authorization, real worker capacity, both views agree on round and score, reload |
| 15b — online fighter controls | Bind each browser to one server-owned fighter | Reload, run both fighters, compare both scores, reset epochs and reject replaced inputs |
| 15c — interpolated online poses | Smooth the complete authoritative scene at browser refresh rate | Shortest-path rotation, clock wrap, adaptive delay, capped extrapolation and stream recovery |
| 15d — native Skyline rosters | Simulate two through eight independent fighters | Spawn slots, cross-roster combat, simultaneous elimination and round rebuild |
| 15e — online match workers | Route two through eight input queues into one world | Eight independent acknowledgements and 120 replicated fighter limbs |
| 15f — multiplayer rooftop | Admit and render two through eight ready players | Eight browser profiles, independent ownership and shared elimination |
| 15 | Two-to-eight online clients | Interpolation, resync, backpressure and latency matrix |
| [15g — private-match latency](slices/15g-match-latency.md) | Measure match-socket RTT and indicate sustained delay | Impair one player at 20, 60, 100 and 200 ms and verify warning and recovery on the same connection |
| [15h — jump feedback](slices/15h-jump-feedback.md) | Send the first jump edge without waiting for the next snapshot | Compare key-to-send and key-to-draw delay in independent regional-profile starts |
| 16a — verified private admission | Anonymous secure cookies, create/join and room metadata | Scoped retry, eight distinct seats, ninth rejected, real Chromium cookie isolation |
| 16b — browser lobby | Create, join, copy invite, reload and leave | Separate browser sessions, host transfer, narrow viewport and switching rooms from an invite |
| 16c — authenticated ready controls | Connect, welcome, roster changes and per-seat readiness | Toggle, cancel, disconnect, socket replacement, explicit reconnect and stale-seat fencing |
| 16d — map loading | Load the renderer and validate the engine and map before ready | Failed WASM download, successful retry, mismatched hashes and new socket checks |
| 16e — disconnect grace | Reserve an offline seat for 20 seconds and reclaim abandoned rooms | Reconnect, batch forfeits, stable scoring and host transfer |
| 16e.1 — disconnect grip verification | Check the holder and the target independently | Native stale-input and forfeit cases, physical separation and two-client recovery with a held key |
| 16f — online rematch | Replay a completed match in the same room | Host-only return, fresh serial, scores and ready roster |
| 16g — automatic match reconnect | Reclaim an interrupted fighter from the same tab | Failed retry, full snapshot, neutral controls and newer-tab ownership |
| 16h — silent match outages | Detect missing snapshots and unanswered WebSocket probes | Separate upstream and downstream TCP interruptions with two live fighters |
| 16i — native round lineups | Carry scores and connection status across round-boundary roster changes | Two-to-eight growth, compaction, input epoch fencing and final-match protection |
| 16j — late joins | Move connected spectators into the next playable round | Three-client spectator-to-fighter transition, score continuity, seat reuse and input ownership |
| 16k — game entry | Open the main menu at the site root | Direct host/join/local choices, separate practice route, cold download and responsive keyboard navigation |
| 16 | Private sessions and lobby | Link/code, ready/start, auth, reconnect, host transfer, late joining |
| 17a — hazard warnings | Show the current walkway and wind warnings during online play | Shared text, server timing, reconnect and round cleanup |
| 17b — skinned fighters | Drive the original Roof Crew GLB from local or online body poses | Independent skeletons, per-vertex deformation, model loading and retry |
| 17c — hazard audio | Four original warning cues in local and online Skyline | Real audio unlock, bounded PCM, mute persistence and reconnect deduplication |
| 17d — fighter condition | Local and online knockout and recovery announcements | Keyboard combat, replicated native transitions, bounded audio and named notices |
| 17e — strike audio | Give damaging contact an audible location and weight | Keyboard and online combat, silent misses, stereo PCM, stale-state and voice limits |
| 17f — grip audio | Hear catches, low-stamina holds and releases | Local crate grabs, two-client fatigue, silent misses, bounded stereo PCM and resync suppression |
| [17g — landing audio](slices/17g-landing-audio.md) | Sound hard contacts with the fixed rooftop | Jump, stumble, silent ring-out, measured stereo falloff and two clients hearing one server event |
| 17 | Original presentation and audio | Original skinned characters, effects, warnings, licensed asset manifest |
| 18 | Settings, accessibility and product flows | Input isolation, keyboard navigation, loading/errors, quality/audio settings |
| 19a — native room baseline | Eight active fighters and eight personalized snapshots | Five release runs with service time, recipient bytes and workload checks |
| 19b — compact snapshots | Advertise compact rotations per connection | Independent fixtures, mixed clients and repeated eight-fighter measurements |
| 19c — snapshot delivery | Wake match connections when their room publishes | Slow-reader coalescing, immediate initial state and repeated eight-client delivery windows |
| 19d — reset endurance | Exercise 100 consecutive round rebuilds in populated native rooms | Physical ring-outs, active grips, hazard retirement, restored spawns and input epochs |
| 19e — graphics presets | Change Skyline resolution and shadows during play | Live GPU allocation, persistent preference, software fallback and independent clients |
| 19f — eight-fighter browser rendering | Replay the native combat workload through the private-room page | Validate eight skins, current input ticks, frame intervals and callback costs on each preset |
| [19g — graphics recovery](slices/19g-graphics-recovery.md) | Restore local play or reconnect to a live private arena after a graphics interruption | Real context loss, neutral controls, frozen local clock, live remote scores and startup retries |
| 19 | Performance and endurance | Declared hardware measurements, 60-minute soak, 100 resets, eight-player stress |
| 20 | Deployment and release audit | CI, container, compatibility manifest, real two-client smoke and all §33 gates |

Physics feel and grip stability are early risks. Network responsiveness must pass measured latency and human playability gates. Native/WASM deterministic parity is a testable property, not an assumption. No future slice is complete merely because a module exists.
