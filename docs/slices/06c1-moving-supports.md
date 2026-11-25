# Slice 06c1: Native moving-support transitions

Status: verified on November 25, 2026. Specification §§12.1, 12.2, 14.4, 29. Design: [ADR 0010](../decisions/0010-moving-support-response.md).

## Change

Idle locomotion now follows bounded changes in support velocity while retaining soft damping against player-relative impacts. A rotating deck no longer pulls the feet out from underneath an otherwise idle character. The same change handles translation starts and stops. This introduces no stronger motors, vertical lift, position tether, transform changes or larger total impulse budget.

Support sampling also rejects disabled foot bodies/colliders, sensor feet, disabled surfaces/parents, and deleted peer colliders before the next narrow-phase rebuild. Disabling or deleting a ridden deck immediately removes assistance and preserves existing horizontal departure momentum.

## Behavioral evidence

The new rotating-platform test failed at tick 61 on the prior controller. A faster-damping prototype stayed upright but failed the chest-impact gate, and was replaced rather than accepted. Review exposed partial-transfer over-counting and a deleted-collider panic; both received failing regressions before correction. The final helper and physical fixtures check:

| Fixture | Gate |
| --- | --- |
| Deck velocity reference | Each scheduled change ≤0.2 m/s for the tested 12 m/s²/60 Hz budget; start and stop converge |
| Contact transfer | Already supplied motion is subtracted from the bounded increment; steady relative impacts receive no extra carry correction |
| Vertical separation | Carry is horizontal, including when the sampled surface has vertical motion |
| Idle translating deck | Each cardinal direction at 0.8 m/s for 300 ticks, then abrupt stop for 300 ticks; head >1.1 m, local horizontal drift <0.25 m while riding, final horizontal speed <0.2 m/s |
| Idle rotating deck | Yaw 0.4 rad/s, spawn radius 2 m, 600 ticks; head >1.1 m, horizontal platform-local position error <0.4 m, final tangential velocity error <0.3 m/s |
| External impact | Three consecutive 10 N·s +X chest impulses after riding; head falls below 0.8 m within 240 ticks |
| Support departure | Disable body, disable collider, or actually remove collider before stepping; no subsequent drive/yaw, momentum change <0.001 kg·m/s per horizontal component over 30 ticks |
| Command budget | From rest, an 8 m/s deck speed change exercises the existing ≤8 N·s total tick impulse; summed momentum matches reported acceleration, Y impulse stays zero, transforms remain unchanged |
| Reset | Pending deck response is discarded; subsequent default-floor trace matches a fresh rig |

Ride/impact fixtures also require finite state, anchor error <0.05 m, acceleration ≤12 m/s² and yaw effort ≤20 N·m. Tolerances are scenario assertions, not claims that arbitrary platform motion is stable. The rotation test checks platform-local position throughout, so retaining radius without following the deck cannot pass.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked --release -p brawler-engine --lib
npm --prefix web run build
npm --prefix web test -- --run
npm --prefix web run test:e2e -- --workers=2
```

The workspace passed **84 native tests**, including all 45 library tests in both debug and release builds. **16 TypeScript tests** and all **27 existing Chromium integration tests** passed. Rust formatting, Clippy with warnings denied, strict TypeScript and the production WASM/browser build passed. Independent review found no remaining source blocker after the partial-transfer and collider-deletion corrections.

Verification used Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0 and Chromium 159.0.8154.6 through Playwright 1.65.1 on the local macOS host. Engine source/compiler/config fingerprint: `da0a405ce81f4a7d83d92c6bec5c7400068ffed17992a5f2dbf2966d04639df0`. Built WASM SHA-256: `8d02af4b0d1e2020c4f3e8091b0ef29a23396776964cebd9abb1761ca06c9218`. Production output was 541.53 kB gzipped WASM and 203.66 kB gzipped JavaScript; existing build-tool/chunk-size advisories remain. These are correctness/build measurements, not gameplay performance results.

## Scope and remaining terrain work

This is a native correction; no moving-platform browser scene or moving-terrain native/WASM trace is claimed. Existing flat-ground browser regressions remain applicable. Terrain work is split into independently verified increments in the implementation plan. [Open terrain scenarios](../verification/terrain-gaps.md) retain the initial slope, stair and reversal failures and the exact setup needed for the next slice. None of those scenarios, human playability, or the broader MVP release criteria is complete.
