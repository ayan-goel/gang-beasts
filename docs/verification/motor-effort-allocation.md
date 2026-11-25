# Joint effort allocation and remaining rotating-support failures

Status: unaccepted native candidate within [slice 06c2b4](../slices/06c2b4-transverse-slopes.md), November 25, 2026. This supersedes the integration checkpoint in [internal pose continuity](support-loss-pose.md). No force ceiling, fixture threshold, observation duration or configuration value changes.

## Causal controls before changing motors

The previous candidate passed 101/104 library tests. Its −15° forward run at yaw 270° began losing posture while still supported, before its first unsupported tick at total tick 235. Holding the preceding internal targets for its reacquisition tick 236 did not move the first head-clearance failure from tick 248. Holding the +15° run74 stop's targets for the first neutral tick also did not prevent failure (tick 162 versus 163). These matched single-tick interventions reject a simple transition-hold explanation for these remaining cases.

A second diagnostic preserved the downhill trajectory through tick 200, then allocated the spine's existing 48 N·m vector budget to pitch only. The physical joint limits remained unchanged; yaw/roll motors were disabled in the diagnostic. Head clearance stayed above 1.20769 m through tick 480, compared with the baseline fall to .16935 m. The accepted equal-axis allocator could supply only 48/√3 =27.7128 N·m to pitch. The same diagnostic did not repair the run74 stop. This establishes that unused directional motor capacity contributes to the downhill failure; it does not justify a permanently pitch-only spine.

## Candidate allocation contract

`implicit_motor.rs` retains the existing solver velocity target, damping model, torque units and equal/opposite internal actuation. Only per-axis ceilings change. Let `C` be the configured joint vector ceiling, `n` its number of free angular axes, `e` the parent-frame shortest-arc rotation error and `w` the current child-minus-parent angular velocity in that same frame:

```text
factor = Kd + Kp × tick_dt
raw_demand = Kp × e − factor × w       (locked axes zeroed)
demand = magnitude_clamp(raw_demand, C)
reserve = max(C² − |demand|², 0) / n
axis_ceiling[i] = sqrt(demand[i]² + reserve)   (free axes only)
```

Zero free axes, zero damping factor and zero total ceiling still disable the joint's actuation. In exact arithmetic the squared free-axis ceilings sum to `C²`; therefore every combination of solved axis efforts stays within the existing vector ceiling. Tests allow floating-point roundoff. An isolated saturated demand can use the whole joint budget. At zero predicted demand, equal spare capacity remains available to damp contact-induced motion during the solve.

This is a pre-solve estimate, not a claim to predict the final coupled constraint solution. A fully saturated estimate can leave no capacity on another axis until the next tick. The actual target velocity and damping remain solver-evaluated. Independent inspection of pinned Rapier 0.26.1's `joint_constraint_builder` confirms its angular motor rows use orthonormal parent-joint-frame axes, matching the budget coordinates. Explicit PD motors are unchanged.

## Regression tests

The first isolated-axis reproduction failed with −1.73205 N·m against the requested −3 N·m single-axis load, while the whole-vector ceiling was 3 N·m. The candidate passes. Three new tests cover:

- Independent X/Y/Z position and damping loads reaching the existing vector ceiling without lateral effort.
- Simultaneous signed loads with three, two or one active axes at 1/8/16 solver iterations, checking the norm of configured ceilings, actual solved effort, inactive-axis zero effort and conserved angular momentum.
- Impacts arriving after a zero-demand sample, verifying that each axis retains damping capacity.

The existing saturated-effort/substep test now expects the full 3 N·m for its isolated axis instead of 3/√3. Its impulse-duration and equal/opposite momentum assertions remain intact. This is an explicit allocator-contract revision; it does not increase the configured whole-vector limit. All six focused motor tests pass.

## Rotating-stop diagnosis

The full candidate fails the +0.4 rad/s deck, radius zero, +X run after 240 movement ticks: head clearance crosses 1.1 m at release index 80 (total tick 381). An independently matched hybrid used the old allocation through total tick 300 and switched to demand allocation only on release; minimum head height remained 1.343 m. The reciprocal hybrid used demand allocation through tick 300 and old allocation during release; it still fell to .157 m. Each hybrid matched its reference body's trajectory exactly before switching.

At release, the failing candidate's COM was only .088 m behind the foot midpoint, versus .166 m for the passing old allocator. By total tick 355, COM was .41 m ahead of the loaded footprint, versus .12 m in the passing case. The subsequent fall involved pelvis, chest and head together; spine pitch effort was approximately 8 N·m, not saturated. The stopping pair remained at its 120 N·m ceiling. This identifies a dependency on the posture and foot placement reached before release, rather than a simple post-release motor instability.

A diagnostic that used demand allocation only at the spine preserved all 48 terrain fixtures and the sustained rotating-deck panel but introduced a brief-neutral-gap reversal fall. It is not retained. First-order smoothing of all neutral pose targets regressed the slope-stop and flat-impact gates and did not repair the rotating stop; it was also reverted.

### Subsequent stop experiments

The following experiments were applied separately to the 107-test allocation checkpoint and rejected. The later support-relative settling correction changes measurement only; it retains none of these pose/force experiments.

| Experiment | Result | Reason rejected |
| --- | --- | --- |
| Return toward a terrain-adapted neutral stance with first-order smoothing | 103/107 pass | Rotating fall moves only from release 80 to 84; slope release and flat impact compliance regress. |
| Add knee crouch proportional to horizontal support-relative speed during stopping | 102/107 pass | Adds slope-stop, neutral-gap reversal and short-deck failures while retaining both original failures. |
| Remove the extra active tilt/rate correction while retaining drag and carry | 87/107 pass | Improves the failing impact aggregate but loses established movement, stopping and force-allocation contracts. |
| Estimate stopping drag from each foot's actual surface slip | Adds a third production failure | Rotating fall is delayed to release 140, then occurs; a previously passing slope stop fails. The isolated 108-test diagnostic suite includes one extra trace test and reports 105 passes. |

In the rotating-stop trace, soles nearly stop slipping before COM stops moving: at total tick 325, COM-relative X speed is about 1.24 m/s while the two soles slip at approximately .01/.02 m/s. From ticks 351–381, whole-rig momentum accounting gives approximately −.20 N·s of net deck X impulse, compared with +93.43 N·s commanded at the feet and −116.25 N·s distributed over the body. The body command is braking. These observations narrow the remaining problem to the evolving support geometry and available restoring moment; they do not prove a replacement gait controller.

Per-contact tangential diagnostics in the pinned dependency initially contained nonfinite cached accumulator values. An isolated diagnostic initialization made those fields readable without changing the matched trajectory; whole-rig net reaction was checked independently from momentum. No dependency instrumentation is included in production. Do not treat uncorrected cached tangential values as reliable measurements.

Evidence: `.cache/neutral-stance-return-library.log`, `braking-crouch-library.log`, `no-extra-active-lean-library.log`, and `.cache/transverse-review/friction-{clean,actual-long,actual-library}.log`.

## Impact compliance remains unresolved

Independent enumeration runs all eight spin/direction cases instead of stopping at the aggregate test's first failure. With full demand allocation, six topple as required; both −X impact cases survive. For spin −.4, minimum pelvis/chest/head up-Y values are .781/.665/.476; minimum head height is 1.194 m. For spin +.4, they are .840/.578/.206 and minimum head height is 1.125 m.

Applying the existing .5 movement-eligibility threshold to all three trunk segments disables drive during the folded-head interval, but the −.4 case still survives. It only moves the other case to a delayed fall. This experiment is not retained; no threshold or timer was tuned to these cases.

Earlier matched controls on the preceding candidate show that removing the support allocator's tilt correction allows toppling, while removing its longitudinal preservation rule improves recovery. The observed resistance is a combined effect of bounded world assistance, leg motors and entry posture, not a missing incoming impulse. Do not add impulse-specific branches or waive the established overpowering requirement.

## Verification before the support-relative settling correction

The subsequent [support-relative settling record](support-relative-settling.md) reports 112/114 library tests with the same two physical failures. The results below preserve this allocator checkpoint.

- Release library: **105/107 pass**, two failures, zero ignored.
- All **48/48** forward/backward/cross-slope traversal fixtures pass; backward/cross-slope releases and desired-foot geometry checks remain included.
- All varied flat/slope stop phases and all twenty-second short-deck stops pass.
- Native integration: **40/40 pass**.
- The two unresolved library tests are sustained rotating traversal/release and rotating impact compliance, described above.

Formatting and workspace Clippy are checked with the current source. Browser/WASM regeneration, terrain scenes, human movement, steps and all remaining MVP gates remain pending. No implementation commit or slice acceptance follows from this checkpoint.

Reproduce with `cargo test --locked --release -p brawler-engine --lib`, `cargo test --locked --release -p brawler-engine --test '*'`, and `cargo clippy --locked --workspace --all-targets -- -D warnings`.

Ignored evidence: `.cache/demand-allocation-red.log`, `demand-allocation-focused.log`, `demand-final-library.log`, `demand-allocation-integration.log`, `demand-allocation-clippy.log`, `landing-transitions-{baseline,one-tick-control,spine-allocation}.log`, and `neutral-target-filter-library.log`. Independent causal traces are under `.cache/transverse-review/demand-deck-x-*.log`, `rot-impact-*.log`, and `impact-trunk-*.log`. Diagnostic interventions are absent from production and the maintained test suite.
