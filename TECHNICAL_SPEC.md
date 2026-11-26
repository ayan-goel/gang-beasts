# Technical specification: Browser-based physics brawler

**Version:** 1.1 — consolidated and technically reviewed  
**Date:** November 24, 2026  
**Reference game:** Gang Beasts  
**Target platform:** Desktop web browsers  
**Core:** Rust + Rapier 3D, compiled natively and to WebAssembly  
**Presentation:** Three.js + TypeScript  
**Status:** Two-player Skyline melee is playable locally in the browser. Private online matches support two through eight fighters on a shared authoritative rooftop. Native and Chromium/WASM fixtures cover combat, knockout recovery, rooftop hazards, elimination, scoring and rematch. The online MVP, performance measurements and human playtests remain incomplete. See `docs/verification/acceptance.md`.
**Intended location:** `TECHNICAL_SPEC.md` at the project root.

Build a browser multiplayer brawler that feels as close to Gang Beasts as practicable: persistent physical characters, independent hands, struggling grips, momentum-driven throws, recoverable knockouts, precarious climbing, hazardous arenas and a readable group camera. Friends should open a link and play. Use original branding, characters, models, sounds, music and level layouts. Gameplay fidelity is the product goal; the MVP is a milestone toward the full game, not permission to replace physical interaction with a generic animated fighter.

This file consolidates all 37 numbered sections of the supplied **Gang Beasts Overview** specification. It preserves its stack, mechanics, map concepts, configuration values, performance hypotheses, APIs and release plan, while making implementation ambiguities explicit. Interactive illustrations have become portable equations, tables and requirements; third-party reference images are not bundled as assets.

### Reading and evidence conventions

- **Verified research:** A public primary source directly supports the stated historical behavior. A historical patch note does not prove the current implementation is identical.
- **Reference observation:** A mechanic or stage description retained from the supplied conversation. It is useful design context, not proof of private implementation details or an exhaustive current catalog.
- **Proposed implementation:** Our independently designed architecture, formulas, controls, tuning values and algorithms. Unless explicitly identified as research, all normative requirements and numbers below belong to this category.
- **Target:** An acceptance goal to measure, not an achieved benchmark or guarantee.
- **MUST / SHOULD / MAY:** Required behavior / preferred approach with documented exceptions / optional extension.

No source establishes Gang Beasts' exact body masses, joint layout, motor gains, damage equation, camera math or wire protocol. Do not describe our values as recovered proprietary constants. Pin actual dependency/toolchain versions during foundation work; the live documentation linked here can change.

### Corrections and clarifications from version 1.0

| Topic | Resolution |
| --- | --- |
| Fifteen-body mass table | Recalculated: **40 kg is correct**. Preserve the original masses; arithmetic is shown in §11.1. |
| Input record | Exactly **18 bytes**, including its type/version bytes, excluding WebSocket/TLS/TCP overhead. Specify byte offsets and endianness (§17.4). |
| Sensors | Sensors are **colliders configured as sensors**, not a fourth rigid-body type (§10.2). |
| Physics settings | Outer gameplay ticks, explicit whole-world substeps, solver iterations and CCD substeps are different settings (§10.1). |
| Motor limits | Distinguish force/torque from impulse. For a substep of duration `h`, `J = F h`; angular impulse is torque × time. |
| Damage | Reduced mass is a gameplay approximation for articulated contacts, not the exact effective mass of two linked ragdolls. |
| Snapshot state | Acknowledged input and acknowledged snapshot are separate concepts; prediction needs more than transforms. |
| Bandwidth | Character-only and environmental-body examples use explicit decimal units and distinguish per-client download from total room egress. |
| Determinism | Native and WASM are not automatically bit-identical. Feature compatibility is version-dependent and must be tested. |
| MVP responsiveness | Interpolation-only networking is a development stage, subject to a playability gate; it is not proof of close online fidelity. |
| Project documentation | This root file is canonical; do not maintain a conflicting copy at `docs/technical-spec.md`. |
| Research links | Remove tracking parameters and broken citation widgets; use real links and disclose the wiki access failure. |

<details>
<summary>Contents — all 37 sections</summary>

1. [How Gang Beasts actually works](#1-how-gang-beasts-actually-works)
2. [Understanding the ragdoll system](#2-understanding-the-ragdoll-system)
3. [Original combat mechanics](#3-original-combat-mechanics)
4. [Map design research](#4-map-design-research)
5. [Camera research](#5-camera-research)
6. [Original game modes](#6-original-game-modes)
7. [Product requirements](#7-product-requirements)
8. [Final technology stack](#8-final-technology-stack)
9. [High-level system architecture](#9-high-level-system-architecture)
10. [Core physics engine](#10-core-physics-engine)
11. [Character architecture](#11-character-architecture)
12. [Procedural locomotion](#12-procedural-locomotion)
13. [Combat system](#13-combat-system)
14. [Map engine and environment system](#14-map-engine-and-environment-system)
15. [Proposed original map roster](#15-proposed-original-map-roster)
16. [Camera engine](#16-camera-engine)
17. [Networking architecture](#17-networking-architecture)
18. [Server architecture](#18-server-architecture)
19. [Lobby and session management](#19-lobby-and-session-management)
20. [Rendering engine](#20-rendering-engine)
21. [Audio system](#21-audio-system)
22. [Game modes](#22-game-modes)
23. [Customization and player progression](#23-customization-and-player-progression)
24. [User interface](#24-user-interface)
25. [Repository structure](#25-repository-structure)
26. [Core engine API](#26-core-engine-api)
27. [Physics development playground](#27-physics-development-playground)
28. [Performance specification](#28-performance-specification)
29. [Testing strategy](#29-testing-strategy)
30. [Security and abuse prevention](#30-security-and-abuse-prevention)
31. [Infrastructure and deployment](#31-infrastructure-and-deployment)
32. [MVP implementation milestones](#32-mvp-implementation-milestones)
33. [MVP acceptance criteria](#33-mvp-acceptance-criteria)
34. [Full-game roadmap](#34-full-game-roadmap)
35. [Technical risks](#35-technical-risks)
36. [Decisions to commit to now](#36-decisions-to-commit-to-now)
37. [Where to start](#37-where-to-start)

</details>

# Part I — Gang Beasts research

## 1. How Gang Beasts actually works

**Verified research.** In a 2016 developer response, Boneloaf described a continuously simulated ragdoll controlled through forces, alongside dynamic/destructible stages and difficulties stabilizing connected bodies. This supports a force-driven physical character architecture; it does not disclose a complete controller. [Boneloaf developer discussion](https://discussions.unity.com/t/official-physics-improvements/619788/24)

| System | Evidence and interpretation |
| --- | --- |
| Engine | Unity, documented in official development posts. |
| Physics backend | The previous conversation identified PhysX. Treat this as Unity-stack context, not a verified current backend version or settings contract. Our implementation uses Rapier. |
| Movement | Custom force-controlled persistent ragdolls, directly supported by the developer discussion. |
| Animation | Physics-driven motion; our pose-generator design is an independent implementation. |
| Camera | Historical Cinemachine use is explicitly documented in the 0.6.0 notes. |
| Input | Keyboard/controller behavior is reference context; our bindings are independently specified. |
| Environment | Dynamic props, breakable structures and hazards are central to the reference experience. |
| Multiplayer | Official notes discuss synchronization work; no public wire protocol is established here. |
| Audio | Official notes describe impact emitters and hazard warnings; our event format is original. |

Official historical camera and gameplay evidence: [0.6.0 development notes](https://gangbeasts.game/posts/2017/9/20/patch-notes-for-the-gang-beasts-060-unstable-beta-build). Product identity and gameplay overview: [official press kit](https://gangbeasts.game/press-pack).

## 2. Understanding the ragdoll system

| Approach | Behavior | Role in this game |
| --- | --- | --- |
| Traditional skeletal animation | Pose tracks directly determine bones; physics is secondary. | May generate desired poses, but must not overwrite colliding physical bodies. |
| Passive ragdoll | Jointed bodies respond to gravity/contact with no intentional motor control. | Knockouts and portions of falling/recovery. |
| Active ragdoll | Motors/torques pursue poses while contacts can overpower them. | Core locomotion, combat, lifting and climbing. |
| Soft-body simulation | Physical deformation of geometry/volume. | Deferred; not necessary for rubbery appearance. |

Use the same physical skeleton before, during and after a knockout. Change motor strength and controller state, not ownership of the body transforms. Skin an original rounded mesh over the physical skeleton. Cosmetic deformation is permitted if it does not misrepresent hit geometry. Volumetric soft bodies are not an MVP requirement.

## 3. Original combat mechanics

The following reference behaviors define the fidelity target. Their exact timing and formulas remain unknown.

| Mechanic | Required experiential behavior |
| --- | --- |
| Walk / run | Wobbly but intentional movement, momentum and resistance. |
| Jump | Physical lift, meaningful landing and limited airborne control. |
| Left/right punch | Independent arms; a free hand can fight while the other holds. |
| Left/right grab | Hold opponents, props and ledges with actual constraints. |
| Headbutt | Short-range physical head strike. |
| Kick / dropkick | Leg-driven and airborne attacks; momentum creates opportunities and risk. |
| Lift / throw | Struggle to raise bodies, then release their momentum. |
| Climb | Use hands to hang, pull and recover over ledges. |
| Dive / crawl / slide / sit | Distinct low or airborne postures in later releases. |
| Knockout / recovery | Temporary loss of control, followed by physical recovery. |
| Stamina | Prevent indefinite holds, including grounded holds. |
| Emote | Expressive pose without bypassing normal physics. |

**Knockout is not elimination.** The core loop is strike → destabilize or knock out → grab → lift/drag → attempt an environmental elimination before recovery. Victims can cling to the environment or another player. Mutual elimination is valid.

Historical official notes support release assistance, directional targeting, secondary attacks and concussion scaling; they do not establish our numerical model. [0.6.0 notes](https://gangbeasts.game/posts/2017/9/20/patch-notes-for-the-gang-beasts-060-unstable-beta-build)

The Aquarium notes document grip/stamina changes, physical limb targeting and recovery behavior. Use them as historical evidence, not a current balance sheet. [Aquarium update](https://gangbeasts.game/posts/2020/12/8/gang-beasts-aquarium-update)

## 4. Map design research

### 4.1 Representative stages

Reference observations retained from the conversation: **Roof** emphasizes exposed edges; **Trucks** uses moving vehicle platforms and overhead obstacles; **Gondola** uses suspended platforms and failing supports; **Subway** combines incoming trains with recoverable track edges; **Buoy** uses unstable floating terrain; **Aquarium** introduces articulated creature danger; **Wheel** uses rotating/suspended structures; **Crane** combines elevation with controllable machinery.

Crane's official update confirms player-operated crane controls and hostile birds. [Crane update](https://gangbeasts.game/posts/2024/11/19/is-it-a-bird-is-it-a-crane-update-125-crane-stage)

The prior conversation listed 21 historical stages plus Crane. Preserve the following **22 reference entries**, but do not claim that this is the complete current roster: the linked community wiki could not be fetched during verification.

### 4.2 Arena catalog and reusable systems

| Reference map | Central reference mechanic | Proposed reusable engine system |
| --- | --- | --- |
| Roof | Elevated edges | Elimination bounds and ledge climbing |
| Ring | Enclosed fighting platform | Elastic rope-like constraints |
| Billboard | Elevated breakable panels | Detachable structural segments |
| Girders | Narrow construction surfaces | Balance and traversal |
| Trucks | Vehicles and road obstacles | Kinematic platforms and telegraphed hazards |
| Subway | Incoming trains | Tick-scheduled moving hazards |
| Grind | Industrial grinders | Motorized mechanisms and damage regions |
| Incinerator | Fire pits and throwable objects | Damage/elimination sensor colliders and props |
| Chutes | Trapdoors and machinery | Hinges and mechanism state machines |
| Containers | Suspended cargo | Cable-like constraints |
| Gondola | Hanging destructible platforms | Support-joint failure |
| Elevators | Moving/failing lifts | Mechanism sequencing |
| Vents | Air currents | Directional force volumes |
| Towers | Unstable elevated walkways | Structural destruction |
| Blimp | Airborne precarious surface | Wind and constrained terrain |
| Wheel | Rotating ride | Articulated structures |
| Lighthouse | Balcony/glass hazards | Pre-fractured geometry and camera occlusion handling |
| Buoy | Floating/disintegrating terrain | Buoyancy and fragmentation |
| Train | Moving railway setting | Relative-motion simulation |
| Trawler | Boat/aquatic hazards | Buoyancy and hazard AI |
| Aquarium | Articulated creatures | Physical AI limbs |
| Crane | Player-controlled machinery | Interaction ownership and mechanism controls |

These are engineering interpretations of reference mechanics, not instructions to duplicate layouts, names or artwork.

### 4.3 Environmental escalation

Compose arenas from persistent physical geometry, interactive objects and server-timed escalation. Progressive support failure, rising water, increasing wind, more aggressive creatures and shrinking safe space should discourage camping. The Aquarium update documents changes spanning buoyancy, glass and stage behavior. [Aquarium update](https://gangbeasts.game/posts/2020/12/8/gang-beasts-aquarium-update)

Every escalation needs a warning, authoritative trigger tick, bounded resource cost, reset behavior and a test fixture. Avoid untelegraphed random deaths. Preserve counterplay: stable refuges can become temporary, but must not be silently removed by arbitrary client-side timers.

## 5. Camera research

**Verified historical fact:** Boneloaf documented a camera built on Cinemachine. Earlier official announcements discuss group tracking, geometry clipping, camera bias, interpolation and damping. [0.6.0 notes](https://gangbeasts.game/posts/2017/9/20/patch-notes-for-the-gang-beasts-060-unstable-beta-build), [official historical announcements](https://store.steampowered.com/oldnews/?appgroupname=Gang+Beasts&appids=285900&enddate=1577865600&feed=steam_community_announcements)

The conversation's claims about particular rotation speeds on Lighthouse are retained only as unverified reference observations. Do not copy a supposed exact camera algorithm from that claim.

Our camera must frame the fight, ledges and approaching danger from an elevated perspective. It is a client-side presentation system. It must never determine authoritative hit tests, elimination or hazard scheduling. Online players may independently orbit; shared-screen local players use one agreed camera.

## 6. Original game modes

The reference-mode taxonomy below is retained from the conversation; it is not a claim about every historical label or current platform menu.

| Mode | Reference experience | Our release |
| --- | --- | --- |
| Melee | Free-for-all survival | MVP |
| Gang / team combat | Team elimination | Full game |
| Waves | Cooperative combat against AI ragdolls | Full game |
| Football | Physical ball and team goals | Full game |

Use melee first. Keep round-win targets configurable and the initial room cap at eight. The official press kit establishes the party-brawler context but is not sufficient by itself to verify every mode rule in this table. Detailed rules for our game are in §22.

# Part II — Product and technical specification

## 7. Product requirements

### 7.1 Core design principles

1. **Close gameplay fidelity:** Every main interaction must remain physical. Do not replace grabs with proximity animations, throws with guaranteed trajectories, or climbing with teleportation.
2. **Emergent play:** Multiple players, props and hazards can interact simultaneously; outcomes emerge from their combined state.
3. **Controlled chaos:** Players can deliberately move, strike, recover and climb while external forces can defeat their controllers.
4. **Physical readability:** Visible hands, surfaces and hazard boundaries correspond to actual interaction geometry.
5. **Low friction:** Link-based joining, no installation and no required account for friend rooms.
6. **Replayability:** Original maps have meaningful mechanical differences and escalating danger.
7. **Performance:** Meet documented hardware targets without lowering authoritative physics fidelity per client.
8. **Extensibility:** Add content through validated data and reusable mechanics.
9. **Original identity:** Use original or properly licensed assets and distinct names/branding; track licenses in the asset manifest.

### 7.2 MVP versus full game

| Feature | MVP | Full game |
| --- | --- | --- |
| Players | 2–8 | 2–8 baseline; larger rooms experimental |
| Online | Private rooms | Private and regional public matchmaking |
| Local multiplayer | Physics development sandbox | Multiple controllers and shared screen |
| Arenas | One polished Skyline arena | 12–20 polished original arenas |
| Physics | Full active-ragdoll interactions | Expanded articulated environments |
| Movement | Walk, run, jump, get-up | Dive, crawl, slide, sit, advanced traversal |
| Combat | Independent punches/grabs, lift, throw, headbutt | Kicks, dropkicks, advanced combinations |
| Consciousness | Knockout and recovery | Refined concussion/recovery tuning |
| Climbing | Reliable simple ledge recovery | Vertical surfaces, moving holds, hand-over-hand |
| Modes | Melee | Teams, waves, football and optional original modes |
| Camera | Dynamic group framing and basic spectating | Map-specific and expanded spectator behavior |
| Destruction | Breakable edge panels/props | Hierarchical structure and pre-fractured glass |
| Customization | Name and color | Original costumes/accessories |
| Input | Keyboard/mouse | Remapping, gamepads, local device assignment |
| Audio | Core effects and warnings | Spatial mix and adaptive music |
| Accounts | None | Optional persistent profiles |
| Replays | Debug recordings | User-facing replay/highlight tools |
| Network | Binary WebSockets, interpolation | Compression and gated prediction; optional new transport |

A fun, complete loop is the MVP gate. Do not ship a stiff capsule controller and call the core physics finished. Temporary simpler rigs may support debugging, but the release rig is the fifteen-body architecture unless a documented fidelity-equivalent design is approved.

## 8. Final technology stack

| Layer | Selection | Responsibility |
| --- | --- | --- |
| Core engine | Rust | Physics orchestration and gameplay |
| Physics | `rapier3d` | Rigid bodies, joints, contacts and queries |
| Math | Rapier-compatible `nalgebra` | Vectors, transforms and quaternions |
| Browser engine | WebAssembly | Local playground and later prediction |
| Bindings / packaging | `wasm-bindgen`, `wasm-pack` | Rust/browser boundary |
| Network service | Axum + Tokio | HTTP, WebSockets, asynchronous I/O |
| Serialization | Explicit custom binary protocol | Compact versioned realtime records |
| Renderer | Three.js + WebGL 2 | Baseline browser graphics |
| Future renderer | Optional WebGPU | Only after profiling/compatibility review |
| UI | React + TypeScript + Vite | Menus, settings, lobby |
| UI store | Zustand | Low-frequency presentation state |
| Audio | Web Audio API | Effects, spatialization and mixing |
| Assets | Blender, glTF/GLB | Original models and scenes |
| Rust tests | Cargo tests + proptest | Gameplay invariants and protocol properties |
| Performance | Criterion.rs, Tracy, flamegraphs | Microbenchmarks and whole-room profiling |
| Browser tests | Playwright | Multiclient product flows |
| Deployment | Docker + persistent Linux host + CDN | Authoritative server and static assets |
| CI | GitHub Actions | Reproducible build/test/package pipeline |
| Observability | `tracing` + Prometheus | Structured diagnostics and metrics |
| Persistence | None initially | Optional PostgreSQL for profiles/statistics |
| Room registry | In-process initially | Redis only when multiple hosts require discovery |

### 8.1 Rationale and dependency policy

Rust provides a cohesive native/WASM core and controlled ownership across room workers. This is an engineering choice, not an unmeasured claim that all Rust implementations outperform all C++ or JavaScript alternatives. Rapier.js is itself WASM-based; the benefit here includes sharing surrounding gameplay code.

Three.js owns presentation without introducing a second full game framework. No Bevy/Unity runtime, custom general-purpose physics engine, mandatory ECS or database is needed for the MVP. Consider Jolt/C++ only if a controlled equivalent-workload prototype demonstrates a material advantage that outweighs migration cost.

Foundation implementation selects Rust 1.88.0 and Rapier 0.26.1 with `enhanced-determinism` and `debug-render`, and no parallel/SIMD features; dependency changes require native/WASM verification. Configuration files use strict versioned JSON (see `docs/decisions/0001-foundation-boundaries.md`). Pin Rust in `rust-toolchain.toml`, commit `Cargo.lock` and the frontend lockfile, pin the Node runtime and `wasm-pack`, and record Rapier features. Use a compatible math version; do not accidentally introduce duplicate incompatible transform types. Validate both native and `wasm32-unknown-unknown` targets before accepting upgrades. The current documentation uses APIs that may differ from older examples; this document specifies behavior rather than promising every illustrative signature matches an unpinned release. [Rapier documentation](https://rapier.rs/docs/), [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)

## 9. High-level system architecture

### 9.1 Ownership boundaries

```text
Browser input → world-space intent → binary WebSocket → bounded room inbox
                                                            │
                                                 Native Rust simulation
                                                 Rapier + gameplay rules
                                                            │
Browser UI/audio/camera ← render state ← interpolation ← authoritative snapshots
          │
          └── optional local Rust/WASM world: playground, later prediction
```

The server owns transforms, velocities, bodies, joints, health, stamina, grips, destruction, hazards, RNG, eliminations, scores and match state. Clients own camera, graphics, audio mix, settings, interpolation and reversible cosmetic feedback. Clients send intentions, never authoritative positions, hit claims or winners.

The shared engine must not depend on HTTP, Tokio, Three.js or wall-clock time. A room owns one engine instance. Transport, protocol, simulation and presentation remain separate modules with explicit data contracts.

### 9.2 Frequencies

| Loop | Initial proposed rate | Meaning |
| --- | --- | --- |
| Authoritative gameplay tick | 60 Hz | Fixed `dt = 1/60 s` |
| Client rendering | 60 FPS baseline; up to 144 on capable devices | Independent render loop |
| Snapshot publication | 30 Hz | Every two completed gameplay ticks |
| Input state sampling | 60 Hz | Preserve action edges between samples |
| Explicit physics substeps | 1 per gameplay tick initially | Increase only with measured stability need |

Use integer ticks for gameplay timers. Rendering frequency, network jitter and host wall-clock adjustments must not change action cooldowns or hazard timing.

## 10. Core physics engine

### 10.1 World configuration

| Parameter | Proposed baseline |
| --- | --- |
| Units | Meters, kilograms, seconds, radians internally |
| Coordinates | Right-handed, Y-up; define character-local forward as −Z |
| Gravity | `(0, -9.81, 0)` m/s² |
| Gameplay timestep | `1/60 s` |
| Character | Approximately 1.6 m tall, exactly 40 kg configured mass |
| Solver setting | Start at 8; compare 8–12 in difficult ragdoll scenes |
| Explicit world substeps | 1; compare 2 or 4 only when justified |
| Character restitution | Low; initial candidate 0–0.1 |
| Friction | Material-specific; tune feet, body, surfaces separately |
| Collision detection | Discrete generally, selective CCD for fast limbs/props |

**Keep settings distinct.** If a gameplay tick contains `N` explicit physics calls, each has `h = (1/60)/N`; recompute continuous controller forces consistently for each call. One-time jump/throw impulses apply once per gameplay event, not once per substep. Aggregate contacts and damage before resolving the gameplay tick. Rapier solver iteration terminology and internal substepping are version-specific; `max_ccd_substeps` is not a substitute for explicit whole-world substeps. Map these concepts to the pinned API and benchmark the combined cost. [Rapier integration parameters](https://rapier.rs/docs/user_guides/rust/integration_parameters/)

### 10.2 Bodies and colliders

| Rigid-body type | Use |
| --- | --- |
| Dynamic | Limbs, props, detached structure |
| Fixed | Static world architecture |
| Kinematic, position- or velocity-based | Scripted platforms, vehicles and machinery |

**Sensors are collider configuration, not rigid bodies.** A sensor collider detects overlap without solid contact response; it can be attached to an appropriate body or exist as static geometry. Use sensors for water, elimination, interaction candidates and damage regions. Explicitly avoid unintended mass contributions from auxiliary sensors. Configure the collision/event filters required by the pinned Rapier version. [Rapier colliders](https://rapier.rs/docs/user_guides/rust/colliders/), [rigid bodies](https://rapier.rs/docs/user_guides/rust/rigid_bodies/)

Separate render geometry from collision geometry. Prefer spheres, capsules, cuboids and simple convex hulls. Use static triangle geometry selectively; do not derive detailed dynamic collision meshes from arbitrary render triangles.

### 10.3 Collision layers

Define `Character`, `CharacterAuxiliary`, `Environment`, `DynamicProp`, `Hazard`, `GrabSensor`, `EliminationSensor` and `Cosmetic` categories. These are conceptual categories; implement the actual masks/pair filters with explicit tests.

| Pair | Policy |
| --- | --- |
| Different characters' important limbs | Physical contact |
| Adjacent bodies in one skeleton | Suppressed |
| Nonadjacent own limbs | Selective, after stability tests |
| Character ↔ environment/props | Physical contact |
| Grab sensor ↔ grabbable targets | Candidate overlap only, no damage |
| Elimination sensor ↔ character | Authoritative trigger, deduplicated by player |
| Cosmetic ↔ authoritative world | No gameplay influence by default |

Category masks alone cannot distinguish adjacent limbs of the same character from another player's limbs. Use body ownership/topology-aware pair filtering or joint contact settings. Grabbing does not automatically suppress every contact between the two characters.

### 10.4 Stability and failure handling

Bound motor torque, controller force, release impulses, linear/angular velocity and object counts. Use plausible neighboring mass ratios, aligned anchors, angle limits and selective CCD. Track penetration, joint error, maximum speed and nonfinite values.

Never allow an invalid body to poison the whole room. Isolate and remove an invalid entity safely, release all incoming/outgoing grips, publish its lifecycle event and log a replay reference. If the entire world is invalid, end the round as a technical abort with no winner, then rebuild it. Do not silently award a win because of a server error. Resetting a body during ordinary combat is not an acceptable substitute for fixing stability.

## 11. Character architecture

### 11.1 Ragdoll skeleton and verified mass arithmetic

| Body part | Count | Mass per body | Subtotal | Collider |
| --- | ---: | ---: | ---: | --- |
| Head | 1 | 4 kg | 4 kg | Sphere |
| Chest | 1 | 10 kg | 10 kg | Capsule |
| Pelvis | 1 | 6 kg | 6 kg | Capsule |
| Upper arm | 2 | 1.5 kg | 3 kg | Capsule |
| Forearm | 2 | 1 kg | 2 kg | Capsule |
| Hand | 2 | 0.5 kg | 1 kg | Sphere |
| Thigh | 2 | 3.5 kg | 7 kg | Capsule |
| Shin | 2 | 2.5 kg | 5 kg | Capsule |
| Foot | 2 | 1 kg | 2 kg | Rounded cuboid |
| **Total** | **15** | | **40 kg** | |

`4 + 10 + 6 + 2×1.5 + 2×1 + 2×0.5 + 2×3.5 + 2×2.5 + 2×1 = 40 kg`.

The source table was already correct. Do not change its masses merely to manufacture a correction. Ensure actual Rapier body mass includes exactly the intended collider/body mass contributions; extra sensor or cosmetic colliders must not silently change it. Test the computed total within floating-point tolerance.

The anatomical tree contains 14 joints: neck 1, spine 1, shoulders 2, elbows 2, wrists 2, hips 2, knees 2, ankles 2. Eight players mean **120 dynamic character bodies and 112 anatomical joints**, before grips and environment. At most two outgoing grips per player means at most 16 player-owned grip constraints, although many can target one character.

A reduced rig is permitted for early diagnosis, not as an unexamined final fidelity reduction. Define collision shapes, anchor frames, bind transforms, body IDs and local forward axes in one versioned character asset/configuration.

The initial native passive rig uses `config/character.json` schema 1: canonical anatomical body order, the table's per-part masses/shapes, positions, shared-frame anchors and per-axis angular limits. The version-one frame convention is right-handed +Y up/−Z forward with identity body and local joint-frame bind rotations; spawn supplies translation only. Dimensions produce a 1.6 m rig with feet at local Y=0. Identity frames are an explicit schema restriction; future rotated authoring must extend it. Strict validation preserves anatomical topology, shape categories and angular-axis patterns. See `docs/decisions/0005-passive-character-rig.md` for limits and the native diagnostic spawn contract. Native passive fixtures are partial evidence; browser ragdolls and active control remain separate gates.

### 11.2 Joint design

| Joint | Proposed constraint |
| --- | --- |
| Neck / spine | Limited angular movement |
| Shoulders / hips | Multi-axis angular limits and motors |
| Elbows / knees | Revolute, anatomically sensible axis/limits |
| Wrists | Fixed initially or softly limited angular motion |
| Ankles | Limited angular motion |

“Limited spherical” describes required behavior, not a promise that a bare spherical joint exposes every angular limit/motor. Select a generic joint with locked translation and configured angular axes where needed. Start with impulse joints because temporary inter-character grips can create loops; evaluate multibody representations only after verifying loop handling and lifecycle complexity. [Rapier joints](https://rapier.rs/docs/user_guides/rust/joints/)

Expose target rotation/velocity, stiffness, damping, limits and maximum effort. A fixed wrist has no meaningful motor degree of freedom. Keep limits per-axis and identify local joint frames explicitly. Do not copy illustrative PD numbers without considering mass, inertia, timestep and the selected motor model.

### 11.3 Active-ragdoll controller

For a single rotational axis:

```text
torque = Kp × angle_error + Kd × (target_angular_velocity − angular_velocity)
torque = clamp(torque, −max_torque, +max_torque)
```

Use a shortest-arc quaternion error converted to a rotation vector for 3D orientation, measured in the same frame as the angular-velocity error. Normalize quaternions and resolve the `q`/`−q` sign ambiguity. Do not subtract quaternion components as if they were angles.

Illustrative normalized UI values from the source were stiffness 45 and damping 7; the playground also used stiffness 60. These are separate illustrative presets, not calibrated physical gains. Store real per-joint gains with units/model metadata. Increase damping to reduce oscillation, but verify recovery speed and contact compliance. Character control must yield to meaningful external forces.

The first native pose-control diagnostic uses `config/motors.json` schema 1 with model `explicit_pd`, per-joint stiffness in N·m/rad, damping in N·m·s/rad, and a whole-vector torque cap in N·m. It targets the relative bind pose and zero relative angular velocity. Quaternion rotation-vector error and relative velocity are expressed in the parent joint frame; locked-axis effort is removed before vector clamping. All joints sample the same pre-actuation state, then apply equal/opposite world-space angular impulses of `torque × 1/60 s`. No persistent force, transform override, or absolute airborne stabilization is added. Fixed wrists have no actuation. These gains and input ceilings are initial tuning, not a stability guarantee for arbitrary accepted values. See `docs/decisions/0006-bounded-relative-pose-motors.md`. Native and browser standing use the solver preset below; the browser inspector exposes the actual targets, effort and support readings. Locomotion and human playability remain subsequent gates.

The standing preset in `config/standing-motors.json` extends motor schema 1 with `implicit_velocity_pd`. It retains the shortest-arc quaternion rotation-vector error, then sets force-based solver velocity motors with factor `Kd + Kp × tick_dt` and target relative angular velocity `Kp × error / factor`. Targets are sampled once per tick. Each free axis receives at most `joint_vector_cap / sqrt(free_axis_count)`, conservatively preserving the vector effort limit. Zero factor/effort disables actuation. This preset addresses the observed instability of stronger explicit gains; it adds no pose constraints or transform overrides. `config/motors.json` retains the explicit diagnostic model. See `docs/decisions/0007-support-aware-standing.md` for tuning, solver assumptions and effort-coordinate conventions.

The native locomotion preset in `config/locomotion-motors.json` strengthens only thigh/shin/ankle actuators to stiffness 600 N·m/rad, damping 12 N·m·s/rad and vector cap 150 N·m. Implicit motor input ceilings are now 600/20/150; explicit ceilings remain 200/20/50. Fixed wrists remain unactuated. Supported movement/braking uses the active preset, which relaxes to standing after settling or support loss. The bounded pelvis tilt controller remains unchanged; movement adds a separate supported yaw budget up to 20 N·m. See `docs/decisions/0008-native-locomotion.md` for physical tradeoffs and limits.

### 11.4 Orthogonal character state

Conceptual fields:

```text
Character:
  id, body_registry, anatomical_joints
  locomotion_state, consciousness_state
  left_hand_state, right_hand_state
  health, stamina, knockout_count
  desired_direction, gait_phase, grounded
  support_body, support_contact, attack_ids
  last_damage_tick, recovery_start_tick, elimination_tick
```

Locomotion, consciousness and each hand are separate state machines. A character may walk, hold with one hand and punch with the other. Consciousness gates available actions; elimination overrides all gameplay controllers.

### 11.5 Consciousness state machine

`Active → KnockedOut → Recovering → Active`; any live state may become `Eliminated`.

Knockout disables active balance, lowers pose motor strength and removes the knocked-out player's own grips. Other players may continue holding that body. Recovery ramps effort over 0.5 seconds initially, selecting a get-up pose instead of snapping the skeleton upright. State changes happen at tick boundaries, emit one event and are idempotent. Repeated damage cannot create unlimited knockout extension.

## 12. Procedural locomotion

### 12.1 Walking and running

Transform input to a bounded world-space horizontal intent; determine target speed; obtain the support surface velocity; apply bounded acceleration; generate alternating limb targets; drive motors; update foot contacts and balance. Never set the torso position each frame to force movement.

```text
phase_next = phase + 2π × gait_frequency × dt
left_hip_target  = amplitude × sin(phase)
right_hip_target = amplitude × sin(phase + π)
```

This gait supplies desired poses, not guaranteed foot trajectories. Scale stride with intended speed and blend toward idle. Foot contacts determine stance; avoid an actuator that accelerates indefinitely against a wall. Full-game work adds terrain-aware foot placement, slope handling, contact-aware transitions and stumbling.

The implemented native `LocomotionRagdoll` uses versioned `config/locomotion.json` with 2.5/4.5 m/s walk/run targets, 1.6/2.5 Hz cadence and maximum 12 m/s² horizontal assistance. It normalizes world X/Z intent, subtracts sampled moving-support point velocity from COM feedback, and allocates one bounded absolute impulse budget between mass-proportional body application and supported feet. Solver-combined friction, motion direction, support normals and pelvis lean/rate determine the allocation; this is ground-gated assistance, not a traction limit, and remains possible on zero-friction support. No vertical impulse or transform override is used. Alternating stance sweeps and raised returns generate bounded relative targets. Release uses a bounded braking interval, weaker velocity damping and a settled transition to idle gains; no position target is stored. Native direction, speed, alternating-foot, wall, moving-platform, ledge and shove fixtures are partial locomotion evidence. Native/WASM locomotion parity and browser controls are verified in `docs/slices/06b-browser-locomotion.md`; terrain and human playability remain required. Gait sine and heading atan2 use Rapier math traits so the pinned enhanced-determinism build uses the same implementation across native/WASM. See `docs/slices/06a-native-locomotion.md` and `docs/decisions/0009-browser-locomotion.md`.

Native slope adaptation derives bounded hip/knee reach and sole targets from measured support normals in a desired gravity-upright frame, retaining pelvis heading. A virtual support plane adjusts ankle height and reserves extension for downhill strides; the two-link solve and all final joint angles remain limited. An ankle limit may leave sole-orientation error rather than moving a reachable step target to erase it. This is pose planning, not terrain snapping or an absolute foot anchor. The reversal-braking allocation can apply a counteracting foot impulse while braking the torso: a signed foot share in [−1,1] is normalized so the sum of absolute commanded linear impulses remains ≤8 N·s per tick at the bundled 40 kg/12 m/s²/60 Hz settings. Ordinary active steering retains a nonnegative foot share; the reversal transition below may temporarily blend toward signed braking. Diagnostics report the resulting net acceleration. Idle damping, pure balance/yaw limits, motor caps and unsupported behavior remain unchanged. See `docs/decisions/0011-slope-gait-and-braking.md` and `docs/slices/06c2a-native-slopes.md` for the native ±15° uphill/downhill fixtures and their limits. Cross-slope travel and stop-phase coverage are recorded below. The [low-step pose decision](docs/decisions/0016-low-step-pose.md) describes stair detection, tuning and limitations.

Direction reversals are sampled at physics ticks using the latest requested input. An obtuse change from a recent supported direction enters a bounded braking transition: squared negative directional cosine blends propulsion, gait and yaw toward neutral and permits the existing normalized counteracting foot allocation. Resume the latest analog direction and run state when opposing support-relative COM speed is at most `maximum_acceleration × tick_dt`, or when `braking_ticks` expires. Repeated samples cannot restart the interval. Neutral always cancels movement but retains direction history for at most `braking_ticks` neutral ticks, so short keyboard release/press gaps cannot bypass reversal braking. Neutral preserves the last nonzero gait cadence during deceleration; a new nonzero request immediately selects its requested walk/run cadence. Reset, disable, absent support and insufficient upright posture clear the transition/history. No configuration field, actuator cap or transform authority changes. See `docs/decisions/0012-reversal-braking.md`; fixture evidence and remaining limits are tracked in `docs/slices/06c2b1-reversals.md`.

Explicit neutral input with active motor strength now uses a separate supported stopping force pair. It counteracts drag and pelvis lean/rate even as net speed approaches zero, allocating mass-proportional body impulses and equal supported-foot impulses within the original absolute linear budget. The full moment about the actual rig COM is separately capped at `stopping_moment_cap_nm × motor_strength`; bundled schema 3 retains the mandatory finite [0,120] N·m setting, default 120. This additional moment includes yaw and is distinct from the 30 N·m pure pelvis balance and 20 N·m intentional turn limits. Zero disables the pair. No vertical impulse, position tether or transform override is added. The pair fades with motor strength and clears on disable, reset, absent support or ineligible posture; nonzero input retains the existing steering/reversal allocation.

Stopping drag compares velocity at the same point: sampled support contact velocity is extrapolated to rig COM with `support_angular_velocity × (COM − contact_point)`. This avoids interpreting rigid co-motion on a rotating deck as sliding. Settling compares pelvis spin against mean selected-support spin; fixed surfaces contribute zero. Ordinary propulsion/support diagnostics retain their contact-point velocity convention. Short deck stops and varied flat/forward-slope phases have regression coverage. The subsequent rotating-support correction below extends moving traversal and idle observation. The traversal fixtures below cover backward slopes, cross-slopes and the three-riser approach. Human playability remains open. See `docs/decisions/0013-stopping-force-pair.md` and `docs/slices/06c2b2-stopping.md` for evidence, moment accounting and the increased resistance to impacts during braking.

Rotating supports additionally use a bounded horizontal compensation pair during active movement and settled idle. Estimate the changing support velocity field at COM by averaging `support_angular_velocity × (COM_velocity − support_body_linear_velocity)` and discarding Y. Subtract the existing mass-distributed body command along this estimate, clamp the remaining load between zero and its original magnitude, and apply an opposite foot command with matching distributed body assistance. Measure the incremental full-vector moment about actual rig COM and cap it at `support_moment_cap_nm`. Normalize the combined original-plus-compensation allocation by `|body_command| + |foot_command|`, preserving the original absolute linear impulse budget. Scale both stopping and support diagnostics by the same factor. No vertical impulse, transform override, scene-specific branch or unbounded force is added.

Locomotion schema 3 requires the new support cap, finite in [0,120] N·m, bundled 120; schema 2 is rejected and migration requires an explicit cap choice. Zero disables compensation. This incremental moment is additional to stopping, pure balance and intentional yaw, and can remain nonzero after active motor strength settles to zero. Reset, disable, absent support and ineligible posture clear it. Static/constant translating supports receive no new correction. The estimate assumes constant support angular/linear velocity; arbitrary mechanism acceleration and changing support topology remain separate verification concerns. Native evidence covers 32 four-second traversals followed by six-second stops, 64 short deck stops extended to twenty seconds, and the formerly failing finite-deck run. Impact resistance varies with phase and radius. See `docs/decisions/0014-rotating-support-moment.md` and `docs/slices/06c2b3-rotating-traversal.md`; browser terrain and human movement acceptance remain open.

#### 12.1.1 Terrain controller and stumble recovery

The terrain controller extends the rotating-support baseline with a lateral gait clearance envelope, projected normal-load estimates, corrected interior cuboid support normals, and bounded active contact and lean correction. Ordinary steering can change the signed foot allocation within the absolute impulse budget.

Settling measures consecutive completed pelvis orientations relative to a shared moving support body, mapping the resulting rate to current world axes. Fixed parents and unparented static colliders share the world frame. Support changes and discontinuous ticks use instantaneous relative spin; mixed moving supports clear history and use current pelvis spin minus mean support spin. Pelvis, chest and head must meet the existing upright threshold before strength decreases. Force damping still uses instantaneous angular velocity. Active drag and the main movement/settling/carry feedback extrapolate support velocity to COM, matching stopping drag; support-velocity diagnostics retain their contact-point convention. Controlled native fixtures verify the settling predicate during support acceleration/deceleration and topology changes; they do not establish free-running traversal under arbitrary platform acceleration. See [support-relative settling](docs/verification/support-relative-settling.md). Retained directional stride after neutral input was rejected for regressing impact compliance.

During upright support loss, all world assistance still stops immediately, while each internal joint-angle target returns toward bind by at most its anatomical range times `tick_dt / motor_release_s` per tick. This stateless rate is independent of active motor strength; settled slope targets must not snap either. Reset, disable and ineligible posture still clear targets immediately. Support reacquisition resumes normal gait/terrain calculation. See [internal pose continuity](docs/verification/support-loss-pose.md) for the landing-shock reproduction and scope.

The implicit motor allocator uses the following rule. Estimate parent-frame effort as `Kp × rotation_error − (Kd + Kp × tick_dt) × relative_angular_velocity`, zero locked axes, and clamp the estimate to the existing joint vector ceiling. Distribute remaining squared capacity equally among free axes; each axis ceiling is the square root of its squared predicted effort plus that reserve. Thus the squared axis ceilings sum to the original squared vector budget within floating-point precision. This replaces the earlier equal-axis allocation within the same configured vector ceiling. A saturated prediction can temporarily leave another axis without spare capacity. See [joint effort allocation](docs/verification/motor-effort-allocation.md) for the causal controls, tests and tradeoffs.

When movement is released during an advancing swing, the airborne leg can finish its placement using support-relative COM velocity and displacement from the planted foot. The correction ends at touchdown, loss of stance support, the half-cycle boundary or braking expiry. Requested foot geometry and anatomical limits bound each correction. A stumble begins when any of pelvis, chest or head drops to the existing minimum upright projection. Walking assistance resumes when the entire trunk recovers to its settling threshold, with the minimum eligibility threshold still satisfied. All 48 native forward/backward/cross-slope cases and the prior stopping and impact regression suites pass together. The subsequent low-step controller covers the default three-riser approach. Other terrain transitions and human assessment remain open. [Design decision and verification](docs/decisions/0015-terrain-locomotion.md)

### 12.2 Balance

A baseline upright correction is `torque = Kp × (body_up × desired_up) − Kd × angular_velocity`, with bounded effort. Near upside-down configurations, the cross product alone is ambiguous; select a recovery pose or use the quaternion-error controller. Do not let upright torque cancel arbitrary impacts or act as infinite airborne stabilization.

An advanced controller projects the combined center of mass into the foot support region and changes step targets. Reduced support or high external loads should cause stumbling. Grounded assistance must stop or weaken when feet leave the ground.

The initial native standing controller uses strict versioned `config/balance.json`: a 30 N·m pelvis tilt-torque cap, halved with one supporting foot and zero with none. Yaw remains free. Support requires an active external foot contact, a fresh current-shape query within 0.015 m, upward normal Y ≥0.7, and foot/surface normal separation speed ≤0.5 m/s. Own limbs and sensors are excluded; current enabled flags for feet, surfaces and their parent bodies are checked before using cached contacts. Deleted peer colliders are skipped safely. Actual shape revalidation rejects lateral ledge departure. This contact tolerance is separate from the later locomotion ground-query distance. Queued impulses are consumed before support evaluation. No vertical lift, horizontal tether, gravity override or transform snapping is used. Native/WASM standing and push fixtures, plus browser motor/support inspection, provide partial evidence. Locomotion and human playability remain separate gates.

### 12.3 Initial movement constants

| Parameter | Proposed value |
| --- | ---: |
| Walk speed | 2.5 m/s |
| Run speed | 4.5 m/s |
| Jump velocity change | Up to 4.5 m/s |
| Maximum grounded acceleration | 12 m/s² |
| Ground-check distance | 0.15 m beyond the foot shape |
| Jump input buffer | 100 ms = 6 ticks |
| Coyote window | 100 ms = 6 ticks |
| Walk gait frequency | 1.6 Hz |
| Run gait frequency | 2.5 Hz |

Configuration owns these values. Grounding requires plausible support normals/contact distance; a wall or another arbitrary nearby collider is not automatically ground. Use contact data plus short shape queries and stable support selection.

### 12.4 Jumping and airborne movement

`J = m × Δv`; for the 40 kg character and 4.5 m/s change, the total impulse budget is **180 N·s**, not 180 N·s per body. Distribute it by the chosen mass/actuation policy and preserve the total. A pelvis/chest-only impulse can create joint shocks; compare it with a mass-proportional impulse in the playground.

Allow one jump from a support episode or the coyote window; consume buffered press edges once. Holding jump does not retrigger it. Use bounded weaker airborne steering, with no unlimited horizontal acceleration. Climb pulls use their own stamina/force rules. Moving-platform departure retains existing support momentum without adding it twice.

The local controller implements a mass-proportional takeoff impulse on the next eligible simulation tick. The six-tick press buffer includes the first sampled tick. The six coyote ticks begin at the first unsupported sample following eligible support. A held key remains consumed across landing. New support after a sampled departure rearms takeoff. An ineligible trunk posture or disabled controller clears the windows. Reset starts a fresh episode. A release preserves a buffered tap, while input cancellation on pause or focus loss discards it.

Air steering after an intentional jump has two limits. At bundled settings the character can change horizontal velocity at 2.4 m/s² and spend at most 1 m/s of total horizontal velocity change during that flight. Steering reversals spend the same budget. The impulse preserves existing support momentum, and ordinary ledge departures keep their ballistic motion. Schema-three locomotion configuration accepts a nested `jump` object and supplies the bundled defaults for older files. [Jump fixtures and controls](docs/slices/07a-jumping.md)

### 12.5 Get-up behavior

Detect torso orientation and contacts; select face-up, face-down or side recovery; move limbs toward support poses; gradually regain balance. A blocked or held character must struggle physically rather than clip through the captor/floor. Resume normal gait when posture and support are sufficient. Retain controllable wobble, not a compulsory canned animation.

The local locomotion rig initiates get-up after 18 consecutive eligible ticks with low pelvis speed relative to a fixed or kinematic supporting surface and a fallen trunk. Shoulder reach changes with the direction the character faces relative to gravity. Knees tuck and then extend as the pelvis rises. Joint targets stay within the authored angular limits. Recovery effort ramps over 30 ticks. A grounded force pair acts on the upper and lower body with equal and opposite total linear impulses. Its default limits are 600 N per side and 360 N·m for the resulting moment. Drive permission ends immediately on loss of suitable contact or disable. A time limit and rest period bound obstructed attempts. Normal gait takes over after both feet support an upright trunk for twelve ticks. These limits and recovery-specific loads are separate from ordinary standing balance and walking assistance. Character actuators and diagnostic buffers address the fourteen owned anatomical joints, leaving independent constraints intact. [Design and verification](docs/slices/07b-recovery.md)

## 13. Combat system

### 13.1 Controls

These are our proposed bindings, not a claim about Gang Beasts' exact keyboard layout.

| Input | Action |
| --- | --- |
| WASD | Camera-relative movement |
| Space | Jump; pull/climb while gripping |
| Shift | Run |
| Left mouse | Left punch, then reach/grab if held |
| Right mouse | Right punch, then reach/grab if held |
| E | Lift held object; raised-arm pose |
| Q | Headbutt |
| F | Kick, full game |
| Ctrl | Crouch/crawl, full game |
| Z / X | Previous/next allowed camera angle |
| Tab | Scoreboard |
| Esc | Settings/menu; does not pause an online room |

Disable the browser context menu over the game canvas, not globally. Do not require pointer lock for basic play. On blur, focus loss, menu opening or device removal, send releases/neutral state and stop accumulating gameplay actions. The server also expires stale held input. Text fields consume typing without moving the player.

Full-game gamepad proposal: left stick movement, A/south jump, shoulders independent hands, Y/north lift, B/east headbutt, X/west kick, stick click run, right stick or D-pad camera, Start menu. Device labels and remapping must be configurable; do not assume every device uses the standard layout. Provide keyboard alternatives for mouse actions and test keyboard rollover.

### 13.2 Independent hand state machines

`Idle → WindUp → Strike → Recovery → Idle`; held input after the strike transitions to `Reach → Gripping` when contact is valid. Releasing before a grip ends reach/retracts; releasing while gripping removes the constraint. On a new press, start a strike immediately rather than delaying it to classify a tap or hold. Attack cooldowns still apply.

Define priority: elimination/knockout overrides everything; an existing grip remains a grip while held; lifting/climbing modifies gripping-arm targets; a free hand may punch; a held hand does not simultaneously create a new damaging punch. Each attack has a server-created ID and a per-target hit set. A continued hold is not an automatic punch macro.

The local controller samples each hand independently. A press and release between two ticks still produces one punch. Focus loss clears held input and unsampled presses. A punch that has started keeps its full 24-tick cooldown. Each accepted punch gets a monotonically increasing ID within the character fixture. Networked hits must also carry the attacker and round identity.

### 13.3 Punches, targeting and timing

| Phase | Proposed design time | Initial 60 Hz schedule |
| --- | ---: | ---: |
| Wind-up | 80 ms | 5 ticks ≈83.3 ms |
| Strike | 120 ms | 7 ticks ≈116.7 ms |
| Recovery | 200 ms | 12 ticks =200 ms |
| Total | 400 ms | 24 ticks |

The playground uses shoulder/elbow pitch pairs of [0.15, 1.80] radians in wind-up, [1.65, 0.15] in extension and [1.45, 0.30] in held reach. The active arm uses stiffness 160 N·m/rad, damping 4 N·m·s/rad and a 45 N·m torque ceiling per motor. Authored joint limits bound each pitch. The opposite arm retains its ordinary gait or independent action. Over the twelve recovery ticks, a tap retracts toward rest and a hold settles into reach. When a foot is supported, the controller measures the gravity moment of the six arm bodies about the chest and biases the chest and supported ankle pitch targets to counter it. Each bias is capped at 0.20 radians and retains the ordinary motor torque ceiling. These versioned settings live in `config/hands.json`.

Drive shoulder and elbow targets through anticipation, extension and retraction. Each new punch selects an unobstructed, reachable opponent surface within a 40-degree forward cone from that shoulder. Rank by alignment, distance, fighter slot and limb index. The controller can adjust shoulder pitch and yaw by up to 0.35 radians per axis. Preserve the ordinary swing when the gap between the fighters’ pelvis shapes is at most one fist width. Freeze the correction for the swing and fade it through recovery. Only qualifying physical contact can deal damage. See the [directional punch targeting slice](docs/slices/08c2-punch-targeting.md) for the geometry and verification.

Headbutts drive the neck and chest with bounded joint motors. The [implemented move](docs/slices/08c1-headbutts.md) spends six stamina on acceptance, winds up for six ticks, strikes for fourteen and returns to the ordinary pose over eighteen. A fresh press may start the next attack after those 38 ticks. Q triggers the move online and for Coral locally. Blue uses M on the shared keyboard. A head strike uses the same contact, closing-speed, damage-cap and per-target rules as a punch. Later kicks/dropkicks/dive attacks use leg/body motor targets and physical contact; missing an airborne attack can leave the attacker vulnerable. Define each move in data: eligible states, effort envelope, timing, damage multiplier, stamina cost and recovery pose.

The native two-fighter sparring fixture runs both complete character controllers before one world step. Each player owns its body and joint handles, hand timers, locomotion and balance state. A local fighter ID qualifies hand and body observations within the fixture. Both fighters can independently walk, jump and punch. Reset rebuilds both rigs and clears the shared contact history.

The [native Skyline simulation](docs/slices/15d-native-roster.md) accepts two through eight fighters. Each slot uses its authored spawn. All controllers share one solver step, strikes register once per victim, and hands can grip any eligible fighter. Round reconstruction preserves the roster and scores. A fresh match resets all scores.

The browser’s Local sparring scene exposes the same duel through WASM. Coral uses WASD and J/K or the mouse buttons. Blue uses arrows and comma/period. Each has a separate run and jump binding. A shared camera frames both pelvis positions. The panel counts qualifying hits separately and shows the most recent hit region and points.

### 13.4 Impact and damage

A proposed baseline approximation is:

```text
reduced_mass = (m1 × m2) / (m1 + m2)
impact_energy = 0.5 × reduced_mass × approaching_normal_speed²
damage = clamp(k × max(0, impact_energy − energy_threshold)
               × region_multiplier × attack_multiplier, 0, max_damage)
```

Use pre-solve point velocities, including `ω × r`, and clamp the normal component to approaching motion. For a fixed/kinematic environment, do not substitute its zero inverse mass as a literal zero mass in this formula; the appropriate two-body limiting approximation is the dynamic body's mass. Articulated limbs and rotational inertia make this only a gameplay proxy. A solver-impulse model is an alternative to validate, not an additional term to add blindly and double-count the same impact.

| Region | Proposed multiplier |
| --- | ---: |
| Head | 1.5 |
| Chest | 1.0 |
| Pelvis | 0.8 |
| Arms | 0.4 |
| Legs | 0.5 |

Tune `k`, threshold and cap using standardized impact fixtures; no values are represented as known Gang Beasts internals. Resting contacts, solver support impulses and self-contact must not repeatedly damage a character. Aggregate substep contacts, use minimum closing speed and per-attack/per-target registration, and apply cooldowns to environmental impacts. Separate hazard damage-over-time from impact damage. Props can hurt through actual collisions without needing an attack animation.

The native duel samples all limb velocities before the solve on active strike ticks. A positive solver contact impulse and a normal facing into the punch qualify a contact for evaluation. The local tuning requires at least 0.5 m/s closing speed and energy over 0.2 J. It awards five damage points per joule above threshold, applies the region multiplier and caps a single hit at 35 points. Within a tick, the strongest eligible region wins, with ties resolved in anatomical order. The first accepted hit consumes that punch for the opposing fighter. These fixture settings live in `config/impact.json`.

### 13.5 Knockouts

| Parameter | Proposed initial value |
| --- | ---: |
| Maximum health/consciousness | 100 |
| Regeneration delay after damage | 2 s =120 ticks |
| Regeneration rate | 8 points/s |
| First knockout duration | 3 s =180 ticks |
| Additional time per prior KO | 1 s =60 ticks |
| Maximum duration | 12 s =720 ticks |
| Health on recovery | 60 |
| Motor recovery ramp | 0.5 s =30 ticks |

For knockout number `n ≥ 1`, use `min(3 + (n−1), 12)` seconds initially. Reset the counter each round. Any later decay or anti-camping immunity is an explicit balance change, not a hidden assumption. Damage while down must not reset the timer every tick; keep total incapacitation bounded. Button mashing does not accelerate recovery in the MVP. Avoid large health bars by default; use body language, face state and clear knockout/recovery audio.

Local and online Skyline name each knocked-out or recovering fighter. The [fighter condition cues](docs/slices/17d-fighter-condition.md) announce fresh knockout and recovery transitions from the displayed simulation state, with a separate voice for hazard warnings. A joining or reconnecting player sees the current condition immediately and hears subsequent changes.

Local sparring applies the combined qualifying punch damage from a completed tick to each active fighter. Both down and wake durations have fixed deadlines. Punches against a downed or waking body still collide but award no further points. The 30-tick wake ramp limits internal motor torque before movement and new combat input resume. The body then completes any necessary supported get-up. [Timing, grip ownership and replay evidence](docs/slices/10b-consciousness.md).

### 13.6 Grabbing

A reaching hand may acquire an eligible physically reachable surface on actual contact or a tightly bounded contact-proximity rule validated against geometry. A broad sensor overlap alone does not authorize a long-range grip.

1. Resolve target entity/body and grabbability flags; exclude self, sensors and cosmetic-only objects.
2. Validate range, contact/visibility, consciousness, stamina and free hand.
3. Convert the world contact point into hand-local and target-local anchor coordinates.
4. Insert a temporary compliant constraint at a safe mutation boundary.
5. Record owner, hand, target entity/body, joint ID, local anchors, creation tick and configured grip strength.
6. Monitor load and stamina; remove once on release, break, knockout, elimination, disconnect or entity destruction.

Use a soft positional grip with bounded effort and permissive angular movement; do not default to an infinitely rigid weld. Multiple characters can form closed loops. Cap one outgoing constraint per hand, maintain reverse references to each target and test loops under stress. Fixed world surfaces may need a fixed anchor body representation so grip joints reference valid bodies.

The local opponent-grip fixture uses a 25 mm maximum gap between collider surfaces, checks intervening solids, and stores the midpoint in each body’s local frame. The three linear spring motors share a bounded force budget and leave relative rotation free. [Native implementation and verification](docs/slices/09a-opponent-grips.md).

Skyline uses compliant constraints and per-hand stamina accounting for crates, vents and exposed upper roof or walkway edges. Solid lips also limit slip and use the fall-arrest overload calibration in §13.7. Reach motors angle toward lower props during approach. Grips remain attached to a falling panel until release, failure or debris retirement. [Surface eligibility and verification](docs/slices/10c1-arena-grips.md).

Local and online Skyline identify each attached hand and its target. Private matches resolve the target from the displayed authoritative snapshot, including on reconnect. A free hand can land a qualifying punch while the other attachment persists. [Independent hand verification](docs/slices/09e-online-hand-actions.md#verification)

### 13.7 Stamina and grip failure

| Parameter | Proposed value |
| --- | ---: |
| Maximum stamina | 100 |
| One-hand base drain | 4 points/s total |
| Two-hand base drain | 7 points/s total, not 7 per hand |
| No-grip regeneration | 18 points/s |
| Additional load/climb drain | Configurable, nonnegative and bounded |

During play, the scoreboard shows the fighter’s stamina while a hand is attached or the stamina pool is refilling. At 25 points or less it displays “Low stamina.” Local Skyline reads the simulation directly. Private rooms use the same authoritative snapshot as the grip notices. [Stamina feedback](docs/slices/09f-stamina-feedback.md)

A baseline load term can be `load_rate × max(0, measured_load/reference_load − 1)`, summed or capped by documented policy. Base drain always applies while gripping, including when grounded. Repeated grab/release does not bypass drain or reset accumulated overload.

A grip breaks at zero stamina, excessive sustained load, owner knockout, invalid target or explicit release. Load units must be explicit: a solver impulse in N·s divided by its corresponding interval gives an average force in N; angular impulse needs separate handling. Aggregate substeps correctly, smooth short spikes and require a short overload window to avoid frame-dependent breakage.

The source playground's **250 N** grip threshold is illustrative. A 40 kg character weighs approximately **392.4 N**; one-hand hanging at 250 N would fail without assistance. Calibrate one-/two-hand hanging tests before selecting release thresholds. Do not silently weaken gravity to make an inconsistent grip preset work.

The local simulation samples each grip’s last solver-substep force at 60 Hz and smooths it over a 0.08 s time constant. Opponent, crate and vent grips accumulate overload at 650 N with an eight-tick break threshold. Subthreshold or released ticks reduce accumulated strain by one tick. Additional drain sums across attached hands and caps at 12 points/s per fighter. The acquisition tick incurs base drain. A failed grip needs a fresh hand press. [Calibration, sampling units and verified scope](docs/slices/09c-grip-stamina.md).

A Skyline lip permits up to 25 mm travel per linear axis before its constraint limit engages. The solver reaction from both the spring and the limit counts toward load. For an arrested fall the lip overload preset is 740 N for 72 accumulated ticks. One hand supports the 40 kg fixture and releases a sustained 80 kg load within 90 ticks. [Catch geometry](docs/slices/10c2-ledge-catch.md) and [fall-arrest timing](docs/slices/10e-ledge-rescue.md).

### 13.8 Lifting and throwing

Lift raises gripping-arm targets and applies bounded torso assistance. Opponent mass, posture, active resistance and environment contacts affect success. An awake opponent remains able to move, grab and strike with a free hand.

Release removes grips while preserving target linear/angular momentum. Optional release assistance is a small capped impulse aligned with the existing swing/intended direction. Apply it once per throw episode and target, not once for each hand. When only one hand releases, the remaining grip stays valid; avoid injecting the full two-hand throw bonus. Record the impulse in events/replays. Never guarantee an arbitrary travel distance or teleport the target outside an arena.

Local sparring uses E for Coral and slash for Blue. Each gripping arm can request up to 400 N of upward assistance on fixed or kinematic footing. The hand receives 75% of that budget and the lifter’s chest receives 25%. Losing footing disables the assistance while retaining internal arm targets. The initial release preset uses existing momentum without a bonus impulse. [Controller and test evidence](docs/slices/10a-lift-release.md).

During an opponent grip, private-room controls explain how to lift and release, or how to grab back. A defender who grabs the lifter can keep the physical attachment after the lifter lets go. A matched Skyline browser fixture produces an east-edge ring-out with lift and leaves both fighters on the roof with lift omitted. [Throw and counter-grip verification](docs/slices/10d-skyline-throws.md#verification)

### 13.9 Climbing

Basic sequence: grab a ledge → press/hold pull input → drive elbow/shoulder targets and bounded upward/inward assistance → align torso → clear the ledge → regain support.

Require a valid grip, remaining stamina and plausible ledge/surface geometry. A mantle needs pelvis clearance, obstacle clearance and a reachable support surface. Assistance cannot lift against an arbitrary ceiling indefinitely. Grip anchors move with their target bodies. Release only after support/clearance permits or the player chooses; do not automatically drop a recovering player into a fall.

MVP: simple Skyline edges, both one- and two-hand rescue cases. Full game: vertical surfaces, moving holds and alternating hand-over-hand climbing. Tests must include occupied ledges, low ceilings, broken panels, high latency and a second player hanging from the climber.

In local and private Skyline matches, a reaching hand can catch an exposed lip within anatomical reach as the floor falls away. The climber sees their pull-up key while holding a fixed roof edge. Holding jump drives a one- or two-hand mantle. The controller checks the approach and landing every assisted tick, caps one-hand and two-hand force at 760 N and 960 N, and releases the roof grips once the pelvis is inside and the trunk or legs bear weight. Native fixtures cover obstruction, panel failure and a hanging passenger. Two private-room clients observed a catch below the main deck and a return to play after the west walkway collapsed. Human evaluation and the full latency matrix remain open. [Catch geometry](docs/slices/10c2-ledge-catch.md), [mantle calibration](docs/slices/10c3-mantling.md) and [private-room rescue](docs/slices/10e-ledge-rescue.md).

## 14. Map engine and environment system

### 14.1 Versioned map definitions

Each map has an ID, schema/content version, simulation content hash, spawn points, static colliders, dynamic objects, mechanisms, hazards, elimination volumes, escalation schedule, camera profile and mode rules. Keep render assets separate from physics data; a headless server never needs a GLB mesh merely to load collision geometry.

```text
assets/maps/skyline/
  map.json
  scene.glb
  collisions.json
  thumbnail.webp
  audio/
```

Use JSON or RON in development, with a chosen canonical format per asset type. Validate and package at build time. Reject invalid dimensions/masses, missing IDs, cyclic unsupported definitions, out-of-bounds spawns, excessive objects and unknown actions. Check client/server content hashes before a player enters a round. Cosmetic-only asset updates may have a separate hash from simulation content.

### 14.2 Environmental primitives

| Primitive | Required behavior |
| --- | --- |
| Moving platform | Bounded trajectory and meaningful surface velocity |
| Rotating mechanism | Motor or prescribed rotation; limits and safe reset |
| Hinge | Physical pivot with optional motor/stops |
| Suspended platform | Cable-like constraints and support failure |
| Breakable joint/panel | Server-owned damage/load threshold, one detachment event |
| Force volume | Directional force and deterministic membership |
| Water volume | Buoyancy, drag, surface level, drowning policy |
| Damage volume | Tick-based damage, deduplicated per character |
| Elimination volume | Permanent lifecycle transition |
| Object spawner | Budget, cooldown, lifetime and valid spawn region |
| Interactable | Ownership/occupancy and server-validated actions |
| AI hazard | Telegraphed targeting using bounded physical actuators |

### 14.3 Structural destruction

Begin with authored segments and breakable connections. Server damage/load accumulation decides breakage. Detaching a segment preserves appropriate velocity and produces a reliable event with entity/joint IDs and tick. Large debris is authoritative; small fragments are visual-only and cannot block or hurt players.

Define per-map active-body/debris budgets and cleanup rules. Do not despawn a gripped platform under a player merely because a cosmetic lifetime expired. Pre-fractured glass and hierarchical structure belong in later content. Arbitrary runtime mesh fracturing is deferred.

### 14.4 Moving platforms

```text
surface_velocity = platform_linear_velocity + platform_angular_velocity × contact_offset
wanted_world_velocity = wanted_relative_movement + surface_velocity
```

Accelerate toward the relative target with a bounded controller. Apply the support contribution once; friction already transfers some motion. Kinematic platform updates must use the engine's kinematic motion interface so contacts see velocity, rather than discontinuous pose teleports. Crushing cases need explicit gameplay treatment.

The native idle locomotion controller additionally follows horizontal changes in support velocity through a bounded reference: advance the reference by at most `maximum_acceleration × tick_dt`, subtract motion already supplied by contacts and ordinary idle damping along that increment, and add only the remaining carry correction. The final combined command retains the existing acceleration/impulse budget; relative-motion damping, motor caps and balance caps remain unchanged. Active movement/braking already uses its fast support-relative response and adds no separate carry term. Reset, disable, support loss and insufficient upright posture clear the reference. Native rotation, cardinal start/stop, chest-impact and departure fixtures are documented in `docs/slices/06c1-moving-supports.md`; browser terrain and general terrain acceptance remain pending.

For railway/highway scenes, keep the simulated arena near the origin and move scenery/obstacles. Choose one consistent reference frame for all gameplay bodies. Decorative scenery does not supply physical velocity; actual moving obstacles do. If world rebasing is later required, rebase bodies, anchors, caches and network origins coherently.

### 14.5 Buoyancy

Approximate with submerged sample points: `buoyancy = fluid_density × gravity_magnitude × submerged_volume`, upward. Add depth-dependent linear/angular drag relative to water motion. Multiple samples create tilting torque. Bound per-body forces and define entry/exit transitions.

Water appearance is not a fluid simulation. Character survival uses explicit head-submersion/time rules rather than immediate elimination whenever any limb touches a water sensor. Floating ice and boats use the same tested buoyancy primitive. Only authored water states and authoritative bodies affect gameplay.

### 14.6 Hazard schedules

Represent events by trigger tick, stable event ID, action and optional repeat rule. Actions include mechanism activation, object spawn, joint break, wind change, creature activation and platform modification. Execute sorted by tick and stable ID; consume seeded RNG only in defined order.

Each map supplies warning timing and audiovisual event IDs. Late join/reconnect receives current hazard state rather than restarting warning timers locally. Timed warnings accompany sirens, lights, vibrations or motion cues. Reset schedules and random state completely between rounds.

## 15. Proposed original map roster

| Original concept | Mechanical focus | Complexity / release |
| --- | --- | --- |
| **Skyline** | Rooftop edges, vents, crates, failing perimeter | Low; MVP |
| **Scrapyard** | Conveyors, crushers, scrap and barrels | Medium |
| **Freightline** | Connected rail cars, tunnels, separating wagons | Medium |
| **Dockside** | Swinging cargo, suspended decks, failing supports | High |
| **Storm Rig** | Wind, rising water, collapsing bridges | High |
| **Carnival** | Rotating ride, swinging seats, detachable platforms | High |
| **Icebreaker** | Floating ice and progressive fragmentation | High |
| **Containment Lab** | Robotic articulated hazards and containment doors | Very high |

Additional concepts retained for a 12–20 map roster: an original boxing arena, rooftop helipad, airport baggage system, cargo ship, construction site and space station. These are concepts, not completed assets. Make layouts, silhouettes, colors, routes and props distinct from reference maps. Do not embed the earlier image-search results as licensed production assets.

### 15.1 MVP map: Skyline

| Property | Proposed setting |
| --- | --- |
| Playable dimensions | 24 × 20 m |
| Player limit / spawn points | 8 / 8 |
| Floor top | Y =0; collider thickness extends below it |
| Fall boundary | Character root/pelvis below Y =−12 m |
| Dynamic crates | 4 |
| Static ventilation units | 2 |
| Breakable edge panel groups | 2 |
| Escalation | Progressive edge-panel failure |
| Normal round duration | 180 s |

Place eight safe, nonoverlapping spawns with space to move and no immediate unavoidable elimination. Ensure crates cannot permanently seal every recovery route. Rails/panels must have visible collision and grab surfaces. Use a broad fall sensor plus explicit bounds checks; a hand dipping below the plane does not alone eliminate its owner.

Proposed escalation schedule: warning at 90 s, first perimeter group weakened/detached at 120 s, second group at 150 s, sudden death at 180 s. Sudden death uses an authored, telegraphed hazard or additional edge failure with a 30 s maximum extension; then draw if unresolved. These timings are new tuning proposals, not source-game constants. Reset crates, panels, grips, hazard timers and camera defaults every round.

The initial Skyline manifest uses a 20 × 20 m central deck and offset two-metre exterior walkways. Both walkways detach on the schedule above. At 180 s, a lateral gust ramps over three seconds to 9 m/s². Layout, mass, spawn and timing values come from one [versioned manifest](assets/maps/skyline/map.json). [Headless physics and verification](docs/slices/11a-skyline-physics.md).

## 16. Camera engine

Implement an independent TypeScript camera module at render frequency, consuming interpolated presentation state.

### 16.1 Camera model

| Parameter | Proposed value |
| --- | --- |
| Projection | Perspective |
| Vertical FOV | 45° |
| Downward angle | 35° |
| Default yaw | 45° |
| Minimum distance | 8 m |
| Maximum distance | Per-map framing bound |
| Position damping | 5 s⁻¹ |
| Zoom-in / zoom-out damping | 3 / 7 s⁻¹ |
| Orbit increment | 45° |
| Screen inset | 15% of full width/height from each edge |

Define the inset convention exactly: at 15% per side, the usable central fraction is 70%. Camera maximum distance is a design constraint, not justification for clipping active players; use an alternative view/framing mode if the arena cannot fit.

### 16.2 Group tracking

Weighted center: `center = Σ(weight_i × position_i) / Σ(weight_i)`. Give the local online player a modest framing priority; shared-screen active players have equal weight. Include full character bounds and important hazards/ball as appropriate, not only pelvis points.

Remove eliminated players. Continue framing a falling player while recovery remains possible, with a smooth reduction once they pass the unrecoverable region. Do not let debris or an already eliminated body pull the camera away. Handle zero tracked players with the map overview. Retarget smoothly when a player leaves.

Skyline weights the active online player at 1.35 and each peer at 1. Local shared-screen matches and spectators in group view use equal weights. Full limb bounding spheres must clear the inset planes at the current orbit and aspect ratio. [Framing geometry and verification](docs/slices/12e-skyline-framing.md)

### 16.3 Dynamic zoom

For a centered bounding sphere of radius `r`, a basic fit is `distance ≥ r / sin(limiting_FOV/2)`. Derive horizontal FOV from vertical FOV and aspect ratio. Account for the inset by using an effective half-angle `atan(usable_fraction × tan(half_FOV))`.

Production framing should project all relevant bounds into the intended camera orientation and solve distance/target offsets for horizontal and vertical screen limits. This handles elongated trains and height differences better than a sphere alone. Recompute on resize and device orientation/aspect changes. Zoom out promptly when players separate; ease inward to avoid breathing/pumping.

### 16.4 Smoothing

Use `alpha = 1 − exp(−lambda × frame_dt)` and interpolate toward the desired position/zoom. Smooth orbit separately with shortest-angle interpolation. Clamp or reset smoothing after long tab suspension instead of performing one enormous extrapolation step. Camera shake is presentation-only, small and optional; it must not alter control direction unpredictably.

### 16.5 Collision and occlusion

Use authored camera collision geometry and permitted orbit/bounds. Sphere-cast from target toward the desired position, then choose a clear nearer position or alternative map camera. If no local physics world exists online, provide Three.js-side camera query geometry or an explicitly scoped query implementation; do not require a full duplicate authoritative world merely for camera collision.

Use subtle silhouettes/markers or fading of designated foreground meshes when geometry partially hides a character. Do not make arbitrary gameplay obstacles invisible. Test overhead platforms, glass, walls, moving machinery and surviving players at opposite corners.

On Skyline, opaque scenery covers the ordinary suit and reveals a striped team-color silhouette of the concealed limbs. Exposed limbs retain their normal suit and lighting. The silhouette follows the same fifteen-body pose in local and online play and disappears with the fighter on elimination. [Skyline occlusion and verification](docs/slices/12f-skyline-occlusion.md)

### 16.6 Camera-relative input

Skyline uses Z/X to turn in 45° increments from its existing rooftop view. The two local fighters share an orbit, while each online player and spectator controls an independent view. Held direction follows the displayed heading through each turn. [Implementation and verification](docs/slices/12d-camera-orbit.md)

Flatten and normalize camera forward/right on the horizontal plane, combine input, clamp vector length to one, and quantize world X/Z. With a near-vertical camera, reuse a stable previous forward vector rather than normalizing a nearly zero projection. Use intended camera orbit orientation rather than transient shake.

The server receives world-space intent, not camera yaw or trusted speed. Rotating one client's camera cannot rotate another client's controls. Define whether a held direction follows camera orbit continuously; initial policy is yes, with smooth camera rotation and no discontinuous sign changes.

### 16.7 Spectating

After elimination, support group view, survivor cycling, allowed orbit changes and scoreboard. Spectating never generates gameplay input. When a target is eliminated, choose the next surviving target or map overview. Full-game local multiplayer must handle a mix of living and eliminated local players without obscuring the shared fight.

In private Skyline matches, eliminated fighters and spectators waiting to join can use **Previous fighter**, **Next fighter** and **Group view**. Following frames the selected fighter’s full body, including during knockout or ledge recovery. A fall advances the camera to the next survivor in roster order. An empty roof returns to the arena overview. Each new round starts with the default framing for the viewer’s seat. [Spectator camera](docs/slices/12h-spectator-camera.md)

## 17. Networking architecture

### 17.1 Authoritative model

One native server world per room decides all gameplay. No peer host authority, naive lockstep or client-reported hit outcomes. Different hardware and network conditions make a physics chain particularly sensitive to divergence.

### 17.2 Timing and messages

Clients sample intentions and send monotonic sequence numbers, advisory client ticks, held state and action edges. The room chooses when accepted input applies. Client timestamps are not permission to rewind hits or accelerate simulation. Snapshots identify the authoritative completed tick and the last input actually applied for each receiving player.

Estimate server time with application-level ping/pong and a smoothed offset/RTT. Use a monotonic browser clock. Keep buffering and clock adjustments bounded. Wire `u32` ticks/sequences use modular comparison with a half-range validity window; internal room ticks may remain `u64`.

### 17.3 Transport

MVP: binary secure WebSockets. Use one ordered connection per session with application message kinds for control, inputs, snapshots and events. TCP loss can delay newer state through head-of-line blocking; application dropping cannot remove bytes already handed to TCP.

Each native room publishes one shared latest snapshot and wakes its subscribed match sockets. A subscriber can skip superseded frames under backpressure. A new subscriber can immediately read the current state. Heartbeat and ownership checks also advance when publications pause.

Keep a `Transport` interface for open/send/receive/close and measured queue status. Evaluate WebTransport or WebRTC data channels later only with real latency/loss tests and deployment/browser compatibility checks. Unreliable snapshots need explicit lifecycle/baseline recovery. Raw UDP is not the browser baseline. The browser WebSocket API has no automatic receive backpressure; bound application work and monitor queued sends. [MDN WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)

### 17.4 Input wire contract — exactly 18 bytes

Use little-endian multibyte integers and explicit encoding/decoding. Never serialize Rust struct memory, padding or enum layout directly.

| Byte offset | Field | Type | Bytes |
| ---: | --- | --- | ---: |
| 0 | Message type | u8 | 1 |
| 1 | Protocol version | u8 | 1 |
| 2 | Input sequence | u32 LE | 4 |
| 6 | Advisory client simulation tick | u32 LE | 4 |
| 10 | World movement X | i8 | 1 |
| 11 | World movement Z | i8 | 1 |
| 12 | Held buttons | u16 LE | 2 |
| 14 | Pressed buttons | u16 LE | 2 |
| 16 | Released buttons | u16 LE | 2 |
| | **Total** | | **18** |

The implemented input kind is 0x01 and the protocol version is 0x01. A lone edge must agree with the final held state. The browser sampler preserves up to four ordered records per sample interval. [Codec and conformance evidence](docs/protocol.md).

Arithmetic: `1+1+4+4+1+1+2+2+2 = 18`. The final field ends at byte 18 exclusive. No additional game-message header is prepended to this record in protocol v1. WebSocket framing, masking, TLS and network overhead are additional. At 60 records/s this is **1,080 application bytes/s per client** before overhead.

Movement components encode `[-127,127]`; reject −128, dequantize, and clamp vector magnitude to one so diagonals do not move faster. Reserve button bits: 0 jump, 1 run, 2 left hand, 3 right hand, 4 lift, 5 headbutt, 6 kick, 7 crouch, 8 emote, 9 interact; bits 10–15 must be zero in v1. Future actions require compatible version negotiation. Camera/menu/scoreboard inputs are local.

Pressed/released bits preserve short taps between samples. If both occur in one frame and the final held bit is clear, interpret one press then release; final held set means release then press. At most one activation per button per input frame. Preserve rapid transitions as ordered records when needed, with a bounded burst allowance; do not collapse multiple input records into just the latest held state. The server limits cooldown activations regardless of record rate. Reject duplicates and implausibly distant sequence jumps; an input acknowledgement means applied, not merely received.

Clear stale input after an initial proposed 250 ms without accepted frames. This is separate from a disconnect grace period. Never let a stalled socket leave an infinite held punch/grab.

### 17.5 Snapshots, events and baselines

MVP full snapshots are preferable to premature compression. Use stable application IDs and explicit field encoders. Define a v1 snapshot header, extending the source's conceptual fields:

| Field | Type | Meaning |
| --- | --- | --- |
| Kind / version | u8 / u8 | Snapshot record type and protocol |
| Flags | u16 | Full/delta, velocity presence, supported extensions |
| Server tick | u32 | Completed authoritative tick |
| Applied input acknowledgement | u32 | Recipient's last applied input; validity in flags |
| Baseline tick | u32 | Delta base, ignored for full snapshot |
| Entity-record count | u16 | Number of bounded following records |
| Gameplay payload bytes | u16 | Length of separately versioned state block |

The v1 full Skyline record uses kind 2. Its [wire layout and golden fixture](docs/protocol.md#full-skyline-snapshots) specify the fighter, round, arena and transform fields, reference validation and full-state replacement semantics. The header is **20 bytes**. It is distinct from the 18-byte input record. Define all remaining record layouts before implementation and lock them with golden fixtures; do not infer a wire format from illustrative Rust types.

Initial transform record proposal: stable body ID `u32`, position `3×f32`, quaternion `4×f32` = **32 bytes**. Body identity is stable across snapshots within a round and mapped to character/body-part at spawn. Reject nonfinite values; normalize or reject invalid rotations. Optional velocity fields add bytes explicitly.

Snapshots also carry consciousness, stamina/health as needed for presentation, grips, relevant mechanism states, round/epoch ID, scores and hazard progression. Reliable creation/destruction events include dependencies and generations; a full resync reconstructs all lifecycle state. Events have unique IDs so audio/effects and round scores are not replayed twice.

Skyline clients advertising `compact=1` accept a 23-byte full transform with the original u32 ID, three exact f32 positions and a seven-byte smallest-three quaternion. Header bit 2 selects this encoding. [Wire contract and rotation tolerance](docs/protocol.md#full-skyline-snapshots). Both layouts support independent full-state recovery. Further candidate optimizations include position and velocity quantization, changed-entity masks, deltas and periodic keyframes. A candidate **20-byte transform** is body ID 4 + position 6 + quaternion 7 + flags 1 + reserved 2. Position uses map-defined origin and 1 cm signed-16-bit units (about ±327.67 m); values outside range require an escape/full record or explicit map rejection, never silent overflow. Quaternion reconstruction needs a largest-component index, sign convention and normalized decode. Specify tolerances and fuzz the codec.

Input ACK does **not** acknowledge a snapshot baseline. Send a separate snapshot-applied ACK/control record. Delta only against a retained baseline the client has explicitly applied. On missing baseline, request full resync and stop applying dependent deltas. A dropped unsent snapshot cannot remain the dependency of the next queued snapshot. Initial proposed full-keyframe interval: 1 second, plus join/reconnect/resync.

### 17.6 Bandwidth arithmetic

Decimal units: 1 KB =1,000 bytes; 1 Mbps =1,000,000 bits/s; 1 GB =1,000,000,000 bytes.

| Scenario at 30 snapshots/s | Per-client transform payload | Eight-client room payload |
| --- | ---: | ---: |
| 120 character bodies ×20 B | 72,000 B/s =72 KB/s | 576 KB/s =4.608 Mbps |
| 120 character +24 environmental bodies ×20 B | 86,400 B/s =86.4 KB/s | 691.2 KB/s =5.5296 Mbps |
| Same 144 bodies ×32 B uncompressed | 138,240 B/s =138.24 KB/s | 1.10592 MB/s =8.84736 Mbps |

The optimized 144-body example produces `691,200 × 3,600 = 2,488,320,000` bytes ≈**2.49 GB room egress/hour**. All figures exclude headers, lifecycle/gameplay events, keyframes, velocities, transport overhead and retries. Spectators add recipients. The ≤100 KB/s target is for a typical optimized application stream, not an achievable guarantee for the uncompressed MVP prototype.

Reusable estimator: `B = (15P + E) × record_bytes × snapshot_rate × changed_fraction`; room egress is `B × recipients`. Delta savings depend on actual motion and acknowledgement behavior. Do not claim a compressed ratio before measuring a recorded fight.

### 17.7 Interpolation

Maintain snapshots keyed by round epoch and server tick. Render at estimated server time minus a **67–100 ms initial interpolation delay**. Interpolate positions linearly and orientations with shortest-path SLERP. Use consistent time for all limbs so a character does not tear apart across mixed ticks.

The [initial browser implementation](docs/slices/15c-interpolation.md) uses one delayed tick for every limb and arena object and checks bounded extrapolation, stall recovery and epoch discontinuities. Adapt buffering to measured jitter with bounded changes. On missing future state, extrapolate for a short capped interval (initial candidate 100 ms), then freeze/fade appropriately and show connection status. Do not allow indefinite extrapolation. Spawn, deletion, teleport/reset and round boundaries are discontinuities, not poses to blend through.

### 17.8 Local prediction and fidelity gate

| Phase | Strategy |
| --- | --- |
| Initial network prototype | Authoritative interpolation for all bodies |
| MVP polish | Immediate reversible feedback and carefully limited local pose anticipation |
| V1 | Uncoupled local locomotion prediction |
| Advanced | Contact/constraint island prediction and reconciliation |
| Experimental | Whole-world rollback only if justified |

Interpolation-only controls may incur one-way input delay, simulation scheduling and interpolation buffering before visible response. At 60 ms RTT, 67–100 ms buffer and one tick of scheduling, visible response can exceed 100 ms even before render delay. This is a planning estimate, not a measured latency result. If friend playtests find it sluggish, responsiveness work is required before declaring the MVP complete.

A grabbed opponent, stacked prop or contact chain couples local and remote dynamics. Replaying only the local pelvis is invalid. Prediction checkpoints need velocities, controller state, timers, grips, RNG and relevant solver/world state, not just transforms. Expand islands when new contacts occur; when the necessary state is unavailable, fall back safely to authority.

Reconcile actual simulated state from an authoritative checkpoint, replay still-unacknowledged inputs, and smooth only the rendered correction within limits. Never let a visual correction create an authoritative hit. Suppress duplicate predicted sound/events. Do not rewind server hits merely because the client supplies an old tick.

Rapier offers determinism under documented conditions, including feature and initialization requirements. Exact compatibility of enhanced determinism, SIMD and parallel features depends on the pinned release. Stable iteration/event ordering, seeded RNG and reproducible math remain application responsibilities. Validate native/WASM replay parity; snapshot authority does not require parity to function. [Rapier determinism](https://rapier.rs/docs/user_guides/rust/determinism/)

### 17.9 Backpressure

Use bounded per-client outgoing queues with separate control/event and replaceable snapshot handling. Replace superseded snapshots only before socket write; preserve lifecycle ordering and reconstruct a valid delta chain. Disconnect or reduce snapshot rate for persistently slow receivers. Never block the room on one client or grow queues without limit.

Initial proposed limits: two unsent snapshot slots, a bounded reliable queue, and a 256 KiB application send high-water mark. Measure actual socket buffering; values are operational candidates. If critical events cannot be delivered within bounds, close the connection with a recoverable reason and resync on return. Decreasing snapshot rate must not alter authoritative tick frequency.

## 18. Server architecture

### 18.1 Room ownership

A room contains its ID, players/sessions, one `Simulation`, match state, bounded input buffers, tick counter, seed and settings. Exactly one simulation owner mutates the physics world. Network handlers validate/decode and enqueue messages; they do not reach into Rapier bodies.

The [headless Skyline room](docs/slices/15e-worker-roster.md) accepts two through eight fighters and up to eight pending input records per fighter and forward sequence gaps of up to 120. The worker consumes pending records in slot and sequence order before stepping once. Input expires after 15 completed room ticks. Round reconstruction clears the input sequence and acknowledgement namespace. The transport must bind each sender to its fighter and synchronized round and fence old connections. [Room input and verification](docs/slices/14a-room-input.md).

Use stable player/session IDs distinct from connection IDs. A reconnect attaches a new connection to the same eligible session without creating a second authoritative character.

### 18.2 Concurrency

Run CPU-bound simulation on dedicated workers, separate from Tokio's network executor. Initially assign one or several rooms per worker based on measurements. Scale with a fixed room-worker pool; do not spawn an unbounded thread per incoming room. Never hold a global room-directory lock during a physics step.

The first [native worker pool](docs/slices/14c-worker.md) reserves one room per prestarted worker, retains a single latest full snapshot and suspends admission while a worker is behind by at least 60 ticks. Room-level parallelism is the first scaling strategy. Internal parallel physics must earn its overhead in small-world benchmarks. All queues have limits and clear overload behavior. A worker owns room ordering and never concurrently steps the same room twice.

### 18.3 Fixed-timestep pipeline

The authoritative gameplay tick sequence is:

1. Consume bounded, validated inputs in stable order and expire stale held state.
2. Apply lifecycle transitions and scheduled hazard/mechanism actions.
3. Refresh support/contact information available for controllers.
4. Update locomotion, balance, attack targets, grips and continuous forces.
5. Step physics using the configured explicit substeps; apply one-shot impulses once.
6. Collect contacts and aggregate per-tick impacts, grip candidates and loads.
7. Resolve damage/consciousness, new grips, failed joints and entity cleanup in a defined order.
8. Evaluate all eliminations for the tick, then decide the round result.
9. Finalize events and publish a snapshot every two ticks.
10. Increment/commit the tick according to one documented indexing convention.

Adopt convention: state starts at tick 0; `step()` advances to tick 1; events/snapshots label the completed state. Do not mix pre-step and post-step tick labels. Collect mutations during callbacks and apply them outside physics iteration.

Use a monotonic scheduler and fixed accumulator. Allow at most four catch-up ticks per scheduling pass initially, then yield and record overload; never enlarge `dt` to catch up. Persistent overload stops new room admission and triggers alerts/draining. Record any deliberate lag/drop policy; do not silently skip gameplay input while claiming a continuous replay.

### 18.4 Match state machine and reset

`Lobby → Loading → Countdown → Playing → RoundResults → Loading` for another round; after the match-winning score, `MatchResults → Lobby` or rematch. `Draining/Closed` are lifecycle states, not game modes.

After online MatchResults, the current host can [return everyone to the same lobby for a rematch](docs/slices/16f-online-rematch.md). The room retains its code and seats. Starting again requires fresh readiness and uses a new match serial in the network round epoch.

| Transition | Guard / action |
| --- | --- |
| Lobby → Loading | Host request, at least two ready players, valid settings/content |
| Loading → Countdown | Required players loaded compatible map; timeout policy applied |
| Countdown → Playing | Scheduled start tick reached |
| Playing → RoundResults | Winner/draw determined after all tick eliminations |
| RoundResults → Loading | Next-round delay elapsed and enough players remain |
| RoundResults → MatchResults | Win threshold reached |
| Any → Closed | Empty expiry, controlled shutdown or unrecoverable room failure |

Rebuild or fully reset bodies, joints, grips, props, hazards, health/stamina, KO counts, timers, RNG, input edges, event sequence state and presentation epoch. Scores persist only within the current match. Never carry late packets or old baselines into the new round. A deterministic round seed derives from match seed and round number using a documented method.

### 18.5 Elimination

Use sensor colliders plus explicit root/bounds checks. A fall requires the configured root/pelvis threshold, not any single limb's contact. Other hazards may define head submersion, damage or full-body removal conditions.

Elimination cancels input, motors and all affected gameplay grips. It is permanent for the round. Process simultaneous eliminations together: one survivor wins, none means draw. Do not award the first processed player a win before processing the rest. Bound post-elimination debris/corpse simulation and exclude it from winner logic.

## 19. Lobby and session management

### 19.1 User flow

Open site → create private room → share link/code → friends join → choose name/color → ready → host starts → play rounds → results → rematch. No required account, installation, command line or exposed implementation settings.

The site root opens the [Skyline Melee main menu](docs/slices/16k-game-entry.md). Hosting or joining opens the private-room form. Local match opens `/play/`, and the simulation inspector is available at `/playground/`. The menu uses static HTML and an original in-game still, so visitors can choose a destination before loading a simulation or opening a room session.

The [private lobby](docs/slices/16b-browser-lobby.md) lives at `/rooms/`. Its share link carries the room code in `?code=`. Opening an invite while seated preserves the new destination until the player chooses to leave their current room.

Show actionable states for unavailable server, full room, wrong code, incompatible version, loading timeout and reconnect failure. A browser tab becoming hidden is not permission to pause everyone else's game.

### 19.2 Room configuration and identity

Settings: maximum players, rounds to win, normal round duration, map rotation, mode, friendly fire and visibility. MVP defaults: 8 slots, first to 3 wins, 180-second rounds, Skyline, melee, private visibility. Validate ranges server-side; do not accept arbitrary map paths or unchecked enum values.

Use six-character human-readable random room codes with collision checks and rate limits. Room codes are locators, **not authentication secrets**. Issue cryptographically random session credentials separately (at least 128 bits of entropy). The share link contains room identity, never another player's session credential.

### 19.3 API contract

| Method | Endpoint | Purpose / access |
| --- | --- | --- |
| GET | `/healthz` | Process liveness; no secrets |
| GET | `/readyz` | Admission/readiness status |
| GET | `/api/maps` | Compatible map IDs, versions, display metadata |
| POST | `/api/session` | Bootstrap an anonymous same-origin session cookie |
| POST | `/api/rooms` | Validated create request; returns room/session info |
| POST | `/api/rooms/:code/join` | Join or obtain new session after validation |
| GET | `/api/rooms/:code` | Minimal allowed metadata; never session secrets |
| GET | `/api/rooms/:code/lobby` | Member-only roster, capacity and viewer slot |
| POST | `/api/rooms/:code/leave` | Release the authenticated seat and transfer lobby host |
| WS upgrade | `/ws` | Authenticated/versioned gameplay session |
| GET | `/metrics` | Internal/protected monitoring endpoint |

The [native service](docs/server.md) implements liveness, worker admission readiness and a compatibility catalog built from the bundled engine and Skyline map. Use bounded JSON for low-rate HTTP control requests; binary protocol for realtime gameplay. Standard error codes include invalid input, unavailable/full room, expired session, incompatible version and rate limit. Create requests need request IDs/idempotency handling so an HTTP retry does not silently create duplicate rooms.

Browsers use a Secure, HttpOnly, SameSite=Strict session cookie and explicit Origin validation. The [admission contract](docs/slices/16a-admission.md) defines room codes, retry semantics, quotas and expiry. WebSocket attachment must validate the same session, allowed Origin and room membership. Do not place reusable secrets in URLs/access logs. Unauthenticated sockets get a short handshake timeout and tiny resource allowance. Reconnect credentials are rotated/expired according to one documented policy.

### 19.4 Message catalog

| Client → server | Server → client |
| --- | --- |
| Connect | Welcome |
| JoinRoom / attach ticket | RoomState |
| SelectColor | PlayerJoined |
| SetReady | PlayerLeft |
| UpdateSettings | SettingsUpdated |
| StartMatch | MatchStarting |
| MapLoaded | RoundStarted |
| InputFrame | StateSnapshot |
| Ping | Pong |
| Rematch | RoundEnded |
| LeaveRoom | MatchEnded |
| RequestResync | FullSnapshot |
| SnapshotAppliedAck | GameEvent / Error |

The [lobby control contract](docs/protocol.md#lobby-controls) defines Connect, Welcome, SetReady and RoomState on the authenticated socket. Readiness requires a [map and engine compatibility acknowledgement](docs/slices/16d-map-loading.md) for the current connection. The browser loads the online arena renderer before sending that acknowledgement request, so every ready player already has the code needed to enter the match. Readiness resets on connection replacement or disconnect. A roster revision binds the ready mask to its player identities. The [online input binding](docs/slices/15b-online-input.md) associates each 18-byte control with its socket owner and last acknowledged round. The [online startup implementation](docs/slices/15f-online-roster.md) reserves a worker outside the lobby lock and revalidates every occupied seat before starting. The match maps the two through eight ready sessions to contiguous fighter indices in ascending lobby-slot order.

Prefer HTTP room creation/join followed by authenticated attachment; do not implement two conflicting join authorities. Every control message has type/version, exact length rules, authorization, allowed room states and a size cap. Input frames are exactly 18 bytes. Proposed other caps: 4 KiB control payload, 256 KiB snapshot/resync payload with bounded/chunked loading if necessary. Validate entity counts and lengths before allocating.

### 19.5 Disconnect and reconnect

Keep a disconnected slot for **20 seconds** initially. Immediately neutralize controls and release the disconnected player's outgoing grips; the body remains physically present and vulnerable. Other players' incoming grips can remain until ordinary break/elimination cleanup. On grace expiry, eliminate the character and release all affected constraints.

The [disconnect implementation](docs/slices/16e-disconnect-grace.md) uses monotonic server time, reserves a seat until its deadline, and publishes expirations together for the worker to score. Explicit leave forfeits immediately. With fewer than two eligible competitors after the results interval, return remaining members to the lobby. A service timer reclaims empty rooms even when no further HTTP requests arrive.

The browser [automatically retries an interrupted match](docs/slices/16g-automatic-reconnect.md) for up to 20 seconds from a detected interruption. During matches, eight seconds without an advancing full snapshot triggers browser recovery. The server probes the WebSocket every two seconds after a reply and allows six seconds for the next reply. [Silent-outage detection and verification](docs/slices/16h-silent-match-outages.md) A retry can reclaim only the disconnected seat last controlled by that tab. An intentional new connection can replace its predecessor. Each restored fighter receives current full state and acknowledges the epoch before sending fresh controls. No rewind, respawn or score duplication. A late joiner who has loaded compatible content spectates until the next playable round, then enters with a fresh fighter. Leaving or disconnecting while waiting removes the pending entry. [Private-room promotion](docs/slices/16j-spectator-promotion.md) At the final results tick of a nonterminal round, the authoritative room can rebuild the lineup with returning players and new entrants. Scores follow the returning player across fighter indices, each entrant starts at zero and all controls require the fresh round epoch. [Native round lineup](docs/slices/16i-round-lineups.md) Transfer lobby-host privileges to a stable eligible player on host departure; the host is never simulation authority. Destroy empty rooms after a proposed 60-second inactivity interval. Grace expiry and room expiry use server-controlled time.

## 20. Rendering engine

### 20.1 Ownership and loop

Authoritative transforms → snapshot buffer → interpolated render state → bone transforms → skinned mesh → GPU. Use an imperative `requestAnimationFrame` loop; React manages menus and low-rate state rather than every body transform.

Keep render object IDs synchronized with entity generations. Scene teardown disposes geometry/material/texture/audio resources according to ownership, while shared assets use reference counting or a cache. Handle WebGL context loss with a clear restore/reload path.

### 20.2 Character skinning

Map each physical body world transform through a fixed body-to-bone offset, then derive the bone-local transform relative to its current parent. Respect the GLB bind pose and mesh root transform. Do not assign world rotations directly to local bone rotations. Test asymmetric poses, full rotations and character scale.

Use one original rounded skinned mesh per character, shared geometry/material variants, expressive faces and optional cosmetic squash. A skeleton can have additional visual bones, but their procedural movement cannot alter authoritative collisions. Three.js supplies skinned-mesh/skeleton facilities; our physical mapping is custom. [Three.js documentation](https://threejs.org/docs/)

Skyline renders the [Roof Crew fighter](docs/slices/17b-skinned-fighters.md) from a shared original GLB. Each local or online player has an independent skeleton and kit palette. Diagnostic scenes retain individual collider meshes.

### 20.3 Art direction

Rounded exaggerated characters, distinct color/pattern identity, readable large props, simple expressive faces, original costumes and limited combat-area clutter. Prefer stylized visual clarity over realism. Preserve shape readability while a character is lying down, tangled or silhouetted against the background. Include non-color identity cues for accessibility.

### 20.4 Lighting and materials

MVP: one primary directional light, ambient/hemisphere fill, bounded shadow maps and baked static detail. Share materials and atlases. Later: optional ambient occlusion, bloom, reflections, water shaders and atmosphere. Expensive effects must be disabled on low presets without hiding gameplay warnings.

Skyline selects a lower render resolution and a 512-pixel basic shadow map on recognized software renderers. The match HUD uses the first accepted authoritative snapshot so round, score and hazard information can appear before the 3D scene initializes. Both paths use the same simulation timing and controls.

### 20.5 Optimization

| Technique | Scope |
| --- | --- |
| Instancing | Repeated scenery/props with compatible materials |
| Shared geometry/materials | Character variants and common objects |
| Frustum culling | Offscreen renderables, never authoritative simulation |
| LOD | Background detail; preserve combat silhouettes |
| Texture compression | Downloads and GPU memory |
| Object pools | Particles and temporary visual effects |
| Baked lighting | Static scene detail |
| Dynamic resolution | Frame-time control |
| Batched transform transfer | Local WASM simulation buffers |
| GPU skinning | Smooth character mesh over physical bodies |

Budget draw calls, triangles, texture memory and shadow casters per map after measuring the baseline. Do not choose aggressive culling that hides an offscreen hazard warning. Avoid per-bone JS/WASM calls and per-frame object allocation.

### 20.6 Presets

| Setting | Low | Medium | High |
| --- | --- | --- | --- |
| Render scale | 75% | 100% | 100% |
| Device pixel ratio cap | 1.0 | 1.5 | 2.0 |
| Shadows | Basic | Medium | High |
| Ambient occlusion | Off | Optional | On if budget permits |
| Reflections | Off | Basic | Enhanced |
| Particles | Reduced | Normal | Full |
| Frame cap | 60 | 60 | Configurable |

Effective resolution combines CSS dimensions, capped pixel ratio and render scale; document this to avoid accidental double supersampling. Server physics and collision shapes do not depend on client graphics settings. Support reduced motion and disabling camera shake independently of quality.

## 21. Audio system

### 21.1 Categories

| Category | Examples |
| --- | --- |
| Character | Grunts, jumps, stumbling |
| Combat | Punch, headbutt, kick |
| Contact | Footsteps, body/floor impact |
| Grip | Acquisition, stress and release |
| Environment | Wind, machinery, engines |
| Destruction | Wood, cable, glass |
| Hazards | Siren, incoming vehicle, creature warning |
| UI | Ready, countdown, results |
| Music | Lobby and arena tracks; adaptive layers later |

Use original/licensed sounds and music. Maintain provenance and volume normalization metadata. The reference's historical hazard/audio work motivates readable cues, not copied sound assets. [Aquarium update](https://gangbeasts.game/posts/2020/12/8/gang-beasts-aquarium-update)

### 21.2 Physics-triggered audio

Generate an impact event only above material/intensity thresholds. Volume/pitch may depend on contact speed/impulse, but use capped variation. Enforce per-entity cooldowns, category voice budgets and priority so a character pile does not produce hundreds of simultaneous sounds. Warning cues outrank low-value contact noise.

Unique event IDs prevent duplicate playback after resync/prediction. Discard stale one-shot events after reconnect instead of replaying the entire missed fight. Reconstruct ongoing loops from current mechanism state. Cosmetic local prediction may play low-cost anticipation sounds; confirmed impact sounds follow authority.

### 21.3 Spatial mix

Use Web Audio spatial position and attenuation with a clearly chosen listener policy aligned to the game camera. Avoid making crucial warnings inaudible merely because the group camera zooms out. Unlock audio on an explicit player interaction and handle suspended contexts on tab return.

Skyline’s [hazard audio](docs/slices/17c-hazard-audio.md) uses four original synthesized cues, centered and independent of camera distance. Each live stage transition can play once per round. The initial snapshot, reconnect and return from a pause establish the current stage silently. Local and online play share a persistent mute preference and retain their visual warnings.

Skyline plays an original short strike thud for an observed health loss of at least one point while the defender remains active. The local physics world or the server owns that health change. The sound follows the defender’s rendered pelvis in the current view. Simultaneous contacts share one impact voice, with a 120 ms cooldown per defender. Hazard and consciousness announcements take priority. [Strike feedback](docs/slices/17e-strike-audio.md)

Provide master, music and effects sliders, mute and persistence of settings. Full game adds adaptive music and refined occlusion. Visual warning alternatives must remain available for players who mute sound.

## 22. Game modes

### 22.1 Melee — MVP

2–8 players, individual survival, last survivor wins, default first to three round wins. Draws award no point. Process all eliminations before scoring; award each round once using a unique round ID. Use automatic next rounds, configurable map rotation and rematch without refresh.

The local and private-room result banners name the winner of the round or match. A drawn round explains that no point was awarded. [Scoring verification](docs/verification/round-scoring.md)

In local Skyline, the three-second countdown and five-second round results use visible, unpaused time. Their fixed ticks may catch up to the current phase boundary. Active fighter physics retains its four-step catch-up budget. [Local phase timing](docs/slices/12c-local-phase-clock.md)

At 180 seconds, begin the map's sudden-death sequence. Initial maximum extension is 30 seconds; unresolved rounds become draws. If fewer than two eligible players remain between rounds, return to a waiting lobby instead of farming automatic wins. Exact countdown/result display durations are UI configuration candidates (initially 3 s/5 s).

### 22.2 Teams

Two or more teams, team identity/spawns, configurable friendly-fire policy and victory when only one team has eligible survivors. Define friendly fire separately for damage, physical collisions and grabbing: disabling damage must not silently make teammates nonphysical. No surviving teams means draw. Team assignment/changes are server-authorized between rounds.

### 22.3 Waves

Cooperative players fight AI-controlled ragdolls through the same input/controller interface. AI cannot teleport, bypass cooldowns or set outcomes. Archetypes: Standard (approach/punch), Heavy (slower, stronger configured body/grip), Agile (fast/evasive), Grappler (grab/throw), Boss (authored physical abilities).

Use finite-state/utility behavior: choose a reachable safe target, approach, fight, grab, avoid hazard, recover/climb. Evaluate AI at a bounded lower decision rate while actuators still run at physics rate. Navigation must account for platform connectivity and destruction; a static navmesh alone is insufficient on detached moving platforms. Waves have spawn budgets, maximum live enemies, difficulty progression, victory/failure and round-reset rules. No machine learning dependency is needed.

### 22.4 Football

Two teams, physical ball, goals, score, regulation time and overtime. Detect authoritative goal crossing/valid overlap using a dedicated goal sensor collider and duplicate-event protection. At a goal, suspend scoring, reset ball/players as designed and restart on a server tick. Include the ball in camera framing. Define own-goals, out-of-bounds ball reset and simultaneous goal handling before release. Ordinary physical contact/grabbing/kicking remains available under mode rules.

### 22.5 Additional modes and rule interface

Future candidates: King of the Hill, Hot Potato, Capture the Object, survival challenges and physics modifiers. Modes own scoring, participation, time limits and round results; they do not duplicate the physics engine.

Conceptual interface: `on_round_start(context)`, `on_tick(context)`, `on_event(event, context)` and `round_result()`. Mode-requested mutations become validated commands applied at tick boundaries, avoiding unsafe reentrant changes during physics callbacks. The engine produces events regardless of presentation.

## 23. Customization and player progression

MVP: bounded display name and distinct color. Full game: original hats, shirts, masks, costumes and accessories. Cosmetic selection must not change authoritative mass, reach, collision, grip strength or visibility unfairly. Optional dangling accessories are visual-only with a bounded local budget unless explicitly introduced as gameplay objects for everyone.

Keep readable player identifiers in tangled fights and offer patterns/icons for similar colors. Save local preferences first. Optional accounts later support loadouts, statistics, unlocks and friend lists; they do not become prerequisites for private guest play. Persistent writes run outside the physics tick. No monetization/progression system is required to validate the game.

## 24. User interface

### 24.1 Screens

| Screen | MVP | Full game |
| --- | --- | --- |
| Main menu, create/join room | Yes | Yes |
| Lobby, ready state, name/color | Yes | Yes |
| Controls guide / practice entry | Yes | Yes |
| In-game HUD / scoreboard | Yes | Yes |
| Round and match results | Yes | Yes |
| Graphics/audio settings | Yes | Yes |
| Loading / reconnect / errors | Yes | Yes |
| Costume editor | No | Yes |
| Public matchmaking | No | Yes |
| Profile/statistics | No | Optional accounts |
| Replay viewer | Debug only | User-facing |

### 24.2 HUD and accessibility

Show round timer, win markers, player identity and concise connection status. Keep large health bars off by default. Network/physics diagnostics are an optional debug overlay, not normal player UI. Explain disabled start/ready controls in player terms: waiting for players, map loading or incompatible content.

Support keyboard navigation, visible focus, adjustable text size, colorblind-safe identity, reduced motion/shake and separate volume controls. Full control remapping and gamepads are required for the full release; structure actions independently of bindings from the outset. Persist settings locally with schema migration. UI overlays must not leak held input back into gameplay. No client-side pause freezes an online room.

# Part III — Engineering implementation

## 25. Repository structure

```text
physics-brawler/
├── TECHNICAL_SPEC.md              # Canonical specification
├── README.md
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── crates/
│   ├── engine/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── simulation.rs
│   │       ├── config.rs
│   │       ├── entity.rs
│   │       ├── physics/          # world, bodies, joints, collision, materials, queries
│   │       ├── character/        # skeleton, ragdoll, motors, balance, locomotion, gait, recovery
│   │       ├── combat/           # attacks, targeting, damage, knockout, grab, stamina, throwing, climbing
│   │       ├── environment/      # map, loader, hazards, mechanisms, destruction, buoyancy, escalation
│   │       ├── modes/            # melee, teams, waves, football
│   │       └── ai/               # behavior and navigation, when needed
│   ├── protocol/
│   │   └── src/                  # messages, input, snapshot, encoding, quantization, validation
│   ├── server/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── api/
│   │       ├── websocket/
│   │       ├── sessions/
│   │       ├── rooms/
│   │       ├── scheduler/
│   │       └── metrics/
│   ├── wasm/
│   │   └── src/                  # lib, bindings, buffers
│   └── benchmark/
│       ├── benches/
│       └── scenarios/
├── web/
│   ├── package.json
│   ├── package-lock.json
│   ├── vite.config.ts
│   └── src/
│       ├── app/
│       ├── components/
│       ├── pages/
│       ├── stores/
│       ├── game/                 # GameClient.ts, GameLoop.ts
│       ├── input/
│       ├── network/              # Transport, WebSocketTransport, SnapshotBuffer, Protocol, Prediction
│       ├── rendering/            # Renderer, CharacterMesh, Environment, Effects
│       ├── camera/
│       ├── audio/
│       └── debug/
├── assets/
│   ├── characters/
│   ├── maps/
│   ├── sounds/
│   ├── textures/
│   ├── animations/
│   └── licenses.json
├── config/
│   ├── physics.json
│   ├── character.json
│   └── combat.json
├── tools/
│   ├── asset-pipeline/
│   ├── map-validator/
│   ├── replay-viewer/
│   └── load-test/
├── tests/
│   ├── physics/
│   ├── networking/
│   └── e2e/
├── docs/
│   ├── protocol.md
│   ├── map-authoring.md
│   └── decisions/
├── infra/
│   ├── Dockerfile
│   └── compose.yaml
└── .github/workflows/
```

Create modules as behavior requires them, not dozens of empty stubs. Wire root test fixtures into actual Cargo integration targets; files under a virtual workspace root do not automatically run as crate tests. Keep derived protocol/map docs subordinate to this specification.

### 25.1 Engineering and comment conventions

Use Rust formatting/Clippy, strict TypeScript, explicit error types and domain IDs instead of loose strings. Public boundaries return structured errors. Avoid `unwrap()` on network, asset or player-controlled input. Keep configuration validation and units close to their types.

Apply the supplied commenting standards: explain why, constraints, tradeoffs, determinism, safety gates and state transitions. Required comment zones include metric thresholds, resource limits, lifecycle transitions and security-sensitive credential handling. Comments must describe verified behavior and assumptions; do not add speculative guarantees, trivial narration, commented-out code or unrequested TODOs. Use explicit `INVARIANT:` comments at critical boundaries. Keep ordinary comments short and purposeful.

Illustrative style (not a shipped/tested implementation):

```rust
// INVARIANT: eliminated players cannot keep outgoing grips.
// Releasing here prevents a disconnected or dead controller from holding live bodies.
if character.is_eliminated() {
    release_owned_grips(character.id, world);
}
```

Generated code proposals include a top-of-diff summary of change, expected impact, safety assumptions and actual validation status. Do not claim tests passed unless executed. Any future PR workflow uses explicit user-authorized publication and does not auto-merge. Storage/upload features, if later introduced, must document path ownership and signed URL/credential boundaries; they are not MVP dependencies.

### 25.2 Build and developer command contract

The following commands are the **required repository interface to implement during foundation**, not evidence that this currently document-only deliverable contains a runnable game. Adopt package names `brawler-engine`, `brawler-protocol`, `brawler-server`, `brawler-wasm` and `brawler-benchmark`, and define the named frontend scripts/bench targets.

```sh
rustup target add wasm32-unknown-unknown
cargo build --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
wasm-pack build crates/wasm --target web --release --out-dir ../../web/src/generated/wasm
npm --prefix web ci
npm --prefix web run typecheck
npm --prefix web run test -- --run
npm --prefix web run build
cargo bench -p brawler-benchmark --bench room_scenarios
cargo run -p brawler-server -- --config config/server.local.toml
npm --prefix web run dev -- --host 127.0.0.1
npm --prefix web run test:e2e
```

Foundation must add a documented local server config template without secrets, the benchmark harness and test scripts as their milestones arrive. Pin/install `wasm-pack` in the toolchain setup. Keep deterministic and performance feature builds separate if the selected Rapier release requires it; do not blindly use `--all-features` with incompatible physics options.

## 26. Core engine API

The engine owns simulation, not scheduling/networking. Preserve the source's API intent:

| Operation | Contract |
| --- | --- |
| `Simulation::new(config)` | Validate configuration; return structured error on invalid limits |
| `load_map(map)` | Load validated simulation data transactionally before play |
| `spawn_player(id, spawn)` | Stable IDs; reject duplicates/invalid occupied spawns |
| `remove_player(id)` | Idempotent body, collider, joint and reference cleanup |
| `apply_input(id, input)` | Queue/accept validated intention for tick-boundary use |
| `step()` | Exactly one fixed gameplay tick; no wall-clock reads |
| `snapshot()` | Complete authoritative state or clearly named render snapshot |
| `drain_events()` | Ordered events, drained once; reusable buffers when profiling warrants |
| `reset()` | Full round reset with explicit seed/content/settings |
| `tick()` | Current completed internal tick (`u64`) |

A render snapshot and a replay/prediction checkpoint are different types. The former may contain only presentation-relevant state; the latter must restore simulation dependencies. Never imply that `snapshot()` supports rollback unless its documented type includes the necessary state.

### 26.1 Stable IDs and lifecycle

Application-level entity/body/joint IDs map to Rapier's internal handles. Do not expose raw generational engine handles as durable network identity. Include round epoch and generation semantics; stale references must fail safely. Before deleting an entity, remove incoming/outgoing constraints and attached resources, then emit ordered removal events.

### 26.2 WASM boundary

Conceptual TypeScript interface:

```ts
interface LocalSimulation {
  loadMap(data: Uint8Array): void;
  spawnPlayer(id: number): void;
  applyInput(id: number, input: Uint8Array): void;
  step(): void;
  transformBuffer(): Float32Array;
  eventBuffer(): Uint8Array;
  reset(): void;
}
```

Use explicit lengths/versioning and expose errors; the above signatures describe roles rather than complete ownership/error semantics. Batch transforms in stable ID order with a parallel ID map. Borrowed typed-array views become invalid if WASM memory grows; reacquire after any allocating call or expose copied/owned buffers. Define whether event retrieval drains or peeks.

The passive diagnostic `LocalRagdoll` exports owned buffers: fifteen 14-float body records (position, XYZW rotation, linear/angular velocity, mass) in canonical anatomical order, and fourteen 17-float joint records (parent/child indices, both world anchors and XYZW joint-frame rotations, anchor separation). `definition_json()` supplies the validated geometry and limits. Limb indices are validated before numeric narrowing; tuning replaces the world only after validation. These local diagnostic layouts and the native trace comparison are documented in `docs/slices/04b1-ragdoll-wasm-contract.md`; they are not network snapshots or rollback checkpoints.

The diagnostic static factory `LocalRagdoll.standing(configuration)` uses the same body/joint layout with bundled standing motors and balance. Additional owned buffers expose fourteen parent-joint-local XYZ solver velocity targets (42 floats, rad/s), fourteen local child-side final-substep efforts (42 floats, N·m), and six controller values: enabled, left/right support, world pelvis torque XYZ. These are last-tick diagnostics, not a tick-average effort measurement. Motor configuration is read-only; gravity/solver tuning remains transactional and preserves the controller enable state. Reset preserves that state while clearing targets, effort and support. Enable transitions clear cached motor targets and efforts; invalid non-boolean values are rejected before JavaScript coercion. Passive fixtures return empty controller buffers and reject control changes. See `docs/slices/05b2-standing-browser.md`.

The diagnostic `LocalRagdoll.locomotion(configuration)` preserves these existing buffers and adds strict finite f32-representable X/Z movement input plus a boolean run flag. It returns eleven owned movement values (acceleration XYZ, support velocity XYZ, two support flags, phase, yaw torque, motor strength) and fourteen native relative XYZW target quaternions. Standing returns identity pose targets; passive returns none. Current motor metadata reflects the blended native preset. Configuration replacement preserves fixture kind and enabled state and clears movement; failed validation preserves the world and intent. These are inspection buffers, not network or rollback state. See `docs/slices/06b-browser-locomotion.md`.

The additive diagnostic `stopping_moment()` returns an owned three-float world-space vector in N·m for locomotion, and an empty buffer for passive/standing fixtures. It reports the commanded stopping-pair moment about COM on the last tick, separately from pure balance and yaw. Reset/disable/configuration replacement clears it, and the existing eleven-float movement buffer is unchanged. The locomotion inspector displays its magnitude.

The additive `support_moment()` diagnostic returns an owned three-float world-space incremental compensation moment in N·m for locomotion, empty for passive/standing. It reports the last tick after shared force-budget normalization, separately from stopping and yaw. Reset, disable and configuration replacement clear it; it may remain active during supported rotating-platform idle. The inspector shows its magnitude. The existing eleven-float movement and three-float stopping buffers retain their layouts.

The initial online client may render snapshots without running a duplicate world. WASM is immediately required for the local playground and later prediction. A worker can keep simulation off the main UI thread; begin with transferable buffers. Shared memory/threads are optional and require the relevant browser isolation/deployment headers, not an assumption for the baseline.

## 27. Physics development playground

Build this before the production lobby. It must run the shared Rust engine in the browser and offer a matching headless native scenario runner.

The implemented cuboid fixture inspector uses one pending diagnostic impulse, finite and at most 10 N·s, consumed on the next tick; its upward-impulse button supplies 4 N·s. Reset rebuilds the complete fixture from its validated configuration. Playback rates 1×, 0.5× and 0.25× change cadence, never physics dt. Pause/rate changes discard fractional debt while preserving cumulative discarded time; reset clears both. Collider outlines come from Rapier geometry. These are diagnostic-fixture controls, not character actuation or release-completion claims. See `docs/decisions/0003-playground-controls.md`.

The cuboid fixture recording format is strict versioned JSON containing an engine source/compiler fingerprint, producer target/profile, canonical configuration/hash, seed zero (no random fixture behavior), total completed ticks and tick-stamped impulses. Limits are 256 KiB, 36,000 ticks and 1,024 commands; exceeding a live log budget disables export until reset while simulation continues. Import validates completely before replacing the world, starts paused at tick zero and stops at the recorded endpoint. Replay rejects live impulses; exporting during replay preserves the full imported run. Gravity/solver tuning validates a maximum 4 KiB configuration and starts a fresh paused world/log. Invalid imports/tuning preserve the current world; pending file reads cannot supersede a later reset, tuning change or import. The same recording runs in the native `replay_fixture` CLI. This is tested fixture reproduction under pinned builds, not complete game checkpoints or universal native/WASM determinism. See `docs/decisions/0004-fixture-recording.md`.

Required controls: spawn/remove ragdolls and props; stand/walk/jump/fall/recover; slow motion; pause; single tick; reset with seed; apply known impulses; record/replay inputs; inspect contacts, motor targets, actual poses, anchor error, center of mass, support region, grip load, stamina and step time.

Required overlays: colliders, joint axes/limits, anchors, force vectors, contact normals, desired skeleton and actual skeleton. Live tuning edits a temporary configuration and exports a complete versioned configuration; production multiplayer does not accept arbitrary client tuning values.

The browser inspector offers cuboid, passive-ragdoll, standing-ragdoll, locomotion, low-stair, strike-practice and two-fighter sparring scenes. Scene replacement rebuilds default configuration and releases the prior native/GPU resources; the ragdoll starts paused. Its actual 15-body states drive diagnostic solid meshes, selected-body readings and bounded 4 N·s upward/−Z impulses. Paired world anchors/XYZ frames, maximum anchor separation and selected angular limits are inspectable. Physics tuning/reset use the shared controller and native reconstruction. Standing adds an enable toggle, per-joint targets/gains/effort caps, foot-support and balance-effort readings, and a bounded 10 N·s +X selected-body shove. Pale parent frames represent the relative bind-pose target; saturated child frames show the actual pose. Scene changes discard pending tuning drafts even when both scenes have identical configuration JSON. Cuboid recording remains available; character-input recording, full contact/force overlays and editable motor gains remain unfinished playground requirements. See `docs/slices/04b2-ragdoll-browser-inspector.md` and `docs/slices/05b2-standing-browser.md`.

The locomotion scene adds focused-canvas WASD/arrows, Shift running and captured hold buttons. This diagnostic explicitly uses world axes; production camera-relative controls remain required by §16.6. Pause, reset, successful tuning, disable, focus/visibility loss and disposal clear held intent. Keyboard repeats cannot rearm cleared input. Movement readings expose displacement, speed, support, phase, acceleration and motor blend. Pale target axes compose the parent world frame with native relative target quaternions, while saturated axes show the actual child frame. The inspection camera follows pelvis X/Z at a fixed height and offset; it is not the group camera. Narrow layouts reserve separate canvas/control space. See `docs/decisions/0009-browser-locomotion.md`.

| Example tuning control | Source illustration | Status |
| --- | --- | --- |
| Gravity magnitude | 9.81 m/s² | Baseline |
| Motor stiffness | 60 | Uncalibrated UI example |
| Grip threshold | 250 N | Uncalibrated; fails some 40 kg hanging cases |
| Walking speed | 2.5 m/s | Baseline candidate |
| Solver iterations | 8 | Benchmark candidate |
| Explicit substeps | 1 / 2 / 4 | Separate from solver/CCD settings |

In the single-fighter playgrounds, **Show external contact normals** draws a 0.20 m line at each loaded external contact on the selected limb. The direction shows the normal impulse acting on that limb. The inspector reports the number of loaded points and the sum of their normal impulse magnitudes in N·s for the completed solve. Selecting another body, stepping and resetting refresh the readings. [Contact inspection and stair measurements](docs/slices/06c2b4c-contact-inspector.md)

The **Low stairs** playground scene and native regression share the same physical fixture. Three adjacent 10 cm risers lead to a long landing. The browser draws the enabled terrain colliders from owned center/half-extent records, and reset or valid tuning reconstructs the same terrain. The default approach settles for 60 ticks and walks forward for 150. The shared controller detects a reachable, foot-sized fixed tread and biases both hip and ankle pitch during the approach. Native/WASM body traces agree through ascent, another 90 walking ticks and a 180-tick stop on the landing. Six initial timings and four cardinal headings have native posture, effort and support regressions. Some other starting timings still lead to a fall. Human playability and transitions between different terrain types remain open. [Stair fixture and verification](docs/slices/06c2b4b-stairs.md)

Provide test scenes for flat ground, slopes, stairs, ledges, overhead obstructions, rotating platforms, hanging weights and closed grip loops. Record configuration hash, build/toolchain versions, seed and input stream with every useful bug reproduction. This is the main tool for evaluating feel, not merely a debug screenshot generator.

The local strike-practice fixture places a 2 kg dynamic pad within reach of both fists and suspends it from a fixed pivot with a spherical joint. The playground displays the simulated pad pose and horizontal displacement. Reset and tuning replacement reconstruct the suspension. This fixture provides an observable contact and momentum test for the punch controller.

## 28. Performance specification

### 28.1 Targets, not results

| Metric | Proposed target | Measurement scope |
| --- | --- | --- |
| Server physics step p95 | ≤4 ms | Standard eight-player arena on declared reference CPU |
| Authoritative tick rate | 60 Hz | 16.667 ms scheduling deadline |
| Browser frame time | ≤16.667 ms p95 target | 60 FPS baseline on declared reference device/preset |
| High-end presentation | Up to 144 FPS | Optional; does not change simulation frequency |
| Initial critical download | ≤10 MB compressed | First playable arena, code/WASM and critical assets |
| Typical optimized downstream | ≤100 KB/s application bytes/client | Whole application stream; transport overhead separately |
| Preferred regional RTT | ≤60 ms | Placement target, not a promise about player networks |

Before interpreting any result, record CPU/GPU model, core allocation, RAM, OS, browser, viewport/DPR/preset, compiler, build flags, dependency versions, map/config hash, scenario seed and player count. Include an integrated-GPU laptop class and Safari/Firefox/Chromium coverage. Do not invent hardware measurements or claim a universal “ordinary laptop” result.

### 28.2 Tick budget

| Component | Illustrative allocation |
| --- | ---: |
| Physics | 4.0 ms |
| Character controllers | 2.0 ms |
| Gameplay/contact processing | 1.5 ms |
| Snapshot preparation | 1.0 ms |
| Remaining deadline headroom | ≈8.17 ms |

These allocations sum to 16.67 ms but are planning limits, not simultaneous measured percentiles. Physics p95 ≤4 ms does not mean the entire room consumes only 4 ms. Report total room CPU and scheduling latency independently. Network writes run outside the room's critical step and must not block it.

### 28.3 Benchmark scenarios and methodology

| Scenario | Purpose |
| --- | --- |
| One idle ragdoll | Baseline motors/balance |
| One walking ragdoll | Gait/controller overhead |
| Two fighting ragdolls | Collision and damage processing |
| Eight idle players | Scaling baseline |
| Eight players in a pile | Dense contacts |
| Eight mutually gripping players | Closed-constraint-loop stress |
| Eight players +100 props | Broad phase, contacts and lifecycle pressure |
| Full structure collapse | Burst spawning/breakage |
| Multiple concurrent rooms | Scheduler and host capacity |
| Native versus WASM | Platform/worker cost |
| Optional equivalent Rapier/Jolt prototype | Evidence before an engine migration |

Use release builds, a fixed seed/input script, realistic active bodies and the same simulated duration. Initial procedure: warm up 30 seconds, measure at least 120 seconds, repeat five times per configuration. Report per-run p50/p95/p99, worst observed tick, missed deadlines, active bodies/joints/contact pairs, allocated bytes, resident memory and output bytes. Keep cold-start/loading measurements separate. Compare median run-level results and show spread; do not average away collapse spikes or thermally degraded runs. Record power/thermal conditions and background load.

Criterion supports isolated microbenchmarks; full-room wall-clock traces and sustained load tests are separate required evidence. A regression is actionable when repeatable on the same baseline; an initial proposed alert is >10% p95 total-step regression, investigated alongside variance rather than treated as a universal statistical proof.

The [eight-player Skyline baseline](docs/verification/eight-player-room.md) records the native room and full snapshot measurements for the shipping rig and map.

### 28.4 Capacity estimate

CPU-only upper estimate:

```text
rooms ≤ floor(usable_cores × target_utilization / (tick_rate × total_CPU_seconds_per_room_tick))
```

For 4 usable cores, 65% utilization and **4 ms total room CPU/tick**: `floor(4×0.65/(60×0.004)) = 10 rooms`, or 80 players at eight per room. This reproduces the source illustration, but its 4 ms input is **total room cost**, not the physics-only target.

Using the illustrative 8.5 ms total allocated work instead gives `floor(2.6/(60×0.0085)) = 5 rooms`. Neither estimate proves those rooms meet deadlines; worker distribution, peak contacts, network, memory and host scheduling lower practical capacity. Determine admission limits from sustained load tests and retain headroom for escalation.

### 28.5 Optimization order

Stability → missed deadlines → controller/contact work → bandwidth → render cost → allocations → multi-room scheduling. Profile before changing architecture. Prefer collision filtering, sane mass ratios, sleeping eligible props, stable constraints and bounded debris before micro-optimizing incidental code. Never hide instability by disabling essential opponent collision or dropping physical hands from the release rig.

## 29. Testing strategy

### 29.1 Physics and gameplay tests

Use scenario-based tolerances and invariants, with exact replay comparisons only for declared deterministic builds.

| Test | Required outcome / initial fixture |
| --- | --- |
| Body accounting | 15 bodies, 14 anatomical joints, 40 kg computed mass |
| Idle stand | Stable support on flat ground for 60 s without nonfinite state |
| Walk/run | Intended direction and bounded speed; no wall acceleration exploit |
| Diagonal input | No speed advantage over unit cardinal input |
| Jump | One impulse per accepted edge/support episode, no infinite jump |
| Fall | Root crosses kill boundary; limb-only crossing does not falsely eliminate |
| Punch/headbutt | Damage only on qualifying contact, once per attack/target |
| Resting pile | No repeated support-contact damage |
| Grab/release | Correct anchors and exactly one constraint per hand |
| Destroy grabbed object | All affected references cleaned safely |
| Knockout/recovery | Own grips released; motors ramp; incoming grips remain physical |
| Lift/throw | Momentum retained, assistance not duplicated per hand |
| Ledge recovery | Standard fixture succeeds; blocked mantles do not clip |
| One-/two-hand hang | Load/stamina behavior matches configured thresholds |
| Moving/rotating support | Relative movement and departure momentum are correct |
| Eight-player grip loop | Finite state, bounded error/velocity; no unbounded energy growth |
| High-speed collision | Selective CCD prevents unacceptable fixture tunneling |
| Simultaneous elimination | Stable draw/winner independent of player iteration order |
| Round reset | Counts, seeds, inputs, hazards, constraints and epochs reset |

Proposed quantitative fixture tolerances: joints remain within 5 cm anchor separation under ordinary play, with separately recorded brief impact exceptions; all transforms/velocities finite; standard ledge rescue succeeds in at least 18 of 20 scripted attempts with the same initial setup. These are acceptance candidates to calibrate in the playground, not exact promises of physics realism. Preserve failing seeds as regression fixtures rather than weakening tests to hide a defect.

### 29.2 Network matrix

| Condition | Expected behavior |
| --- | --- |
| 20 ms RTT | Responsive baseline |
| 60 ms RTT | Normal regional playability gate |
| 100 ms RTT | Playable with visible added delay |
| 200 ms RTT | Degraded but stable and clearly indicated |
| Variable jitter | Bounded adaptive interpolation |
| Loss/retransmission | No corruption; TCP delays visible and measured |
| Temporary outage | Neutral input, valid reconnect/full resync |
| Slow receiver | No room-wide blocking or queue growth |
| Duplicate/stale input | No repeated action or stale round effects |
| Malformed frames | Bounded rejection, no panic |
| Flood | Rate limits and session isolation |
| Missing delta baseline | Request full state; no invalid delta application |

Test jitter candidates 0/10/30 ms and loss 0/1/3/5% using a network proxy or OS emulation. Browser WebSocket loss is not application datagram loss: TCP retransmits and stalls. Test both directions, burst loss, 1–5 s interruption and reconnect after grace expiry. Record input-to-visible latency separately from RTT. At 60 ms RTT with 10 ms jitter, set an initial measured p95 action-feedback target of ≤150 ms and require human acceptance; if it fails, revisit interpolation/prediction before shipping.

### 29.3 Headless clients and soak tests

Build clients that create/join, ready, send seeded combat/movement, reconnect, finish matches and collect metrics. Include both random stress inputs and scripted realistic fights. Headless load does not measure GPU/browser performance.

Proposed MVP endurance gate: eight active players for 60 minutes across repeated rounds, no server crash, nonfinite state or stale grips; memory reaches a bounded plateau. Run at least 100 consecutive reset cycles. Multi-room soak determines capacity with peak hazards active, not all characters asleep. The [native reset endurance run](docs/verification/reset-endurance.md) covers 100 consecutive round rebuilds across eight-fighter matches.

### 29.4 Browser end-to-end tests

Playwright uses multiple contexts to test room creation, share-link joining, colors/readiness, content loading, synchronized countdown, knockout/round transitions, scores, host transfer, disconnect/reconnect, late-join spectating and rematch. Test focus loss, menu input isolation, resize, audio unlock and missing WebGL/WASM capabilities.

Use real-browser manual tests for gamepads, sustained rendering, perceived latency and mobile/unsupported-device messaging; a DOM test alone cannot certify those. Cover current supported Chromium, Firefox and Safari versions at release and record exact versions.

### 29.5 Protocol, property and fuzz tests

Golden byte fixtures MUST prove the 18-byte input layout and little-endian fields across Rust and TypeScript. Test every truncation length, extra bytes, reserved bits, negative movement edge cases, sequence wraparound and press/release combinations. Fuzz input/snapshot/map decoders with allocation limits.

Invariants: no malformed record panics; no eliminated player issues actions or retains affected grips; all live grips target existing bodies; no round scores twice; no closed room accepts a session; no content hash mismatch enters play; resources stay bounded. Test quantization boundaries, quaternion sign/normalization, baseline loss and entity ID reuse. A full checkpoint replay test must include controller/RNG/lifecycle state.

### 29.6 Gameplay fidelity playtests

Run repeated sessions with 2, 4 and 8 friends and compare behavior against an agreed Gang Beasts reference build/video set. Record the reference version/date rather than chasing an undefined changing target. Use original game assets only as research references, never distribute them in this project.

Evaluate: controllable wobble, independent hands, resistance while held, lift difficulty, throw momentum, recovery window, rescue climbing, mutual falls, camera readability, hazard warning and online responsiveness. Log control confusion, successful rescue rate, round duration, camping and repeated exploits. Have testers attempt the same tasks in both reference and prototype. Fidelity is judged by these interactions, not by matching an undocumented numeric constant.

## 30. Security and abuse prevention

| Threat | Required mitigation |
| --- | --- |
| Speed/teleport cheats | Only authoritative fixed-tick actuation; no trusted client transforms |
| Attack spam | Server cooldowns and edge/sequence validation |
| Invalid movement | Decode bounds, normalization and finite checks |
| Packet flood | Per-session/IP budgets, bounded queues and allocation limits |
| Malformed content | Strict length/schema/enum validation |
| Room guessing | Random codes, join throttling, optional room lock |
| Session impersonation | Separate secret credentials, expiration and safe reconnect |
| Cross-origin abuse | Explicit WebSocket Origin allowlist and appropriate HTTP protections |
| Resource exhaustion | Limits on rooms, players, spectators, props, messages and lifetime |
| Name injection | Length limits, plain-text rendering and output escaping |
| Infinite grips | Stamina/load and stale-input rules |
| Host privilege abuse | Server checks for settings/start/kick operations |

Suggested initial operational limits: eight players, one active connection per session, 60 normal input frames/s with a bounded transition burst and a 120/s hard ceiling; tune with real accessibility/device input. Keep game rules independent of how fast a player sends packets. Cosmetic client state never changes colliders or mass.

Serve HTTPS/WSS, avoid logging credentials, keep secrets out of Vite/browser bundles, pin dependencies and run vulnerability/license checks. Apply strict limits before expensive decoding or room allocation. Do not expose `/metrics` publicly by default. Origin validation helps browser-origin abuse but is not authentication against arbitrary clients.

Private friend rooms do not need invasive anti-cheat. Public matchmaking adds reporting, blocking, moderation, name policy, abuse handling and retention decisions before launch. Replays/logs may contain names and session metadata: store minimal data, redact secrets and define retention/access. No voice/text chat service is required for the MVP.

## 31. Infrastructure and deployment

### 31.1 MVP deployment

Host content-hashed frontend bundles and assets on a static CDN; run a native Rust server on a persistent Linux VM/container service supporting long-lived WebSockets. Do not use short-lived request functions for physics. A domain, TLS, CDN, one measured-capacity server and monitoring suffice initially; Kubernetes is unnecessary.

Build the server in a multistage container, run without root where practical, define CPU/memory limits and publish only required ports. Configure proxy WebSocket upgrade/idle timeouts, application heartbeat and graceful shutdown. Serve WASM with the correct MIME type and compression; cache immutable assets strongly while keeping the boot manifest/version discovery fresh.

Client boot checks protocol/content compatibility. A deployment must not serve new code with an incompatible cached physics manifest. Retain previous content-hashed assets while existing rooms drain. No shared-memory WASM requirement in the baseline; if later enabled, explicitly test cross-origin isolation headers and dependent assets.

### 31.2 Scale and regions

```text
Global CDN
   └── Room directory / placement
         ├── US East authoritative hosts
         ├── US West authoritative hosts
         ├── Europe authoritative hosts
         └── Additional measured-demand regions
```

Keep one room on one host for its lifetime. Place new rooms based on measured capacity and group latency; route reconnects to the same owner. Host privilege transfer is not simulation migration. Redis may later provide expiring discovery leases; PostgreSQL may store profiles/results. Neither is in the physics tick's synchronous path.

Autoscaling adds capacity for new rooms, not arbitrary live-world migration. During drain, reject new rooms, preserve active sessions, allow bounded finish time and announce any shutdown. MVP host failure may lose an ephemeral match; report it clearly and offer a new room. Do not promise durable recovery without implementing complete checkpoint storage and recovery semantics.

### 31.3 Observability

Metrics: physics and total tick durations, p95/p99, deadline misses, room/player counts, active bodies/joints, contact pairs, snapshot bytes, queue depth, RTT/jitter, reconnects, invalid state, allocation rate and memory. Keep high-cardinality room/player IDs out of metric labels; use structured logs/traces for those IDs.

Logs include build/config/map hash, room/match/round ID, tick and affected entity where useful. Rate-limit repeated errors; attach a bounded replay/debug record for reproducible physics failures. Do not log every body every production tick. Alerts need action: stop admission/drain capacity, investigate nonfinite state, or roll back a bad build.

### 31.4 CI, release and rollback

On changes: format/lint/typecheck; native engine/protocol tests; WASM build/tests; Rust/TS golden protocol fixtures; map validation; browser build and critical multiplayer flows. Run sustained benchmarks on controlled hardware separately from noisy shared CI and compare against stored baselines. Test supported physics feature combinations explicitly.

Package server, web, maps and protocol version as one compatibility manifest. Stage a new build, run a real two-client smoke match, then admit a small number of new rooms. Promote only after health and gameplay checks. Rollback routes new rooms to the previous compatible build while draining existing rooms where possible; do not hot-swap incompatible simulation state. Record release hashes and test evidence.

# Part IV — Development roadmap

## 32. MVP implementation milestones

The order is physics-first. Time estimates below are the source's rough effort hypotheses for one experienced developer, not delivery promises; tuning and original content can dominate.

| Phase | Work | Rough estimate | Exit evidence |
| --- | --- | --- | --- |
| 0 — Foundation | Workspace, pinned tools, Rapier native/WASM, Three.js | 2–4 days | Same simple fixture runs native/browser; build/test commands work |
| 1 — Playground | Bodies, joints, debug overlays, tuning | 3–5 days | Deterministic fixture export, single-step, visible anchors/forces |
| 2 — Active ragdoll | Stand, walk, run, jump, fall, recover | 1–3 weeks | Stable controllable movement; fifteen-body mass/topology test |
| 3 — Combat | Independent strikes/grips, lift/throw, KO, climb | 1–2 weeks | Two local characters can fight and rescue from ledges |
| 4 — Arena/camera | Skyline, props/panels, framing/elimination | 4–7 days | Complete local round with readable camera and reset |
| 5 — Multiplayer | Server, codec, sessions, interpolation | 1–2 weeks | Eight clients agree on events/results under network tests |
| 6 — Product | Lobby, codes, scoring, original assets/audio | About 1 week | Friend can join/play/rematch without developer help |
| 7 — Playtest/deploy | Tuning, performance, soak, release | 1–2 weeks | §33 gates met with recorded evidence |

### 32.1 Concrete implementation order

- Foundation: scaffold only the first engine/protocol/WASM/web boundaries; validate one falling rigid body and versioned configuration.
- Playground: build physical skeleton with correct mass/inertia, then debug anchors and tune one joint before enabling all motors.
- Locomotion: idle balance → walk → run → jump → fallen recovery → slopes/support motion.
- Interaction: one hand strike → two independent hands → one grip → two grips → cyclic multi-player grips → lift/release → KO/recovery → ledge rescue.
- Local game: Skyline bounds/spawns → camera → breakage → round state/scoring/reset.
- Network: golden input codec → single-room owner → full snapshots → two clients → eight → disconnect/resync → latency/backpressure tests.
- Product: private-room user flow → clear errors/loading → original mesh/audio → accessibility → deployment.

Each change has a small behavioral test or observable fixture and a reviewable result. Do not proceed to elaborate matchmaking while ragdoll movement is still unpleasant. Networking research can be a bounded spike, but it does not substitute for the local interaction gate.

## 33. MVP acceptance criteria

Retain all 18 original completion checks, with explicit evidence expectations:

- [ ] **Browser entry:** Opens directly in supported desktop browsers; critical load fits the defined download budget or a documented approved revision.
- [x] **Friend joining:** Two to eight players join a private room link/code without accounts; ninth player is handled safely. [Browser lobby evidence](docs/slices/16b-browser-lobby.md#verification)
- [x] **Active ragdolls:** Characters use the physical rig and motors; body/joint/mass checks pass. [Gameplay rig, solver effort and browser verification](docs/verification/match-rig.md)
- [ ] **Locomotion:** Walking, running, jumping and recovery are controllable in human playtests.
- [x] **Independent hands:** Either hand can punch/grab while the other performs a different permitted action. [Physical strikes under a maintained grip and two-client hand states](docs/slices/09e-online-hand-actions.md#verification)
- [x] **Lifts/throws:** Opponents can be physically lifted/thrown with momentum and counterplay. [Airborne online release, physical counter-grip and rooftop ring-out](docs/slices/10d-skyline-throws.md#verification)
- [x] **Knockout/recovery:** Consciousness changes do not eliminate automatically or snap poses. [Per-limb continuity, score preservation and post-recovery control](docs/slices/17d-fighter-condition.md#verification)
- [ ] **Ledge recovery:** Standard Skyline rescue cases pass and feel intentional.
- [x] **Arena mechanics:** Falling, crates and breakable perimeter work and reset correctly. [Native fall and crate checks, two-client walkway loss and round restoration](docs/verification/skyline-round-restoration.md)
- [ ] **Camera:** Surviving/recoverable players remain readable across supported aspect ratios and map situations.
- [ ] **Authority:** Client packet manipulation cannot dictate transforms, hits or scores.
- [ ] **Smooth multiplayer:** Interpolation is stable and action latency meets the playability gate at the regional test profile.
- [x] **Round/match scoring:** All clients agree; simultaneous elimination and duplicate events score correctly. [Same-tick falls, duplicate results, eight-client agreement and first-to-three matches](docs/verification/round-scoring.md)
- [ ] **Disconnects:** Stale input, grace expiry, host transfer and reconnect work without orphan grips.
- [x] **Rematch:** Repeated rounds/rematches do not require refresh or leak stale physics/network state. [Two consecutive online rematches](docs/slices/16f-online-rematch.md#verification)
- [ ] **Performance:** Eight-player server/client budgets are measured on declared hardware; optimized traffic target is separately measured.
- [ ] **Stability:** Endurance and worst-case tests reveal no game-breaking explosions, persistent invalid states or unbounded memory growth.
- [ ] **Original content:** Original/licensed visual and audio assets are included with provenance.

Maintain an acceptance record with build/config hash, test device, scenario, result and remaining issue. An unchecked gate means the release is not complete. Do not mark checks merely because the corresponding source file exists.

## 34. Full-game roadmap

| Version | Scope | Completion gate |
| --- | --- | --- |
| 0.1 — Physics prototype | One local ragdoll, balance, locomotion, falls, live tuning | First vertical slice in §37 |
| 0.2 — Combat prototype | Multiple local ragdolls, punches, grips, lift/throw, KO | Physical local fights and rescue feel good |
| 0.3 — Multiplayer MVP | Up to eight online players, Skyline, camera, melee, rematch | All §33 checks |
| 0.5 — Expanded game | 3–5 arenas, advanced movement/climb, audio, costumes, gamepads, network refinement | New mechanics meet the same stability/performance gates |
| 0.8 — Content-complete beta | Approximately 12 arenas, teams/waves/football, public matchmaking, buoyancy, destruction, AI hazards | Mode/content completeness and public-abuse readiness |
| 1.0 — Full game | 12–20 polished arenas, ≥4 modes, customization, regions, replay, accessibility and optimized networking | Full regression, content QA and sustained operational evidence |
| 1.x — Extensions | Map creation, modifiers, shareable clips, community rooms, larger matches, optional transport | Separate scoped feature gates |

Full local couch multiplayer with multiple gamepads is a full-game requirement; implement it by 1.0, rather than leaving the source's occasional 1.x mention as a contradictory deferral. Earlier releases may expose a limited sandbox version.

Replay plan: debug recordings store build/config/map hashes, seeds, input/events and periodic complete checkpoints. If native/WASM deterministic replay is not validated, store authoritative state tracks for playback rather than promising input-only reconstruction. User-facing replays need scrubbing, camera controls, event/audio deduplication, compatibility handling, export privacy and bounded retention. Clip/video export is a later feature beyond basic replay playback.

A full game is likely several months or longer for one developer. Estimates must be revisited after locomotion, network and content pipeline prototypes reveal actual work. Do not compress the roadmap by lowering the stated fidelity goal without an explicit scope decision.

## 35. Technical risks

| Risk | Severity | Mitigation / proof |
| --- | --- | --- |
| Too-stiff ragdolls | Critical | Live tuning, push/stumble tests, reference comparison |
| Constant collapse | Critical | Support-aware balance, recovery fixtures |
| Physics explosions | Critical | Effort limits, sensible masses, anchors, stress replays |
| Online input delay | High | Regional placement, measured latency, gated prediction |
| Multi-player grip loops | High | Compliant grips, load failure, eight-player loop benchmark |
| Missed tick deadlines | High | Full-workload profiling, admission limits, headroom |
| Camera loses players | High | Bound projection, occlusion tests, map alternatives |
| Indefinite camping | Medium | Grounded stamina drain and telegraphed escalation |
| Climb exploits | Medium | Valid anchors/support, force limits, blocked-mantle tests |
| Excess bandwidth | Medium | Measured quantization/deltas, lifecycle-aware resync |
| Destruction desync | Medium | Server events, stable IDs, full-state reconstruction |
| Browser GPU/CPU limits | Medium | Quality presets, worker profiling, readable low mode |
| Replay divergence | High | Pinned builds, checkpoints, state-track fallback |
| Scope expansion | High | Release gates, original-asset pipeline, incremental maps |

The primary risk is building extensive infrastructure around an unenjoyable physical controller. Preserve a playable local fixture throughout development and require new systems to improve or support the core interaction loop.

## 36. Decisions to commit to now

| Decision | Selection |
| --- | --- |
| Core language / physics | Rust / Rapier 3D |
| Renderer / frontend | Three.js / TypeScript, React for UI |
| Browser core | Rust → WASM |
| Transport / authority | Binary WebSockets / dedicated native server |
| Character | Fifteen-body, 40 kg proposed rig; fourteen anatomical joints |
| Animation | Procedural desired poses plus physical motors |
| Camera | Elevated dynamic group framing |
| MVP map / mode | Skyline / melee |
| MVP cap | Eight players |
| Storage | Ephemeral in-memory rooms |
| Hosting | Persistent Linux server + static CDN |
| Development | Physics-first, measurable gates |
| Identity | Original assets, branding and layouts |

Defer full soft bodies, arbitrary mesh fracture, whole-world rollback, advanced ECS, databases, Kubernetes, accounts and public matchmaking until justified by the release plan. Deferment does not remove the full-game requirements.

Always validate input, preserve authority, keep configurations versioned, write meaningful invariant tests and document evidence. Revisit a major committed architecture or gameplay-fidelity reduction with the project owner before implementation. Never claim benchmarks passed without measurement, introduce copied proprietary assets, commit secrets or let presentation overwrite authoritative outcomes.

Remaining calibration decisions are bounded tasks, not blockers to the first prototype: motor/effort model, exact angular limits, collider proportions, damage threshold/scaling, grip load thresholds, per-map camera bounds, reference hardware and selected dependency versions. The playground and benchmark suite decide these with evidence.

# Part V — Research references

### Evidence and link verification

Links below were checked while preparing this file on November 24, 2026. “Opened” means the page was retrievable and relevant content inspected; it is not a promise that every historical claim remains true in the current game. Only primary developer/documentation sources support implementation facts. Secondary resources are preserved as contextual references, not authorities for hidden engine internals.

| Resource | Status | Supported use / limitation |
| --- | --- | --- |
| [Boneloaf developer physics discussion](https://discussions.unity.com/t/official-physics-improvements/619788/24) | Opened | Direct developer description of persistent force-driven ragdolls and joint-chain difficulties. |
| [Official development posts](https://gangbeasts.game/posts/) | Opened | Historical research index; use individual dated posts for specific claims. |
| [Official press kit](https://gangbeasts.game/press-pack) | Opened | Product identity and broad gameplay context, not a detailed mode/protocol contract. |
| [0.6.0 unstable beta notes](https://gangbeasts.game/posts/2017/9/20/patch-notes-for-the-gang-beasts-060-unstable-beta-build) | Opened | Historical Cinemachine use, throwing assistance, targeting and physics work. |
| [Historical official Steam announcements](https://store.steampowered.com/oldnews/?appgroupname=Gang+Beasts&appids=285900&enddate=1577865600&feed=steam_community_announcements) | Search result inspected | Historical group-camera development statement; not a current camera implementation guarantee. |
| [Aquarium update, build 1.13](https://gangbeasts.game/posts/2020/12/8/gang-beasts-aquarium-update) | Opened | Historical environment, grip/stamina, recovery, audio and online changes. |
| [Interactables update 1.24](https://gangbeasts.game/posts/2024/8/22/newly-articulated-gang-beasts-update-124) | Opened | Official targeting/interactable-system work and camera clipping fixes; supports the need for explicit target management. |
| [Crane update 1.25](https://gangbeasts.game/posts/2024/11/19/is-it-a-bird-is-it-a-crane-update-125-crane-stage) | Opened | Player-controlled machinery and bird hazards. The displayed publication date and URL date differ; do not infer a release day from the URL alone. |
| [Official Steam announcements](https://steamcommunity.com/app/285900/announcements) | Opened | Additional dated development/release material, not an exact current physics specification. |
| [Developer interview at MCV/DEVELOP](https://mcvuk.com/business-news/when-we-first-started-we-didnt-really-know-what-we-were-doing-how-gang-beasts-became-an-international-success/) | Opened | Development-history context; no private parameter extraction. |
| [Community wiki](https://gangbeasts.wiki.gg/wiki/Gang_Beasts_Wiki) | **Fetch blocked: HTTP 403** | Retained from the supplied conversation. The historical stage catalog was not independently reverified here. |
| [Dexerto controls reference](https://www.dexerto.com/guides/gang-beasts-control-guide-for-xbox-ps4-pc-1725828/) | Opened | Secondary control-guide context only; our bindings and mechanics remain independently specified. |
| [Rapier documentation](https://rapier.rs/docs/) | Documentation pages opened | Foundation API/feature reference; use the pinned release's corresponding docs. |
| [Rapier rigid bodies](https://rapier.rs/docs/user_guides/rust/rigid_bodies/) | Opened | Dynamic, fixed and kinematic body concepts. |
| [Rapier colliders](https://rapier.rs/docs/user_guides/rust/colliders/) | Opened | Geometry, sensors and mass contribution. |
| [Rapier joints](https://rapier.rs/docs/user_guides/rust/joints/) | Opened | Constraint types; inspect the pinned API for exact motor/limit capabilities. |
| [Rapier integration parameters](https://rapier.rs/docs/user_guides/rust/integration_parameters/) | Opened | Timestep, solver and CCD settings. |
| [Rapier Rust determinism guide](https://rapier.rs/docs/user_guides/rust/determinism/) | Opened | Conditional reproducibility requirements, not automatic application determinism. |
| [Original generic determinism link](https://rapier.rs/docs/user_guides/templates/determinism/) | Opened | The old link remains accessible; prefer the Rust-specific page for this project. |
| [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/) | Opened | Rust/WASM interoperability and browser integration. |
| [Three.js documentation](https://threejs.org/docs/) | Opened | Rendering, camera, skeleton and skinning APIs. |
| [MDN WebSocket](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket) | Opened | Browser WebSocket interface and backpressure limitation. |

### Research limitations and maintenance

The specification does not establish proprietary skeleton dimensions, camera equations, exact damage/stamina formulas or a current Gang Beasts protocol. It preserves reference observations where independent verification was unavailable and explicitly labels our implementations. Historical game constants are not transplanted as facts about the current release.

All proposed numerical parameters, capacity estimates, schedules and performance gates require prototype measurement. The original mass and wire-size arithmetic has been checked; no physical feel, browser performance or multiplayer behavior has yet been tested by creating this document.

When revising research, record the exact source, date/version and supported claim. Replace unsupported citations rather than inventing links. When tuning implementation, record config hashes and measured behavior; do not silently rewrite research statements to match the new tuning. Maintain this root file as the canonical specification.

## 37. Where to start

The first deliverable is a browser page with a floating platform and one Rust/WASM active ragdoll that can stand, walk, jump, fall over and recover. Three.js renders the bodies/mesh; a debug panel exposes motors, joint limits, support state and timing. The same fixture runs natively without a browser.

First-slice checklist:

- [ ] Pinned native and WASM builds load the same validated character/physics configuration.
- [ ] Fifteen bodies and fourteen anatomical joints instantiate at a verified 40 kg total.
- [ ] Colliders, anchors, axes and desired/actual poses are visible in debug mode.
- [ ] The character stands, walks, runs, jumps, falls and gets up using bounded actuation.
- [ ] Inputs and seed can be recorded and replayed under a documented build.
- [ ] The browser stays responsive and reports real step/frame timings.
- [x] A second ragdoll can be spawned, gripped, lifted and physically thrown off the platform. [Browser throw with a matched no-lift control](docs/slices/10d-skyline-throws.md#verification)

Do not begin with production matchmaking, costume catalogs or elaborate environments. Once two ragdolls fighting and rescuing themselves is entertaining and stable, build the authoritative online version around that same engine. Preserve the core requirement throughout: close physical gameplay fidelity, original presentation, and effortless multiplayer with friends.
