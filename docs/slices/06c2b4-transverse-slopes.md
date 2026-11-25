# Slice 06c2b4: Backward and cross-slope traversal

The native backward and cross-slope traversal checkpoint passes all 121 library and 40 integration tests. Stair clearance and terrain transitions remain open in slice 06c2b4. Baseline for the investigation was `d30f63b`. [Final controller and verification](../decisions/0015-terrain-locomotion.md)

## Historical investigation before swing completion

The working candidate combines a smooth lateral-gait clearance envelope, projected contact-load estimates during active movement and stopping, continuously blended braking allocation, and a correction for erroneous cuboid face normals. No torque, impulse, posture, distance or deadline limit has been increased. The force estimates and gait changes remain provisional; the normal defect has an isolated production-path reproduction and focused verification in [Support-face normals](../verification/support-face-normals.md).

The pre-swing release library run passes **112/114 tests**, with two physical failures and zero ignored tests. All **40 integration tests** pass. [Support-relative settling](../verification/support-relative-settling.md) records the latest correction and seven new regressions for completed motion, support transitions and fixed seams. [Joint effort allocation](../verification/motor-effort-allocation.md) records the preceding change: per-axis implicit motor ceilings prioritize predicted effort while preserving each configured vector budget. Three new real-physics regressions cover isolated position/damping demands, mixed loads and locked axes, and new impacts after a zero-demand sample. The previous isolated saturation assertion changes from equal-axis capacity to full directional capacity; its whole-vector and momentum limits remain intact.

All **48/48 forward/backward/cross-slope traversal cases** now pass, including the existing backward/cross-slope release and desired-foot checks. All varied flat/slope stops and twenty-second short moving-deck stops pass. Two established failures remain: sustained rotating traversal/release (+.4 rad/s, radius zero, +X running, release index 80) and rotating-support impact compliance. Independent impact enumeration finds both −X spin cases survive; six other cases topple. Neither gate is waived.

A matched spine-only diagnostic rescued the prior downhill fall without increasing the vector effort ceiling. Independent hybrid controls identify the new rotating-stop failure as dependent on posture reached before release, not simply the post-release motor allocation. Neutral pose smoothing and a whole-trunk movement gate were tested and rejected. See the motor allocation record for exact controls and remaining work.

[Internal pose continuity](../verification/support-loss-pose.md) preserves the preceding 101/104 checkpoint and [settling/support velocity](../verification/settling-and-support-velocity.md) the earlier 97/101 checkpoint. Those historical results do not establish current acceptance.

The traversal tests query final requested foot poses every tick, including blended releases, rather than only testing full-stride static samples. No desired-foot penetration was found in the 32 completed cases. A prior candidate had 15.585 mm overlap during release when the feet received inconsistent queried normals on one broad plane. This does not certify actual collision-free motion or continuous swept targets.

Reproduce with `cargo test --locked --release -p brawler-engine --lib`, `cargo test --locked --release -p brawler-engine --test '*'`, and `cargo clippy --locked --workspace --all-targets -- -D warnings`. Current logs: `.cache/relative-settling-final-library.log`, `relative-settling-integration.log`, and `relative-settling-clippy.log`. Browser/WASM and human gates remain pending.

## Required fixtures

`transverse_slope_tests.rs` extends the slope fixture to choose character and terrain headings independently. Original forward-slope tests retain identical headings and geometry. Foot clearance is checked before stepping. Cross-slope commands are verified perpendicular to the horizontal surface normal, so they follow a contour rather than climbing/descending.

Each matrix contains 16 independent cases: ±15° inclination, four world quarter-turns, walking/running. Settle 120 ticks, hold input 240, then release 360. Backward begins with input opposite initial facing; the existing controller may turn toward that command. Preserve vertical head clearance >1.1 m, finite state, joint separation <.05 m, force/torque caps, progress ≥70% of four seconds at target speed, terminal speed <.2 m/s, complete motor relaxation, and zero terminal stopping/support moments. Catch case panics only to finish collecting all independent failures; the test still fails if any case fails.

The shared slope gate now also explicitly checks support-moment magnitude ≤120 N·m. Two additional geometry regressions reconstruct both feet from the authored joint anchors and final relative targets, then query the actual rounded colliders for penetration. The flat test samples 16 movement headings ×120 phases. The terrain test covers the same grid on ±15° planes facing forward and across the character. Their 10 µm tolerance accommodates numerical contact error; it does not permit the centimeter-scale overlaps reproduced below. These are sampled desired-pose checks, not guarantees about swept or physically achieved poses.

Baseline: backward **1/16** passes (only +15°, quarter-turn 1, running); cross-slope **12/16** pass. All eight cross-slope walks pass. Cross-slope running fails at −15° quarter-turns 1/2/3 and +15° quarter-turn 0. The first backward failure is −15°, heading 0, walk, total tick 201 (held 81). First cross-slope failure is −15°, heading π/2, run, total tick 229 (held 109).

## Measured failure mechanisms

### Coupled yaw and lean

In the first cross-slope failure, foot propulsion produces 42–52 N·m orbital yaw at ticks 185–195 while the intentional turn controller opposes it at its 20 N·m cap. At tick 215 orbital yaw reaches approximately 109 N·m. The pelvis develops substantial lateral and forward lean, both supports drop out, and head clearance subsequently fails. This identifies coupling, not proof that canceling the yaw alone is sufficient.

### Restoring assistance disappears near target speed

The candidate with transverse lean feedback improves the matrix but retains a downhill backward walk failure (+15°, heading 0). At total tick 180 its speed is 2.48 m/s against a 2.5 m/s target; requested forward acceleration is .48 m/s² and foot assistance only .60 m/s². The supported-foot midpoint trails COM by .453 m while pelvis-up Z is .649. The lag grows to .566 m by tick 185.

All 61 supported pre-failure ticks have a nonnegative velocity/acceleration dot product, with maximum forward speed 2.4841 m/s. This rules out an overspeed-braking/sign correction for this case. The scalar active controller bounds forward foot force by the shrinking net acceleration even as lean grows. Transverse-only feedback cannot supply the missing longitudinal force pair.

### Upper torso can fail while the pelvis looks stable

The full contact/lean target fails one −15°, quarter-turn 3 backward run at tick 249. Pelvis-up Y is .958 and pelvis plane clearance .755 m, but chest/head up Y fall to .704/.610 and head clearance to 1.096 m. A passing comparison has pelvis clearance .784 m and head clearance 1.380 m. The head differs by .284 m while pelvis clearance differs by only .029 m. Preserve chest/head pose and motor effort in future traces; pelvis tilt alone is inadequate diagnosis.

The failing spine reaches its pitch-axis motor ceiling, 48/√3 =27.7128 N·m, by tick 240. At tick 249 its local pitch is −.5005 rad, at the anatomical limit. The neck pitch effort is only 6.94 N·m against a 10.39 N·m axis ceiling. Increasing the desired spine angle cannot provide additional pitch effort under the current allocator.

### Desired feet overlap; physical contact is one contributor

On the unchanged gait, direction 22.5° from forward at phase zero produces 3.40865 cm of exact rounded-foot penetration. The final terrain-adapted targets at −15°, forward terrain heading, produce 2.64878 cm penetration in the same sampled direction/phase. Both reproduce without stepping a moving character. Opposing lateral sweeps can place the feet inside each other; the nominal center spacing is .22 m while each rounded sole is .20 m wide.

The full-contact uphill failure also has measured foot-to-foot solver impulses: 4.14 N·s at tick 226 and 4.86 N·s at 243. A preceding floor landing is 20.78 N·s at 225. A candidate that separates nominal targets rescues this particular run, but still records a 7.44 N·s foot-to-foot impulse at 229 and regresses other fixtures. Thus overlap is a concrete target defect; removing it alone neither guarantees physical clearance nor establishes the complete cause of every fall.

### Contact selection makes naive stance-rate feedback discontinuous

Replacing pelvis tilt/rate with COM-to-supported-foot displacement/relative velocity, normalized by vertical height and corrected for the authored resting offset, does not preserve the gates. In a projected-load uphill run, switching from two feet to one at ticks 223–224 changes estimated X lean rate from −.503 to −5.584/s while height changes only .660→.658 m. The existing .2 s damping gain translates the rate jump into roughly 10 m/s² of additional requested foot acceleration. The sampled heights remain .608–.757 m, so this is not a near-zero denominator. The run stays up but travels 9.007 m against the 12.6 m requirement. Support selection and gait motion cannot be treated as a continuous torso-tilt sensor without a justified transition model.

## Rejected experiments

All experiments preserved the bundled absolute linear command limit and attempted to bound additional moment by 120 N·m. None met the combined regression gates. Only the provisional combination described above remains in the working tree; it is not release-ready.

| Experiment | Terrain matrix / full library evidence | Reason not accepted |
| --- | --- | --- |
| Project foot force to remove orbital yaw | 0/32; 68/88 library | Removes needed longitudinal foot propulsion. Cross-slope failure moves from tick 229 to 191 and becomes predominantly pitch collapse. |
| Hip yaw target after terrain adaptation | 12/32; 80/88 library | Changes placement and sole orientation after IK; several old fixtures regress. |
| Leg-axis twist with recomputed ankle target | 81/88 library | Still fails both new matrices and regresses old slopes, stops and rotating traversal. |
| Cancel command-induced yaw with pelvis torque | 3/32; 80/88 library | Does not cancel stance-contact yaw or root/limb differential rotation. All eight formerly passing contour walks fail. |
| Transverse support-normal feedforward | 82/88 library | Worsens the first cross-slope walk and changes prior movement fixtures. |
| Transverse pelvis lean/rate, scaled by requested acceleration | 24/32; 83/88 library | Fixes every −15° case; remaining longitudinal lean and post-release relaxation fail. Existing deck/impact/stopping regressions remain. |
| Same transverse feedback scaled by gravity | 80/88 library | Stronger feedback is not a general correction; adds prior regressions. |
| Full contact/lean target with capped correction from old foot command | 28/32; 83/88 library | All cross-slope cases and all walks pass, but four backward runs and old reversal/deck/impact/stop cases fail. Startup allocation and torso folding change. |
| Preserve original traction and subtract already-applied scalar lean | 78/88 library | Does not preserve the combined stopping/terrain/rotation gates. |
| Fade startup motion toward requested direction | 82/88 library | Introduces new slope and release failures; no accepted improvement. |
| Full target using mass-weighted trunk orientation | 79/88 library | Regresses established movement/stopping behavior; averaging trunk axes is not an adequate placement controller. |

Further allocation and geometry experiments, all rejected:

| Experiment | Evidence | Reason not accepted |
| --- | --- | --- |
| Full contact drag with lean/rate scaled by requested acceleration | 28/32; 81/88 library | All contour cases pass, but stopping and rotating-support behavior regress. |
| Preserve the original propulsion projection while adding the full contact/lean correction | 28/32; 84/88 library | Four backward runs fail; original forward-slope, rotating traversal and impact gates regress. |
| Use actual foot slip to estimate drag | 20/32; 80/88 library | Swing/stance velocities do not provide a sufficient replacement for the original allocation. |
| Add the COM-to-foot gravitational lever to acceleration-scaled tilt feedback | 27/32; 81/88 library | Forward slopes and rotating traversal/impact regress. |
| Preserve net acceleration and interpolate foot correction into the remaining budget | 24/32; 83/88 library | At full net acceleration the feasible force-pair budget collapses to a line; needed transverse/beyond-net correction disappears. Deck gates regress. |
| Same net priority with propulsion-projection preservation | 27/32; 82/88 library | Four uphill backward runs fall; impact, slope and stopping gates regress. |
| Project velocity into the slope tangent and estimate normal load as max(g nY − a·n, 0) | 28/32; 84/88 library | Established forward slopes and sustained rotating traversal recover, but impact, short-deck release and new terrain cases remain red. The estimate also precedes final command scaling. |
| Geometry-based symmetric ankle separation using two-link IK | Flat exact-geometry grid passes; baseline terrain 12/32, full-contact terrain 29/32 | Rescues the traced uphill run, but reversal, stopping and impact fixtures regress. |
| Correct only opposed hip/ankle rolls where exact feet overlap | Flat exact-geometry grid passes; baseline terrain 14/32, full-contact terrain 27/32 | Preserving sagittal angles and knee motion still regresses existing movement fixtures. |
| Replace pelvis tilt/rate with COM-to-supported-foot lean/rate | Projected-load terrain 19/32, 73/88 library; full-contact terrain 23/32, 81/88 library | Contact transitions produce large feedback jumps and lost progress; resting-offset subtraction does not remove gait-phase motion. |

Further placement/force isolation results:

| Experiment | Evidence | Interpretation |
| --- | --- | --- |
| Reweight nonnegative foot forces to minimize commanded yaw | Baseline 13/32, 78/88 library; projected-load 27/32, 80/88 | Preserves total force but adds pitch/roll when the feet have different heights. Removing 21.38 N·m yaw in one trace moved collapse 27 ticks earlier. |
| Smooth whole-cycle lateral envelope derived from sole clearance and leg reach | Both static geometry grids pass; baseline 15/32, 84/90 library | Avoids overlap corrections with abrupt phase boundaries, but does not alone fix balance. |
| Combine the envelope with projected load and preserved propulsion | 30/32, 86/90 library before face-normal correction | Remaining backward cases were insufficient progress and a downhill fall. Release and impact regressions remained. |
| Abruptly remove propulsion preservation whenever acceleration opposes COM velocity | 30/32, 82/90 library | Introduced a perpendicular-steering force flip and additional existing regressions. |
| Continuously blend that transition by squared negative motion alignment | 31/32, 85/90 library before face correction | Recovered the perpendicular case but retained a downhill fall and old release/impact failures. |
| Concentrate the active correction's opposing force on the pelvis | 19/32, 78/88 library | Upright uphill cases miss progress; the same finite impulse budget cannot supply sufficient motion with this allocation. |
| Use pelvis speed instead of COM speed for net acceleration | 23/32, 84/90 library | Recovers one downhill run but regresses rotating traversal and forward slopes. COM and pelvis genuinely differ over the gait phase, but a universal substitution is not supported. |
| Orient the nominal swing leg in the world upright frame | 30/32, 84/90 library; 39/40 integration | Produces 77 intersecting final target samples. The nominal swing foot is still reported supported in 63% of sampled high-lift ticks; phase alone is not a sufficient unloaded-foot predicate. |

The apparent overspeed in the downhill trace was pelvis overspeed, not COM overspeed: at tick 307 COM Z speed is 4.168 m/s while pelvis speed is 4.643 m/s against a 4.5 m/s target. Both undershoot by tick 325. The later normal diagnosis is independent of that feedback choice: an interior cuboid face query was approximately 31° off its true normal even in a two-collider static reproduction.

The 88-test experiment comparisons include the two new traversal matrices and exclude isolated diagnostic probes and the two subsequently added geometry regressions. Matrix counts refer to independent case runs and must not be confused with Rust test-function counts. Several library tests themselves contain substantial fixture grids.

## Reproduction and preserved evidence

```sh
cargo test --locked --release -p brawler-engine --lib slope_tests::transverse -- --nocapture
cargo test --locked --release -p brawler-engine --lib gait_does_not_command_intersecting_feet -- --nocapture
cargo test --locked --release -p brawler-engine --lib
```

At the earlier restored-baseline checkpoint, the release library passed all **86 existing tests** and failed the **four new regression tests** (two traversal matrices, two desired-foot geometry checks), with zero ignored tests, in 8.12 s. It reproduced 15/16 backward and 4/16 cross-slope failures. That baseline checkpoint is historical; the later controller resolved both failures. No tests are ignored and no thresholds were weakened. No browser or release claim is made for this investigation.

Local ignored evidence includes `.cache/transverse-review/baseline.log`, `matrix.log`, `cross-slope-trace.log`, `accel-feedback-dense.log`, `full-contact-first-backward.log`, and `accel-feedback-uphill-control.log`. Candidate source snapshots and parent library logs are also under `.cache/`. Those files are diagnostic conveniences, not release evidence or repository dependencies.

The added geometry reproduction and restored-baseline run are `.cache/foot-placement-red.txt` and `.cache/transverse-geometry-baseline.txt`. Further isolated traces include `full-contact-spine-contacts.log`, `full-contact-clearance-first-backward.log`, and `projected-load-stance-run.log` under `.cache/transverse-review/`. Independent review found no FK, shape, rotation-order or final-target-clamping error in the two new geometry tests.

## Next implementation constraints

Retain the failing fixtures, established forward-slope/reversal/stopping/deck checks, and the real impulse/moment accounting. A replacement must provide longitudinal balance when net acceleration shrinks, preserve useful startup propulsion, and account for torso/foot geometry without trading the failure into another axis or later release. The current pose targets are open-loop with respect to COM/stance geometry; support-aware placement is a remaining design direction, not a verified solution. Do not increase caps, relax head clearance, extend deadlines, or declare a subset complete merely to make this matrix green.

Steps, terrain transitions, browser terrain, human playability, jump/get-up, combat and the remaining MVP remain unimplemented or unaccepted as recorded in the main plan.
