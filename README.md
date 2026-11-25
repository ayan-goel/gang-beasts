# Physics brawler

A browser multiplayer physics brawler built around a shared Rust/Rapier simulation and Three.js presentation. Skyline Melee is playable locally with two people on one keyboard. Online private rooms and several release gates remain in development.

The [technical specification](TECHNICAL_SPEC.md) is canonical. See the [documentation index](docs/README.md) for implementation order, decisions, and acceptance evidence.

## Play Skyline Melee

Start the browser project using the setup below and open `/play/`, or select **Play Skyline Melee** from the playground. The match is first to three rooftop ring-outs. A countdown starts each round, and rematch is available after the winning round. Coral uses WASD and J/K, Blue uses the arrow keys and comma/period. The start and pause screens show the full bindings.

For the production build, run `npm --prefix web run build` followed by `npm --prefix web run preview`. Open the printed local address with `/play/` appended. [Local match rules](docs/slices/12a-local-rounds.md) and [browser verification](docs/slices/12b-play-skyline.md).

## Native development

Install Rust using rustup. The repository pins Rust 1.88.0 with rustfmt and Clippy.

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
```

The engine has no network, rendering, or wall-clock dependency. `config/physics.json` is the initial versioned configuration.

The passive character uses `config/character.json`. Run `cargo test --locked -p brawler-engine --test ragdoll -- --nocapture` for its mass, anchor, drop and reset fixtures. The same rig is available in the browser's **Test scene → Passive ragdoll**. See [native rig evidence](docs/slices/04a-native-passive-ragdoll.md) and [browser inspection](docs/slices/04b2-ragdoll-browser-inspector.md).

The native `PoseRagdoll` diagnostic adds bounded bind-pose motors using `config/motors.json`. Run `cargo test --locked -p brawler-engine --test pose_motors -- --nocapture`; see [motor contracts and evidence](docs/slices/05a-bounded-pose-motors.md). The separate `config/standing-motors.json` and `config/balance.json` presets enable support-aware standing. Run `cargo test --locked --release -p brawler-engine --test standing -- --nocapture`; see [standing evidence](docs/slices/05b1-native-standing.md). The same preset is available under **Test scene → Standing ragdoll**; see [browser standing and inspection](docs/slices/05b2-standing-browser.md).

The native `LocomotionRagdoll` adds assisted walking/running, turning, braking and procedural leg targets. Run `cargo test --locked --release -p brawler-engine --test locomotion --test locomotion_config`; see [locomotion contracts and evidence](docs/slices/06a-native-locomotion.md). The same controller is available under **Test scene → Locomotion**; see [browser movement and verification](docs/slices/06b-browser-locomotion.md). Native moving-platform response is covered by [moving-support fixtures](docs/slices/06c1-moving-supports.md). Native uphill/downhill walking, running and braking are covered by [slope fixtures](docs/slices/06c2a-native-slopes.md). Flat-ground direction changes and keyboard neutral gaps are covered by [reversal fixtures and native/WASM traces](docs/slices/06c2b1-reversals.md). Varied flat/forward-slope stops and short moving-deck stops are covered by [stopping fixtures](docs/slices/06c2b2-stopping.md), with an explicit moment cap and visible readout. [Rotating-deck traversal](docs/slices/06c2b3-rotating-traversal.md) adds native held/released movement coverage, twenty-second idle checks and bounded support compensation. The [native terrain controller](docs/decisions/0015-terrain-locomotion.md) covers backward and cross-slope travel, landing after release and stumble recovery. Steps and browser terrain acceptance remain in development.

## Browser playground

Use Node 20.18.0 (`.nvmrc`). Install project-local wasm-pack once, then build the shared engine before starting Vite:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.88.0
cargo install wasm-pack --version 0.13.1 --locked --root .tools
npm --prefix web ci
npm --prefix web run wasm
npm --prefix web run dev
```

Open the localhost URL printed by Vite. During development, re-run `npm --prefix web run wasm` after changing Rust or configuration; generated browser files are ignored. Production `npm run build` always rebuilds WASM and copies the current canonical configuration first. wasm-pack installs the matching wasm-bindgen generator on first use. React manages the playground inspector; per-frame rendering stays imperative.

```sh
npm --prefix web run typecheck
npm --prefix web test -- --run
npm --prefix web exec -- playwright install chromium
npm --prefix web run test:e2e
npm --prefix web run build
```

Browser integration tests start their own local server if needed. They compare a 600-tick WASM trace against the native release fixture and test canvas resize containment. The production static bundle is in `web/dist`; it includes the diagnostic playground and the local game at `play/index.html`.

## Playground controls

Choose **Passive ragdoll** to inspect the fifteen-body rig paused at tick zero. Select a limb for readings and impulses, enable joint frames/colliders, then resume to observe its unpowered fall. Joint limits and anchor separation are available in the inspector. Scene switching starts a fresh world with default tuning. Choose **Standing ragdoll** for the supported standing preset. Toggle character control, inspect support and per-joint effort/velocity targets, or shove the selected body with a 10 N·s +X impulse. Pale parent joint axes show the bind-pose target; saturated child axes show actual orientation. Reset and physics tuning preserve the control toggle. Scene changes discard unsubmitted tuning drafts. Ragdoll recording is not implemented; recording controls apply to the cuboid scene.

Choose **Locomotion**, resume and focus the canvas to walk with WASD/arrows and run with Shift. Directions use world axes (forward −Z, right +X). Hold the arrow buttons or activate them with Space/Enter for an alternative; the Run checkbox selects running. Pause, reset, disabling control and focus/visibility loss clear movement. Pale joint axes show the requested gait. The stopping-moment readout reports bounded righting assistance during release. The separate support-moment readout reports rotating-support compensation; it is zero on the current flat playground. Use Space in the focused canvas or the Jump button to jump. Holding Space gives one takeoff. Directional input provides limited steering in flight. Reset after falling from the open platform. A fallen character automatically attempts to get up after settling on a supporting surface. To inspect it, disable character control, select the chest, shove it, resume, then re-enable control after the fall. The movement inspector reports recovery effort.

With Locomotion running, click the canvas with either mouse button to punch with the corresponding hand. Hold to keep reaching after the punch. J and K and the two hand buttons provide the same controls. The inspector shows each hand’s phase.

Choose **Local sparring** for a two-fighter encounter on one keyboard. Resume to focus the canvas. Coral uses WASD, left Shift, Space and J/K or mouse buttons. Blue uses arrows, right Shift, Enter and comma/period. Tap a hand control to punch, or hold through the punch to grab a nearby opponent. Hold E (Coral) or slash (Blue) to raise attached hands. Release a hand control during the lift or swing to let go with the opponent’s current momentum. Both fighters can move, punch with a free hand and see which limb each hand holds. The stamina readout drops while gripping and recovers with both hands free. Exhaustion releases the grip. The panel counts qualifying hits and displays the latest hit region and points. Punch damage can knock a fighter out. The condition readout tracks health, knockout and recovery. A downed fighter drops their grips and recovers on the same physical rig. Reset encounter starts both fighters at their original spawns and pauses the simulation.

Choose **Strike practice** to punch a suspended 2 kg pad. Resume, then use either hand. The pad can swing and twist. The displacement readout measures horizontal distance from its resting line. Reset hangs the pad at rest again.

Choose **Low stairs** to inspect the same three 10 cm risers as the native traversal test. Resume and walk forward with W or ↑. The current controller trips on the second riser in the scripted approach. [Fixture and reproduction](docs/slices/06c2b4b-stairs.md)

Pause to enable exact single-step. Reset reconstructs the fixture and preserves the paused/running mode. Slow motion changes playback cadence without changing physics dt. The upward impulse is queued once for the next tick. Enable **Show colliders** to inspect Rapier geometry.

**Apply tuning** validates gravity/solver settings and restarts paused. **Export configuration** downloads the running configuration. **Export recording** saves completed ticks and impulses; **Import recording** loads a compatible file paused at tick zero. Resume or single-step to its exact endpoint, then reset to return to live play. During replay, export preserves the entire imported recording. Logs are limited to 10 simulated minutes, 1,024 impulses and 256 KiB; reset starts a new log after a limit is reached.

Reproduce a browser recording with the same engine sources/toolchain natively:

```sh
cargo run --quiet --locked --release -p brawler-engine --example replay_fixture < fixture-recording-v1.json > trace.json
```

The trace includes initial state and every completed tick as position, XYZW quaternion, linear velocity, angular velocity and mass (14 floats). See [recording contracts and evidence](docs/slices/03b-recording-tuning.md). This validates the cuboid fixture, not full-game deterministic replay.
