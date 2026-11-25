# Slice 06c2a: Native slope gait and braking

Status: verified November 25, 2026, within the native fixture scope below. Specification §§11–12, 29. Design: [ADR 0011](../decisions/0011-slope-gait-and-braking.md).

## Change

The native locomotion controller adjusts hip/knee reach and sole targets using validated slope normals while retaining a desired gravity-upright frame. Braking can counter foot drag and brake the torso within the existing total absolute impulse budget. No configuration values or actuator ceilings increased.

## Behavioral evidence

All fixtures use bundled configuration, Rust 1.88.0, Rapier 0.26.1 enhanced determinism, 60 Hz fixed ticks, and no RNG. The support box has half-extents (30, 0.5, 30), its top plane passes through the origin, and spawn elevation is 0.08 m. Both feet must begin separated from the slope. Four yaw orientations rotate both character and slope together.

| Fixture | Required result |
| --- | --- |
| Standing | ±15° × four world orientations × 3,600 ticks: head vertical clearance >1.1 m; final horizontal pelvis drift <0.25 m |
| Walking/running | Same eight slopes, both speeds; settle 120 ticks then move 240: head clearance >1.1 m throughout; progress >70% of speed ×4 s |
| Stop | Both inclines at yaw 0: run 90 ticks, release for 600: remain upright, finish below 0.2 m/s and at idle motor strength |
| Impact | Three consecutive 10 N·s +X chest impulses after settling on both inclines at yaw 0; head clearance falls below 0.8 m within 480 ticks |
| Removed support | Disabling the surface before controller advance yields no foot support, drive or yaw; no transform/velocity mutation and identity relative targets |
| Braking budget | Opposing foot momentum with positive net braking; summed absolute impulses ≤8 N·s, zero vertical impulse, net momentum matches reported acceleration and transforms are unchanged |
| Saturated ankle | Final hip/knee FK preserves the reachable requested ankle point despite a clipped ankle angle; actual root/shin tilt does not change terrain targets at fixed heading/nominal gait |
| Steering continuity | A ±0.00001 m/s perturbation around perpendicular flat steering does not flip acceleration or foot impulse |
| IK math | Reachable nonplanar points match FK within 10 µm; unreachable/folded points remain finite and within geometric reach |

Standing, traversal and stop ticks also check finite body state, joint anchor separation <0.05 m, horizontal assistance ≤12 m/s², yaw ≤20 N·m, pure balance torque ≤30 N·m, and each current motor's whole-vector effort cap. These are physical native scenarios, not a claim of human-tested game feel.

## Verification

Rust formatting and Clippy with warnings denied passed. All **94 native tests** (55 library and 39 integration tests) passed in both the workspace debug run and the engine release run. The production WASM/TypeScript/Vite build, **16 TypeScript tests**, and **27 Chromium integration tests** passed.

Verification uses Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0 and Chromium 159.0.8154.6 through Playwright 1.65.1 on the local macOS host. Engine source/compiler/config fingerprint: `d9ec9f59bdd47c7f07783b54299d380a92c1d267d7b57176c4ee83096e091a84`. Built WASM SHA-256: `f6a3e8f2e4970ccf856ffac997486446a5ec0265788ffe44e8e4ed93b28a6923`. Production output was 544.00 kB gzipped WASM and 203.66 kB gzipped JavaScript. Existing build-tool/chunk-size advisories remain; these are build sizes, not measured game performance.

Independent review prompted the final-placement regression and the near-perpendicular steering regression. The latter exposed a sign discontinuity in a rejected general steering prototype. Counteracting impulses now apply only during release braking, with continuous motion projection; active steering retains nonnegative allocation. Review found no remaining source blocker.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked --release -p brawler-engine
npm --prefix web test -- --run
npm --prefix web run build
npm --prefix web run test:e2e -- --workers=2
```

## Remaining work

Cross-slope travel, stairs, repeated reversals and terrain transitions remain open in [terrain scenarios](../verification/terrain-gaps.md). Browser terrain and native/WASM traces for these new scenes have not been implemented. Flat-browser regressions do not certify slope parity. Jump, recovery, combat, online play and all specification §33 release gates remain incomplete.
