# Slice 06b: Browser locomotion

Status: verified on November 24, 2026. Specification §§12.1, 26.2, 27. Design: [ADR 0009](../decisions/0009-browser-locomotion.md).

## Behavior

**Test scene → Locomotion** starts the native fifteen-body controller paused on the finite platform. Resume and focus the canvas; WASD/arrows walk, Shift runs. Forward is world −Z and right is +X. Pointer or Space/Enter holds on the direction buttons provide an alternative; the Run checkbox selects running. Release, pause, reset, successful tuning, disabling control, focus loss, document hiding and scene disposal clear intent. Typing in tuning fields cannot move the character. Scene replacement starts with default configuration and fresh controls.

The movement inspector reports command, pelvis X/Z and horizontal speed, commanded acceleration, active motor strength, gait phase, yaw effort and support velocity. The existing motor inspector reads current native gains and caps. The thigh cap is 48 N·m at rest and 150 N·m during active movement, with intermediate values during relaxation. The joint overlay shows native desired pose axes alongside actual axes. A following inspection camera preserves the character's horizontal framing. Narrow-screen controls sit below their own canvas area.

This remains a local diagnostic. The camera is not the multiplayer group camera, controls are not yet camera-relative, and the meshes are collider geometry rather than final character art. Open edges permit falls; reset is the recovery control until physical get-up is implemented.

## WASM contract

`LocalRagdoll.locomotion(configuration)` constructs bundled locomotion, standing, balance and character presets. `set_movement(x, z, running)` accepts finite JavaScript numbers representable as f32 and a strict boolean. Native normalization bounds the resulting direction. Invalid input preserves the previous intent. Passive and standing fixtures reject movement commands.

All returned buffers are owned copies and survive further calls, tuning and free. Existing body, joint, velocity-target, effort and control layouts remain unchanged.

| Method | Locomotion layout | Other variants |
| --- | --- | --- |
| `movement_state()` | 11 floats: acceleration XYZ (m/s²), support point velocity XYZ (m/s), left/right support flags, phase (rad), yaw torque (N·m), active strength [0,1] | Empty |
| `pose_targets()` | 14 XYZW relative joint-target quaternions, in definition order | Standing: 14 identity quaternions; passive: empty |
| `motor_configuration_json()` | Current blended native motor configuration | Existing standing/passive behavior |

Diagnostics describe the last completed controller tick; they are not prediction checkpoints. Configuration replacement preserves locomotion kind and enabled state and starts at tick zero with neutral movement. Reset clears movement/phase/diagnostics and restores idle gains while preserving enabled state. Disabling clears locomotion but still allows already queued external impulses, as defined in slice 06a.

## Verification and corrections

The new factory and browser scene began with failing tests. Further regressions reproduced unrelated keyup ending a button hold, stale repeat restoring input after blur, neutral-write failure skipping native/GPU cleanup, and narrow controls obscuring the scene. These defects were corrected before completing the slice.

The 481-frame native/WASM trace includes initial state and 480 completed steps. Before steps 61/181/361 it starts +X walk/−Z run/−Z walk; before 121/241/421 it releases; 271/272 queue chest shoves; 301/321 disable/enable; 341 resets. It compares every component of 210 body, 238 joint, 42 effort, 42 solver-target, 56 pose-target and 11 movement values, plus exact serialized current motor metadata. The absolute deviation gate remains <1e-5.

The trace exposed inherent `f32` trigonometry diverging between native and WASM. Explicit Rapier math-trait calls correct yaw and gait without changing formulas or tolerances. Afterward the captured run had no differing f32 bit patterns; the maximum JSON-number deviation was 3.7671e-6 due to native f32 decimal serialization versus JavaScript number expansion. This is evidence for the pinned fixture, not universal cross-platform equivalence.

Browser tests also cover strict input types/f32 overflow, rejected-command atomicity, buffer ownership through configuration/free, disabled tuning, unsupported fixture capabilities, live 48→150→48 thigh caps, actual displacement, running, key/pointer release, focus/visibility loss and scene replacement. Fault injection checks one native free, canvas removal and WebGL context loss even when neutral input throws. TypeScript tests verify alias normalization and target-axis composition with independently chosen noncommuting rotations.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked --release -p brawler-engine --test locomotion --test locomotion_config
npm --prefix web run typecheck
npm --prefix web test -- --run
npm --prefix web run build
npm --prefix web run test:e2e -- --workers=2
```

The complete workspace passed **72 native tests**, **16 TypeScript tests**, and **27 Chromium integration tests**. Rust formatting, Clippy with warnings denied, strict TypeScript, the production WASM/browser build and the separate ten release locomotion/config tests passed. Independent review found no remaining source blocker after the documented corrections.

Verification used Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0, and Playwright 1.65.1 / Chromium 159.0.8154.6 on the local macOS host. It was a correctness run, not a performance benchmark. Engine source/compiler/config fingerprint: `6763054fb5e89c772b5988c9a2c861cca46f6f052ae64eea05a2f5c96471c1db`. Physics file SHA-256: `514d8064d10f7046f8445bd9ccad203e121dfd6f41129daaa522344740d534fc`. Built WASM SHA-256: `8eab4a45142b492bc3b103c30b34def3f668e8de6d94a85a850ad78c24b2c692`.

Desktop gait frames and the corrected 320 px layout were visually inspected. Widths 320, 768, 1024 and 1440 px had no horizontal document overflow or page errors. Three slow-motion captures showed different leg poses and forward displacement while the character remained framed. Temporary screenshots are ignored under `.cache/locomotion-*.png`; they are inspection evidence, not maintained visual baselines or human playtests. The production bundle measured 541.36 kB gzipped WASM and 203.66 kB gzipped JavaScript. Vite's raw-chunk advisory and wasm-pack's existing tool notices remain.

## Remaining work

Slice 06c covers slopes, steps, rotating/moving support transitions, reversals and human movement assessment. Jump/get-up, combat, grips, Skyline, group camera, match lifecycle, networking, original assets/audio, performance and release gates remain unfinished. Character-input recording, editable motor gains and full contact/force overlays are also outstanding. Automated fixtures and screenshot inspection do not certify game feel or the supported-browser release matrix. All §33 release checks remain open.
