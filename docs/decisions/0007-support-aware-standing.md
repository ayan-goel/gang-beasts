# ADR 0007: Contact-gated balance and solver-based standing motors

Date: 2026-11-24. Status: accepted. Specification §§11.3, 12, 27, 29. Extends [ADR 0006](0006-bounded-relative-pose-motors.md).

## Decision and evidence

`PoseRagdoll` can optionally enable a validated balance controller. Standing uses `MotorConfig::standing()` from `config/standing-motors.json` and `BalanceConfig::bundled()` from `config/balance.json`, with the existing ground-level spawn `[0,0,0]`. The original explicit, low-effort pose preset stays available as a diagnostic. Passive rig behavior remains unchanged.

The first standing attempt with the original pose gains folded below the 1.15 m head-height threshold at tick 80. Increasing explicit gains produced oscillation and still failed sustained standing. The standing preset therefore uses force-based velocity motors solved together with contacts and anatomical constraints. This is a numerical-control change, not added position constraints or transform overrides. The selected preset passed a 3,600-tick stand and yielded to three consecutive 10 N·s lateral chest impulses. These tests establish this flat-floor fixture; they do not establish gait, get-up, human playability or all valid tuning combinations.

## Solver PD model

Motor configuration schema 1 now accepts `explicit_pd` and `implicit_velocity_pd`. Both use the same canonicalized shortest-arc quaternion rotation-vector error in parent joint coordinates and zero desired relative angular velocity. For the solver model:

```text
c = Kd + Kp × tick_dt
target_relative_angular_velocity = Kp × rotation_error / c
Rapier model = ForceBased
Rapier velocity-motor factor = c
per_axis_max_torque = joint_vector_cap / sqrt(number_of_free_axes)
```

Zero `c`, zero effort, or no free axes disables that joint's actuators. The target is sampled once per 60 Hz tick and held over Rapier's internal solver substeps. Implicit velocity feedback adds damping against the velocity being solved rather than the previous tick's velocity. The `Kp × tick_dt` term anticipates position change over the control tick; this is not an exact nonlinear orientation solve. Per-axis caps conservatively bound the torque-vector norm. A single active direction cannot borrow unused effort from other free axes. Fixed wrists remain unactuated.

The solver's physical motor impulses act equally and oppositely on the connected bodies. The separate root balance actuator is intentionally world-referenced, finite and support-gated, as allowed by §12.2. It is applied to the pelvis only; no vertical lift, horizontal tether, COM force, body pinning, gravity override or pose snapping is used.

The standing preset strengthens neck/spine and lower-body gains while retaining the original arm settings. Joint stiffness remains ≤200 N·m/rad and effort ≤48 N·m. Exact per-joint values are in the versioned asset. All motor validation and fixed-wrist restrictions from ADR 0006 still apply. The build fingerprint includes both presets and balance tuning.

## Support and balance contract

Support starts from active narrow-phase pairs involving each foot. Own-body contacts and sensors are excluded, and contact impulse computation must be enabled by the solver flags; a measured nonzero impulse is not required. A fresh Parry shape-contact query uses current body/collider transforms, rather than trusting cached witnesses. This handles both vertical departure and moving sideways off a finite ledge; a supported slide over the same floor still counts. An unsupported shape-query type conservatively gives no assistance.

| Standing setting | Bundled value | Validation range |
| --- | --- | --- |
| Minimum upward support-normal component | 0.7 | 0.5–1.0 |
| Maximum current contact distance | 0.015 m | 0–0.03 m |
| Maximum foot/surface normal separation speed | 0.5 m/s | 0–1 m/s |
| Upright stiffness | 480 N·m/rad | 0–1,000 |
| Upright damping | 4 N·m·s/rad | 0–20 |
| Upright torque-vector cap | 30 N·m | 0–50 |

The standing contact tolerance is separate from §12.3's later 0.15 m locomotion ground-query distance. Surface velocity includes rotation at the contact point. A platform and foot ascending together therefore remain supported. The diagnostic has no contacts at creation; its first tick receives no balance assistance. `BalanceState.feet` records support used for the latest control decision, not a prediction of the next completed pose.

Balance drives pelvis up toward world up without controlling yaw. Quaternion correction handles inverted poses with a deterministic body-right axis when the up vectors are antiparallel. One supported foot halves the full vector effort; zero supported feet removes it. This does not implement a physical get-up sequence. Queued impulses are consumed before support and actuator evaluation, so an upward foot departure reduces assistance on the same tick.

Balance JSON is strict schema 1, capped at 4 KiB, with finite bounded values. Failed configuration is transactional. Removing balance preserves relative pose control; disabling the rig disables both controllers. Reset rebuilds the world, clears pending commands/diagnostics, and preserves tuning and the enabled setting. Re-enabling wakes sleeping bodies because changing Rapier joint motor fields alone does not wake an island.

## Effort diagnostics and time conventions

For explicit motors, `motor_torques()` remains the requested world-space tick torque. For implicit motors it is an **end-frame visualization**: local solver effort rotated onto each completed parent joint frame. It is not a measured solve-time world torque vector or a tick average.

`solver_motor_efforts()` exposes the implicit model's child-side N·m components along the final substep's local parent-joint solver axes, in definition order. Rapier 0.26.1 stores a parent-side impulse from the final substep, so conversion negates it and divides by `tick_dt / solver_iterations`. This assumes the diagnostic's current zero per-body additional solver iterations. A future change to those iterations or Rapier's writeback requires revisiting this conversion. Independent saturated sphere-pair tests cover 1, 8 and 16 solver iterations. A transverse-spin test distinguishes the local components from their completed-frame display orientation.

Cached impulses are cleared before each step. A sleeping island that skips solving reports zero instead of recycling prior effort. Disabling also immediately clears public diagnostics. These details matter for later force overlays and tuning; magnitudes alone would hide a sign, timebase or frame error.

## Primary sources

The [Rapier collision-detection guide](https://rapier.rs/docs/user_guides/rust/advanced_collision_detection/) explains contact-pair ordering and the difference between potential and active contacts. Its current examples target a newer version; implementation uses pinned **Rapier 0.26.1 / Parry 0.21.1** source APIs. Relevant owning sources are `geometry/contact_pair.rs`, `dynamics/joint/motor_model.rs`, `dynamics/joint/impulse_joint/impulse_joint_set.rs`, `dynamics/solver/island_solver.rs`, `velocity_solver.rs`, `joint_constraint/joint_constraint_builder.rs`, `joint_velocity_constraint.rs`, and Parry's `query/contact/contact_shape_shape.rs`. Compiled physical tests verify the version-specific behavior.
