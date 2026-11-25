# Internal pose continuity across support loss

Status: implemented candidate, November 25, 2026. Part of [slice 06c2b4](../slices/06c2b4-transverse-slopes.md), still unaccepted. Grounding thresholds, actuator budgets, posture/progress requirements and observation durations are unchanged.

## Reproduced landing discontinuity

The −15° backward run at yaw 270° originally travelled 12.29038 m against the 12.6 m requirement. It was not leaving substantial unused propulsion: 220 of 240 movement ticks applied more than 7.5 N·s against the 8 N·s budget. After movement tick 165, the torso oscillated and the spine reached its −0.5 rad limit while its pitch motor saturated at 48/√3 = 27.7128 N·m. By tick 200 chest/head up-Y values were .843/.719 despite an almost upright pelvis.

At tick 165 both feet still had cached active contacts and fresh distances −0.000080/−0.008708 m. Their endpoint separation speeds were +.53673/+1.20512 m/s, above the .5 m/s support limit. The controller correctly removed world assistance, but it also replaced every desired joint rotation with bind pose. The desired sole points moved toward the ground at 5.48/2.93 m/s. Left/right solved normal impulses were 12.87/21.50 N·s. Support returned on the following tick, when desired sole points rebounded upward at 1.71/3.34 m/s.

An independent causal control preserved the exact trajectory through tick 164, then supplied only the preceding internal joint targets for the unsupported tick. World assistance stayed zero, and strength and all other logic stayed unchanged. The left normal impulse fell to zero and the right to 18.98 N·s; the real right-foot impact did not disappear. Immediate COM forward speed increased from 3.58 to 3.92 m/s. The unchanged backward traversal/release matrix passed all sixteen cases under that diagnostic intervention. This was evidence for the target discontinuity, not a production branch keyed to a particular scene or tick.

## Current transition contract

`pose_release.rs::toward_bind` limits each parent-local Euler component toward zero by:

```text
maximum_change = (joint_upper_limit - joint_lower_limit) × tick_dt / motor_release_s
next_target = previous_target - clamp(previous_target, -maximum_change, maximum_change)
```

All validated joint intervals include zero. Thus this operation stays within the interval, moves monotonically toward bind, and completes within the existing release duration in exact arithmetic. Accumulated f32 rounding can require one additional callback to reach exact zero; the tests cover that bound. There is no stored clock or new tuning value. The returned quaternions and inspector targets use the same released angles.

On upright support loss, world acceleration, yaw torque, stopping moment and support moment remain zero immediately. Support-motion and reversal histories clear, and gait blend still resets. Only internal joint pose targets return gradually. Joint motors retain their existing effort caps and reaction pairs; this is not an airborne world-space actuator or a gravity change. Active motor strength separately follows its existing release behavior.

Reset and disable clear targets immediately. Ineligible upright posture also returns bind immediately. Regaining support resumes normal gait/terrain target calculation; this candidate does not smooth every supported pose or guarantee a continuous first landing target.

The pose rate is independent of motor strength. A settled slope can have nonzero ankle targets at strength zero. An earlier strength-proportional candidate snapped a .262 rad stationary ankle directly to zero, and a partially relaxed departure changed an ankle by .201 rad in one tick. Both exceeded the .0333 rad per-tick bound for the ankle's one-radian range. A separate full-duration timer fixed the rate but regressed three downhill held runs. The stateless rate limit passes those backward cases without retaining an additional lifecycle state.

## Regression coverage

- `pose_transition_tests.rs` removes support from a running rig whose knee target is −.50673 rad. The original code returned zero immediately and failed. The current test checks bounded internal target change, returned solver targets, zero world commands, unchanged body state during controller evaluation, and eventual bind pose.
- The same module checks reset, disable and inversion after transient pose retention.
- `pose_release_slope_tests.rs` repeats removal from a +15° slope at motor strengths zero, approximately .333 and approximately .067. Every joint/axis must obey the rate bound through expiry, with no world command or immediate body momentum change.
- The existing removed-slope fixture retains its no-drive, no-world-torque and unchanged-body assertions. Its former immediate-bind assertion now requires bounded target change instead; stopping and support moment assertions are explicit. This updates the internal-pose contract, not the grounding or physical acceptance thresholds.

The first full-strength bound initially failed by 7.45×10⁻⁹ rad when subtracting represented f32 endpoints. The 10⁻⁶ rad comparison tolerance is shared with the all-axis departure fixture and does not hide the original half-radian snap.

## Alternatives tested and rejected

Measured-speed release gait, finishing the current nominal swing, and decelerating a release-time command all recovered the sustained rotating stop but regressed impact compliance and brief neutral input gaps. Retaining the last planned step through upright flight also left the slope stop failing. Those plans remain outside production.

The half-sine knee path has a nonzero endpoint rate. Two smooth alternatives were tested with an endpoint-rate regression: sine squared and smoothstep of sine. Each preserved peak lift and passed that local test, but each left seven library failures and regressed established movement cases. They and their proposed endpoint-rate test were rejected together; production retains the prior half-sine. The supported/unsupported target reset was the stronger causal finding.

Nominal stride is also shorter than commanded translation. The flat 0.25 rad hip sweep over a 0.62 m leg at 2.5 Hz corresponds to about 1.53 m/s relative foot sweep. The actual force-assisted rig travels faster: an independent flat run measured median COM speed 4.27 m/s, target sole touchdown speed +2.66 m/s in world coordinates, and load-weighted actual foot slip 1.64 m/s. This metric describes sliding, not a physical speed cap. No cadence, amplitude, collider, mass, joint effort or world-force budget was changed on this evidence alone.

## Checkpoint before demand-based motor allocation

The following describes the preceding source snapshot. [Joint effort allocation](motor-effort-allocation.md) records the subsequent candidate and current verification.

Release library: **101/104 pass**, zero ignored. All **40 integration tests** pass. Formatting and workspace Clippy with warnings denied pass. Backward and cross-slope traversal each pass **16/16**, including final desired-foot geometry and release checks. The sustained rotating traversal/stop panel and all twenty-second short-deck stops pass.

Three library failures remain:

1. Forward downhill run: −15°, yaw 270°, head clearance approximately 1.073 m at total tick 248.
2. +15° slope stop after 74 running ticks: head clearance fails at total tick 163. The formerly failing 60-tick stop now passes.
3. Rotating-support impact compliance: spin −.4 rad/s with sustained −X chest impulses.

Independent enumeration passes 15/16 forward cases, so the three terrain matrices total 47/48. This does not close terrain transitions, stairs, browser/WASM terrain or human movement acceptance. No implementation commit is accepted from this checkpoint.

Reproduce using `cargo test --locked --release -p brawler-engine --lib`, `cargo test --locked --release -p brawler-engine --test '*'`, and `cargo clippy --locked --workspace --all-targets -- -D warnings`.

Ignored logs: `.cache/pose-slew-final-library.txt`, `pose-slew-final-integration.txt`, `pose-slew-final-clippy.txt`, `pose-continuity-red.txt`, `independent-pose-release-red.txt`, `uphill-budget-probe.log`, `uphill-spine-probe.log`, and `uphill-contacts.log`. Independent traces under `.cache/transverse-review/` include `stride-one-gap-control.log`, `stride-one-gap-backward.log`, `stride-candidates.log`, `stateless-all-slopes.log`, and the `planned-*-library.log` experiments. These diagnostic files are not build dependencies.
