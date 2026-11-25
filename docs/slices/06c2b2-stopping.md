# Slice 06c2b2: Varied stopping phases

Status: verified November 25, 2026 within the listed stopping fixtures. Movement and MVP acceptance remain open. [ADR 0013](../decisions/0013-stopping-force-pair.md) records the controller and explicit actuation tradeoff.

## Behavioral gates

- Reproduce +X run 45/release and run 45/reverse5/release falls.
- Preserve head clearance >1.1 m, finite body state, <5 cm joint error and existing motor/balance/yaw limits.
- Verify four cardinal directions, walking/running and varied release phases; include ±15° forward slope travel.
- Settle below 0.2 m/s and return motor strength and stopping moment to zero.
- Measure actual per-body impulse and resulting orbital angular momentum, including zero net acceleration; preserve 8 N·s and the configured moment cap.
- Preserve disable/reset/support-loss semantics and impact compliance.
- Expose an owned three-float world-space stopping-moment buffer without changing the existing eleven-float movement buffer.
- Verify native/WASM traces and the visible diagnostic.

## Current evidence

The original config-cap test failed on schema 1. A 30 N·m configured-cap test measured 29.706 N·m at strength .25 before enforcement (allowed 7.5 N·m); the capped implementation passes. The browser contract initially failed because `stopping_moment()` did not exist. The TypeScript inspector test initially failed because no moment was reported.

Native stopping coverage includes the two original cases; 32 cardinal walk/run phase cases; 84 additional flat/±15° forward cases; zero budgets and low-speed continuity; actual impulse/moment accounting; disable/reset/support loss/resumed intent; and chest-impact doses at multiple brake phases. Original head and effort thresholds are retained. The 32-case grid uses walk durations 45/54/63/72 and run 45/51/57/63 ticks. The 84-case grid uses 32/40/45/52/60/74/90 ticks, both speeds, four flat directions and forward ±15° slopes, after 60 settling ticks and before 360 released ticks. The 64 deck cases use centered spawn / settle 60 / move 32 or 45 and radius-2 spawn / settle 120 / move 24 or 45, both speeds, four directions and either .8 m/s translation or .4 rad/s rotation.

The eight-case native browser-fixture trace uses default spawn and finite platform, 60 settling ticks, walk/run durations 45–72 ticks, optional five-tick reversal and 360 released ticks. It records all 15 bodies, the unchanged movement buffer and separate stopping moment each tick. Native results currently have minimum head height 1.2687 m, terminal speed below .006 m/s and zero final motor strength. The Chromium parity test passed with every component within 1e−5. The rendered UI test also observes stopping effort rise above 1 N·m after releasing movement, return to zero with settled motor strength, and remain zero after reset. The owned-buffer test retains a nonzero stopping vector across configure/free.

## Moving support: correction and remaining traversal failure

A new rotating-deck regression reproduces motor strength remaining nonzero after release: the old settling gate compares absolute pelvis angular speed with .3 rad/s while the platform rotates at .4 rad/s. Subtracting support spin and correcting the drag estimate to compare velocities at COM resolves the short-stop cases. A solved-orientation derivative alternative caused late falls and was rejected.

A separate case uses the default [0,0,0] spawn, a 6 m half-extent kinematic deck at Y=-.5 with .4 rad/s world-Y rotation, 60 settling ticks, +X run60 and release. Head height falls below 1.1 m at release tick 11, before the 60-tick braking interval expires. It remains an explicitly ignored failing regression; no posture threshold was reduced. Independent baseline runs also fall with movement held, making longer moving traversal a distinct open gate. Run it explicitly with the command in the terrain-gap record.

## Verification evidence

Both full debug and release workspace runs passed **118 tests** (78 library, 40 integration), with **one explicitly ignored known moving-traversal failure**. Running that reproduction explicitly still fails at release tick 11. Formatting, Clippy with warnings denied, 17 TypeScript tests and the production WASM/TypeScript/Vite build pass.

All 30 distinct Chromium tests passed across the full run and focused follow-ups. The first full run passed 28/29 tests; the pre-existing passive-ragdoll screenshot test reached its 30-second timeout while debug physics tests were running concurrently. Both passive UI tests passed unchanged on an isolated one-worker rerun (10.0 s). The strengthened ownership test, stopping parity test and new visible-effort test passed together (17.6 s). No timeout or physical threshold was increased.

Independent review found no remaining controller/API blocker after the support-frame corrections and requested direct rendered-effort verification, now covered by the passing UI test. The known long moving traversal gate remains open.

Environment: Rust 1.88.0, Rapier 0.26.1 with enhanced determinism, Node 20.18.0, Playwright 1.65.1 / Chromium 159.0.8154.6, local macOS. Engine source/compiler/config fingerprint: `a9145f332096de80dc95fa049f9280f0b13a11a5db8b537f178b99be1544d834`. WASM SHA-256: `4de68957a306ebb03e0643dd972b62d47f0d247b3985c84234ad8a49978d53d4`. Locomotion JSON SHA-256: `2822c18110c6be4f4d80fe7c20e3992463a1d2833a863315ae499c23500bfd68`. Production gzip sizes: 546.57 kB WASM, 203.73 kB JavaScript. Existing wasm-pack-update and Vite chunk-size advisories remain; these are not full-game performance results.

Desktop 1440×900 and narrow 390×844 screenshots were visually inspected: the separate stopping-moment value and unit remain readable; no page errors were captured. This is visual QA, not a human movement playtest.

## Verification commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test --workspace --locked --release
npm --prefix web run test -- --run
npm --prefix web run build
npm --prefix web run test:e2e -- --workers=2
npm --prefix web run test:e2e -- ragdoll-ui.spec.ts --workers=1
npm --prefix web run test:e2e -- stopping.spec.ts locomotion-contract.spec.ts --workers=1
```

The known failing reproduction has its own command in the terrain-gap record. No §33 release gate is closed by this slice.

## Subsequent correction

[Slice 06c2b3](../slices/06c2b3-rotating-traversal.md) resolves the formerly ignored rotating-traversal regression and extends short deck stops to twenty seconds. Its schema-3 support compensation and moment diagnostic are recorded separately; the evidence above describes this earlier slice.
