# Support-relative completed rotation

Status: focused native correction within unaccepted [slice 06c2b4](../slices/06c2b4-transverse-slopes.md), November 25, 2026. This supersedes the settling-frame limitation in [the earlier investigation](settling-and-support-velocity.md). It does not establish physical traversal on arbitrarily accelerating platforms.

## Defect and measurement

The previous gate subtracted current support angular velocity from pelvis rotation measured over the preceding tick. A pelvis rotating rigidly with an accelerating platform could appear to have relative spin; conversely, a world-stationary pelvis could appear settled after the platform stopped despite relative rotation during that interval. The two isolated regressions fail the preceding calculation.

`PoseMotion` now stores pelvis orientation relative to the selected support body's orientation at each completed tick. For pelvis orientation `P`, support orientation `S`, and fixed timestep `dt`, it measures the shortest rotation from `S_previous⁻¹ P_previous` to `S_current⁻¹ P_current`, divides by `dt`, then maps the result to current world axes with `S_current`. Both orientations therefore cover the same interval. This is an equivalent completed-tick rate, not a replacement for instantaneous solver velocity in force damping.

The sampling contract is:

- One or two selected feet on the same moving body use that body's material frame. A change from two feet to one preserves history if the reference body is unchanged.
- Fixed parent bodies and unparented static colliders share the world frame. Distinct fixed tiles must not invalidate a valid interval merely because their handles differ.
- Different moving support frames have no unique common material orientation. They clear history and use current pelvis spin minus mean selected-support spin.
- New reference bodies, including recycled indices with different generations, start with current relative spin. Initial, repeated and skipped ticks also use the instantaneous fallback.
- Missing support, reset, disable and ineligible posture discard the interval history.

No force budget, motor gain, upright threshold, spin threshold, relaxation rate or public diagnostic layout changed. The additional support body handle is internal to the controller.

## Test evidence and limits

Four new small tests cover acceleration, hidden relative movement during deceleration, noncommuting reference rotations with world-axis output, and reference/tick discontinuities. The existing two completed-pose tests remain. Three production-controller tests additionally exercise actual foot-support selection and observe motor-strength decay through `LocomotionRagdoll::step`.

Those controller fixtures prescribe completed poses and velocities to isolate the settling predicate. They verify shared-deck acceleration/deceleration, support replacement, mixed support means, reacquisition, one-foot continuity and a fixed-body seam. They are not free-running stability demonstrations. A mutation restoring world-interval minus endpoint-spin fails acceleration and new-support cases. Before fixed-parent normalization, the seam test held strength at `0.96666664` instead of decaying to `0.9333333`; it passes after the correction.

At this earlier checkpoint the native library passed **112/114 tests**, with zero ignored tests. The same two physical failures remain: rotating-support impact compliance at spin −0.4 with −X impacts, and sustained +0.4 rotating-deck +X running followed by a fall at release index 80. All 48 forward/backward/cross-slope cases, varied flat/slope stops and twenty-second short-deck stops still pass. Native integration passes **40/40**; formatting and workspace Clippy with warnings denied pass. Browser/WASM regeneration and human verification remain pending. This correction does not close the locomotion slice.

Reproduce with:

```sh
cargo test --locked --release -p brawler-engine --lib pose_motion
cargo test --locked --release -p brawler-engine --lib settling_support_tests
cargo test --locked --release -p brawler-engine --lib
cargo test --locked --release -p brawler-engine --test '*'
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Local diagnostic logs are `.cache/relative-settling-{red,seam-red,final-library,integration,clippy}.log`; independent controller mutation checks are `.cache/transverse-review/settle-deck-{legacy-red,contract,fixed-green,sibling-green}.log`. Maintained source and tests provide reproduction without those ignored files.

The subsequent [terrain locomotion decision](../decisions/0015-terrain-locomotion.md) records the resolution of both physical failures.
