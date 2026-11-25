# ADR 0009: Browser locomotion input and deterministic diagnostics

Date: 2026-11-24. Status: accepted. Specification §§12.1, 26.2, 27. Extends [native locomotion](0008-native-locomotion.md).

## Decision

Expose the existing native controller through `LocalRagdoll.locomotion(configuration)`. Keep a single browser adapter and renderer for passive, standing and locomotion fixtures. Native code owns movement, motor blending, gait and pose conversion. TypeScript owns input collection, focus, diagnostics and the inspection camera. Gravity/solver replacement preserves the fixture kind and controller enable state, while clearing movement and rebuilding the world transactionally.

The diagnostic uses world X/Z directions, matching the native fixture. WASD/arrows and Shift operate only while the canvas is focused. Hold buttons use pointer capture, so release outside the button still ends movement. Space/Enter activate focused direction buttons with distinct internal hold IDs. Ignore keyboard auto-repeat; otherwise a held key could restart movement after a lifecycle clear. Ignore unrelated key releases. Normalize the combined direction, deduplicate key aliases, and never infer speed from event frequency.

Pause, reset, successful tuning, control disable, focus loss, hidden documents and disposal clear intent. Run selection remains a preference, not a held movement command. Removing input listeners precedes the final neutral write. Nested cleanup guarantees GPU disposal and native free even if that write fails. The final gameplay input mapper will supply camera-relative world intent; this development binding is explicitly labeled and is not that product contract.

## Native math and target frames

Rapier's enhanced-determinism feature does not change Rust's inherent `f32::sin` or `f32::atan2`. The new parity trace first diverged at step 75 in yaw; after correcting yaw it first diverged at step 132 in the knee target. Route these calculations through `rapier3d::na::{RealField, ComplexField}`. The pinned Rapier 0.26.1 enables Simba 0.9.1 `libm_force`, whose owning sources route those calls through `atan2f` and `sinf`. No new dependency or numeric tolerance was introduced.

This follows the [Rapier determinism guide](https://rapier.rs/docs/user_guides/rust/determinism/). The live guide covers a newer release; feature behavior was checked in the pinned Cargo metadata and Simba scalar implementations. Do not infer that other current guide feature combinations are valid for our older build.

Expose relative pose targets as native XYZW quaternions, not Euler angles for Three.js to reinterpret. Pale target axes use `parent_world × target_relative`; saturated axes retain the actual child frame. Native motor configuration is read when inspecting, because active leg caps change during movement and relaxation. Caching the standing configuration would display incorrect limits.

## Consequences

The following camera tracks pelvis X/Z at a fixed height and offset to inspect gait; it does not implement multiplayer group framing or follow falls below the arena. On narrow screens the canvas and movement panel occupy separate vertical space so controls do not hide the feet. Diagnostic geometry remains the physical collider rig.

The native/WASM trace, ownership tests, live-cap UI assertions, input lifecycle regressions and independent target-axis unit test verify this boundary. They do not establish general rollback determinism, human playability, terrain support, a complete camera or release readiness. See [slice evidence](../slices/06b-browser-locomotion.md).
