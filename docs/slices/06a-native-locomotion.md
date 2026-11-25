# Slice 06a: Native assisted walking and running

Status: verified on November 24, 2026. Browser locomotion and terrain gates remain open.

## Contract

[ADR 0008](../decisions/0008-native-locomotion.md) owns the command distribution, support selection, gait, heading, braking and preset transitions. `LocomotionRagdoll` exposes construction with validated physics/movement configuration, finite world X/Z intent and running flag, fixed step, enable/reset, selected-body diagnostic impulses, the underlying read-only pose and movement diagnostics. No raw Rapier handles or mutable world are exposed publicly.

Configuration comes from `config/locomotion.json` and `config/locomotion-motors.json`. Both participate in the engine build fingerprint. The standing preset, passive rig, explicit motor diagnostic and browser scene list retain their contracts. Production modules stay focused: the wrapper owns lifecycle, the controller computes assistance, the gait module generates targets, and strict schemas own bounds.

Movement state reports total commanded acceleration, selected surface velocity, observed foot support, phase, bounded yaw torque, gait angles in definition order and active-motor blend. These are controller diagnostics, not measured net world acceleration or replay checkpoints. Contact forces and internal joint constraints can change actual velocity.

## Test-driven corrections

The new integration tests initially failed on the missing locomotion API. The first implemented command tipped the rig below the head-height gate at tick 24, even with gait disabled. Isolated calibration distinguished force application from gait failure. A split, bounded impulse with lean feedback addressed pitching. The original coupled hip/knee sinusoid lacked a forward touchdown and a raised return; it was replaced by separate stance and swing paths. Knee compensation at the hip/ankle produced actual alternating foot clearance.

The standing leg gains collapsed under movement loads. A separate stronger implicit preset passes the motion fixtures. Keeping that strength indefinitely resisted the idle shove fixture, so the controller now restores the standing gains after settling and relaxes after support loss. Braking and weaker idle damping remove residual motion without a positional tether. No test height/speed/clearance threshold was reduced to accept those failures.

Independent review strengthened two initially incomplete checks: wall behavior now requires actual contact and total kinetic energy, and the gait fixture requires repeated alternating swings with opposite-foot support. Review also found accepted zero/exact-one settling endpoints that could never satisfy strict release conditions; their failing regression now rejects those values.

## Native acceptance fixtures

| Fixture | Verified property required by the test |
| --- | --- |
| Walk/run, 120 movement ticks after 60 idle ticks | Head >1.1 m throughout; finite states; >2 m forward displacement; final 30-tick mean pelvis speed in (1.6,3.0) m/s walking and (3.2,5.0) running; anchors <0.05 m; joint effort within current caps; command ≤12 m/s² |
| Actual foot motion | Both feet rise above 0.11 m; ≥4 alternating raised-foot episodes with >0.025 m opposite-foot height difference and observed opposite-foot support |
| Four cardinal directions | >1.5 m intended displacement in 90 ticks, head >1.1 m while moving; after 120 release ticks pelvis >0.6 m and horizontal speed <0.2 m/s; no return to spawn |
| Idle and impacts | 1,200 idle ticks above 1.15 m head height and within 0.2 m lateral drift; three consecutive 10 N·s chest impulses then topple below 0.8 m |
| Stop/preset lifecycle | Movement release eventually restores exactly the standing motor configuration; reset repeats the fresh trace, clears input, and preserves disabled behavior |
| Airborne/disabled | Airborne drive and yaw are zero; an initially airborne rig never gains active leg strength; disabled bodies match the passive rig field-for-field |
| Applied command | Independently summed body momentum change equals `40 kg × command × dt`; sum of impulse magnitudes ≤8 N·s; no Y impulse or transform changes |
| Moving support | A 0.8 m/s translating platform is followed without doubling its velocity while walking forward; point-velocity test includes support rotation |
| Sustained wall input | Actual active contact with the inserted wall; 600 ticks with finite state, no passage, individual speeds <12 m/s and total character kinetic energy <1,000 J |
| Ledge departure | Previously active strength decays to zero and standing gains return over the configured release interval; drive/yaw vanish; airborne X/Z momentum is conserved over 20 further ticks |
| Intent/schema boundaries | Maximum finite diagonal values normalize identically to unit diagonal input; invalid updates preserve intent; unknown, oversized, nonfinite, impossible settling and out-of-range tuning rejected |
| Motor/gait geometry | Nonzero target changes the physical restoring direction with correct momentum; raised return clears the authored two-link leg while stance remains extended |

The finite test platform spans 12 m. The forward walk/run fixture starts at Z=4 m so its two-second running interval does not accidentally test falling off the opposite edge. Ledge departure is an explicit separate fixture. Wall energy bounds establish this scenario, not long-duration engine stability. Gait evidence permits sliding; it does not establish locked feet, exact stride length, smooth terrain adaptation or human playability.

## Reproduction

```sh
cargo test --locked --release -p brawler-engine --lib --test locomotion --test locomotion_config
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix web test -- --run
npm --prefix web run build
npm --prefix web run test:e2e
```

All **72 native tests**, **12 TypeScript tests**, and **20 Chromium regression tests** passed. Rust formatting, Clippy with warnings denied, strict TypeScript and the rebuilt production WASM/browser bundle passed. The final strengthened takeoff/strength-decay assertion also passed in the unoptimized native test build. Independent review found no remaining source blocker after the corrections above; its lifecycle/support/braking wording corrections were incorporated into the ADR.

In the native release fixture, final-window mean pelvis speeds were **2.2399 m/s walking** and **4.3065 m/s running**, with **five** and **four** alternating supported-foot swing episodes respectively. The browser regressions verify the existing cuboid/passive/standing scenes after shared-engine changes; they do not establish locomotion WASM parity or visual quality. The rebuilt payload measured 528.88 kB gzipped WASM and 201.04 kB gzipped JavaScript; existing raw-chunk and wasm-pack notices remain. Temporary calibration probes are removed. No browser walking demonstration or performance benchmark is claimed by the native tests.

## Next gates

Add the locomotion WASM contract and playground controls with native/browser trace agreement and visual inspection. Extend fixtures for slopes, steps, rotating support, support transitions and rapid reversals. Human assessment must examine sliding, turning, stopping and interactions under external loads. Then implement jump/fall/recovery before combat. All specification §33 release gates remain required.
