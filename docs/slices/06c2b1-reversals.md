# Slice 06c2b1: Bounded reversal braking

Status: verified November 25, 2026, within the fixture scope below. Specification §§12, 29. Design: [ADR 0012](../decisions/0012-reversal-braking.md).

## Change

The movement controller briefly brakes existing momentum before turning into an opposing request. Detection runs at physics ticks and retains bounded direction history across neutral keyboard gaps. Requested analog magnitude and nonzero movement run state remain authoritative; neutral retains the previous gait cadence while braking. Force/torque limits, physical bodies, configuration and public buffers are unchanged.

## Behavioral gates

| Scenario | Required evidence |
| --- | --- |
| Repeated reversals | Settle 60 ticks; eight alternating ±local-X legs of 45 ticks each, both walk/run in four world orientations on the 0.08 m-clearance flat fixture; head >1.1 m throughout and signed pelvis speed >0.5 m/s after each leg |
| Default spawn | Same eight legs and speed/posture gates at the browser’s ordinary zero-offset spawn |
| Neutral gaps | 1- and 5-tick gaps within each 45-tick leg; neutral publishes `running=false`; both speeds and both spawn fixtures retain posture and signed speed gates |
| Same-tick key ordering | Direct reversal and neutral/opposite events before a tick produce identical body and movement states for 45 ticks |
| Updated request | Change a running reversal after five ticks to walk with −0.35 analog intent; remain upright for 180 ticks and finish within 0.3 m/s of −0.875 m/s |
| Budget | Reversal changes momentum toward the request, reports matching net acceleration, applies ≤8 N·s summed absolute impulses and no vertical impulse or transform change; yaw is zero during the full catch |
| Lifecycle | Actual active reversal clears on support removal and public disable; controller reset immediately clears pending state and recent direction history |
| Pure transition | Repeated/updated samples cannot extend timeout; neutral never resumes motion; history expires; speed/zero-budget exits and continuous perpendicular steering |
| Native/WASM | Six 421-frame traces: walk/run × 0/1/5 neutral ticks; every body/movement component agrees within 1e−5, with posture and endpoint direction gates in Chromium |

Native physical scenarios retain finite-state, joint-anchor <0.05 m, horizontal acceleration ≤12 m/s², yaw ≤20 N·m, pure balance ≤30 N·m and current whole-vector motor-effort checks. The original failing reversal fixtures ran before the implementation. Independent review found the same-tick keyboard edge and corrected lifecycle tests to prime an active catch. The first expanded Chromium trace failed at default-spawn tick 416 during a one-tick running key gap; native reproduction confirmed the cadence discontinuity, and retaining the previous cadence through neutral corrected it without relaxing the gate.

## Verification

All **108 native tests** (69 library and 39 integration) passed in both the full workspace debug run and the engine release run. **16 TypeScript tests**, **28 Chromium tests**, the production WASM/TypeScript/Vite build, formatting and Clippy with warnings denied also passed. No whole-game or terrain acceptance is claimed.

Environment: Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0, Playwright 1.65.1/Chromium 159.0.8154.6 on local macOS. Engine source/compiler/config fingerprint: `1cb31025f85a476b69ba89c13f57c1938961912b25d7f24513b979919829ddf5`. Built WASM SHA-256: `82ffa26fcbe7d500d5284bfaa027572f8016462533575f9d12597062f767c733`. Production output: 544.73 kB gzipped WASM and 203.66 kB gzipped JavaScript. Existing build-tool/chunk-size advisories remain; these sizes do not establish full-game performance.

The six native trace cases had minimum head height 1.256 m and minimum signed leg-end speed 1.961 m/s. Chromium independently passed the 1.1 m/0.5 m/s gates and matched every trace component within 1e−5. Independent review found no remaining source blocker after the key-event, lifecycle and cadence corrections.

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

Cross-slope running, low steps, terrain transitions, varied stopping phases and human movement assessment remain required. The original §33 MVP release gates are unchanged. The specific full-release failure and terrain reproductions are recorded in [terrain scenarios](../verification/terrain-gaps.md).
