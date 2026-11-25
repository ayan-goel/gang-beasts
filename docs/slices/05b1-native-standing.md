# Slice 05b1: Native support-aware standing

Status: verified on November 24, 2026.

## Scope

The native rig can stand using finite relative pose motors and contact-gated pelvis balance. [ADR 0007](../decisions/0007-support-aware-standing.md) defines the solver model, assets, support rules, ordering, lifecycle and diagnostic coordinate conventions. All production modules remain small and single-purpose. The browser continues to expose the passive rig until slice 05b2 adds the driven WASM contract and inspector.

Standing is a verified physical building block, not playable locomotion. No walking, jumping, get-up, grips, combat, full-game frame-time result, or release acceptance is claimed.

## Fixtures and corrections

- The standing tests first failed on the missing controller contract, then failed behaviorally at tick 80 with weak explicit pose gains. Calibration replaced the standing preset with bounded solver PD; the explicit diagnostic remains independently tested.
- The 60-second stand requires every body-state field to remain finite, head height >1.15 m, pelvis height >0.65 m, head X/Z drift <0.2 m, anchors within 0.05 m, bounded per-joint and root effort, and support on >3,500 ticks. Latest release measurement: **minimum head height 1.411728 m**, **maximum anchor error 0.000332 m**, **3,599 supported ticks**.
- Three consecutive 10 N·s +X chest impulses after two standing seconds topple the rig below 0.8 m head height within eight seconds. The resulting states remain finite. The first 20 ticks of a 2 m drop receive zero world balance assistance.
- A 120-tick standing trace repeats exactly after reset, including support/effort state; pending impulses are discarded. A disabled standing rig matches the passive simulation field-for-field. Invalid balance tuning preserves the existing controller and bodies.
- Contact fixtures cover reversed collider insertion order, floor, wall, ceiling, sensor, own-body rejection, accepted/rejected slopes, distance thresholds, separating feet and matched moving-support velocity.
- Review found that cached normal distance missed sideways departure from a ledge. A failing regression now passes with current-shape contact revalidation; the same test preserves support while sliding over the finite floor.
- A sleeping displaced rig initially stayed asleep when control was re-enabled. The regression now verifies waking and actual restoration without a push. A skipped sleeping solve cannot retain prior effort telemetry.
- Follow-up review reproduced stale effort after disabling and re-enabling while paused. Transitions now clear solver writebacks immediately; redundant enable calls are idempotent.
- A queued upward foot impulse initially reached physics after the balance decision. Its failing regression now passes: command consumption precedes support evaluation and halves assistance immediately.
- Isolated sphere-pair tests independently verify the solver PD equation, restoring direction, equal/opposite momentum and torque caps. They exposed and corrected the solver writeback's parent-side sign and final-substep timebase. Effort units remain correct with 1, 8 and 16 iterations. A transverse-spin fixture verifies the explicitly documented end-frame display convention.
- Direct balance tests verify the two-foot/one-foot/airborne effort ratios, free yaw and finite capped correction at exact inversion. The latter is a torque contract, not a get-up completion test.

## Reproduction and verification

```sh
cargo test --locked --release -p brawler-engine --lib --test standing -- --nocapture
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
npm --prefix web run build
npm --prefix web run test:e2e
```

All **53 native tests** pass in the full workspace, including the 60-second standing fixture in the unoptimized test build. The standing and controller fixtures also pass in release mode. Formatting, Clippy with warnings denied, strict TypeScript checking, the production WASM/browser build and all **16 Chromium browser regression tests** pass. Follow-up independent review found no remaining source blocker after the documented fixes; its two wording corrections were incorporated into the specification and ADR.

The rebuilt browser payload is approximately 512.47 KB gzipped WASM and 199.74 KB gzipped JavaScript. Existing raw-chunk and wasm-pack notices remain. The temporary calibration probe was removed; final gains are versioned data and the standing fixtures reproduce the evidence.

Standing measurements are native only; the passive browser regression does not establish driven native/WASM parity, visible motor targets or human assessment. Those remain required in 05b2. Full-game performance, walking, jumping, recovery, combat and all release gates remain open.
