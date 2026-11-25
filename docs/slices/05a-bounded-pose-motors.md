# Slice 05a: Native bounded pose motors

Status: verified on November 24, 2026.

## Scope and contracts

`PoseRagdoll` adds bounded relative bind-pose control to the fifteen-body native diagnostic. Per-joint physical gains live in `config/motors.json`; [ADR 0006](../decisions/0006-bounded-relative-pose-motors.md) defines frames, units, effort limits, impulse lifetime and tuning limitations. The readonly `rig()` view supplies existing body/joint diagnostics. Queued limb impulses retain the one-slot, 10 N·s contract. Motor output contains fourteen world-space torque vectors in definition joint order, including zero entries for fixed wrists.

This is a sub-slice of standing control. No ground detection, upright balance, gait, recovery, browser motor UI, configurable target poses or motor recording is implemented. All §33 release gates remain open.

## Behavioral evidence

- Integration tests initially failed on missing public motor/configuration types. They now exercise disabled/passive equivalence, driven-drop effort/stability, exact reset trace, and malformed tuning.
- Pure controller tests cover radian-valued proportional response, damping sign, nonzero targets, equivalent quaternion signs including exact half-turns, the 179°/−179° shortest arc, locked axes, and diagonal vector effort caps.
- Physical actuator tests cover conserved angular momentum across unequal limb inertias, unchanged transforms/linear velocities, covariance under a whole-rig rotation, bounded response to a 50 rad/s imposed head spin, and immediate disabling.
- Independent review identified missing evidence that the physical wiring actually restores a displaced joint. An isolated neck test now starts at 0.2 radians with zero velocity and verifies −1.2 N·m child torque and independently calculated ±0.02 N·m·s angular-momentum changes on the connected bodies.
- Temporarily removing the restoring spring and separately doubling the tick impulse each made that physical test fail. Production sources were restored, and follow-up independent review found no remaining blocker.
- A 1,200-tick (20-second) drop with a 3/0/1 N·s head impulse remains finite, retains bodies above the floor, and bounds every requested motor effort. Maximum measured anchor separation is **0.015605 m**, below the existing 0.05 m diagnostic threshold. This is one fixture, not a worst-case gameplay bound.
- A 240-tick disabled run matches every passive body-state field exactly. A 120-tick driven reset repeats every body-state field exactly and discards a pending impulse.

Reproduce native behavior:

```sh
cargo test --locked -p brawler-engine --test pose_motors -- --nocapture
cargo test --locked -p brawler-engine --lib
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

All **34 native tests** pass, along with formatting and Clippy with warnings denied. The production WASM/browser build and strict TypeScript checking pass. All **16 Chromium browser tests** pass against the rebuilt engine, including existing passive-rig and cuboid native/WASM parity, recordings, visible controls, and failure cleanup. Playwright required permission to start its localhost server outside the filesystem sandbox; the initial restricted attempt could not bind the port and ran no tests.

Production output is approximately 511.90 KB gzipped WASM and 199.74 KB gzipped JavaScript. Existing Vite raw-chunk and wasm-pack tool-version/prebuilt-generator notices remain. No visual changes were made. Driven native/WASM parity remains a future gate; rebuilding the passive browser checks regressions but does not expose or validate browser pose control.
