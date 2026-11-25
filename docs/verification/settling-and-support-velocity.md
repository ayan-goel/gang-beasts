# Settling and support-velocity investigation

Status: uncommitted integration work within [slice 06c2b4](../slices/06c2b4-transverse-slopes.md), November 25, 2026. These findings do not close movement or MVP acceptance. Configuration thresholds, actuator ceilings and fixture deadlines remain unchanged.

## Completed rotation versus solver endpoint spin

One +15° walk-stop candidate remained at motor strength 0.1667 after 360 release ticks. Both feet stayed supported, head clearance remained above 1.409 m, and COM speed remained below 0.061 m/s after the first eligible settling tick. Of 300 idle decisions, 275 rejected spin alone. Terminal endpoint angular speed was 0.418 rad/s, while the completed-pose rotation divided by the fixed tick was 0.00642 rad/s. During the final 60 ticks the latter had mean 0.0101 and maximum 0.0213 rad/s. The pelvis was not actually rotating at its reported endpoint rate over the complete tick.

The candidate `PoseMotion` uses the shortest world-space quaternion difference between consecutive completed physics ticks, divided by the fixed tick duration. Quaternion sign changes do not change the measurement. Initial, repeated and skipped samples fall back to instantaneous angular velocity; reset, disable, loss of support and ineligible posture discard history. Two unit tests cover these contracts. The measurement is used only for settling. A separate experiment replacing force damping with completed rotation regressed the library to 88/100 and was reverted.

At this historical checkpoint, the settling comparison subtracted selected-support angular velocity. This is consistent for the fixed and constant-spin fixtures. For accelerating supports, current angular velocity and the preceding pelvis interval can refer to different times. Consecutive pelvis orientations in the same support body's frame would align those intervals; support identity changes would need an explicit fallback. The later [support-relative settling correction](support-relative-settling.md) implements that frame-based measurement and verifies its predicate with controlled native fixtures.

## Whole-trunk recovery before relaxation

Completed rotation alone permitted a flat reversal-cancellation fixture to relax too early. At the first differing decision, tick 181, completed pelvis spin was 0.105 rad/s versus endpoint 0.307 rad/s, against the unchanged 0.3 rad/s threshold. Pelvis/chest/head up-Y values were 0.994/0.820/0.783. The candidate reached zero strength at tick 205 while chest and head were still tilted approximately 34° and 38°, then crossed the head-clearance limit at tick 294.

This was not a COM-outside-support decision: at zero strength, COM was 70 mm inside the convex hull of load-carrying contact points. A support-polygon check would have admitted the same premature release. The endpoint-spin comparison retained assistance until tick 267, when the trunk was nearly upright, and remained standing.

The current candidate requires pelvis, chest and head each to satisfy the existing `settle_minimum_up_y` threshold. A production-step regression preserves identical incoming motion and actual two-foot contacts across upright, chest-folded and head-folded cases. The former gate decreased strength in all three; the candidate decreases only the upright case. No posture threshold or release rate was weakened. The separate reversal-cancellation scenario remains in the library.

## Active drag must compare velocities at the same point

The active support allocator compared COM velocity with deck velocity at the support contact. On a rotating body these points have different velocities even under rigid co-motion. The new regression reproduces spurious additional foot acceleration of approximately `[-0.923, 0, 0.880]` m/s².

The candidate extrapolates each selected support velocity to COM using `v_contact + omega × (COM - contact)`, then averages those values before calculating relative drag. It uses the same reference-point correction already present in stopping drag. The regression checks two feet and either foot alone, preserving the existing rotation-compensation command. The subsequent correction below extends same-point feedback to the main controller while retaining the diagnostic buffer's contact-point convention.

## Main feedback and late idle drift

With only active drag corrected, a radius-2 m rotating-deck stop after 24 running ticks fell at release index 897. Strength had reached zero at tick 238 with the trunk upright. The stopping moment remained zero for the next 804 ticks, while both feet remained supported and COM drifted relative to their midpoint. In deck coordinates the offset grew from approximately `[.007, -.038]` m at tick 240 to `[.176, -.489]` m at tick 1042. At tick 800, true COM-relative deck speed was .004 m/s while the controller measured .036 m/s using the contact point.

An independent matched intervention preserved the entire trajectory through tick 700, then changed only fully idle damping and carry feedback to use deck velocity at COM. This prevented the fall: at tick 1344 head height was 1.413 m, strength and stopping moment stayed zero, and the COM offset moved back toward the feet. Peak commanded acceleration during ticks 700–850 fell from 2.525 to .475 m/s². Thus the late failure does not require changing when relaxation occurs.

The current candidate uses mean deck velocity extrapolated to COM for settling speed, reversal feedback, desired velocity, idle damping, and bounded carry. The public support-velocity diagnostic still reports the sampled contact-point mean. `StoppingDrive` still receives contact-relative velocity and performs its own COM correction exactly once. No clock, position anchor, vertical force, cap or deadline changes.

A controller regression freezes supported geometry and sets every body's velocity to the rigid deck field plus the requested movement speed. After priming bounded carry, the controller must not add a net speed correction in either idle or walking cases. The former code produced `[.0625, 0, -.0595]` m/s² in idle; both cases now pass.

The lifecycle fixture explicitly takes one +X running step after its 60-tick settle, then checks nonzero support moment before disabling/resetting/removing support/inverting the pelvis. Its original fixed idle-tick precondition could legitimately become zero when contacts and carry already supplied the required motion. The test still requires moment >.01 N·m before each exit, no residual acceleration/moment afterward, and unchanged bodies for unsupported/ineligible controller calls. No acceptance threshold was relaxed.

The coherent correction passes all short-deck stops, but a sustained rotating +Z run now fails 48 ticks after release. The integration is therefore still unaccepted.

## Rejected stride-history changes

Retaining directional stride targets after neutral input initially avoided a target discontinuity. Preserving that history across held-input support gaps also changed movement before release: it fixed some stops but introduced an additional held backward-run fall. Restricting history to released input restored the earlier held trajectories but did not repair the +15° run60 stop.

An independent A/B comparison started from identical body states at the end of a +X run. After 15 neutral ticks, three 10 N·s chest impacts toppled the original neutral-gait controller to a minimum head height of 0.154 m; retained-direction gait recovered with a minimum of 1.081 m. Both applied approximately 120 N·m stopping moment during the decisive interval. The retained gait had already reduced lean and COM lead over the feet before the first shove. Settling did not cause this divergence.

The stride-history implementation and its two alternative-behavior tests were therefore rejected together. They are archived in ignored diagnostic files, not accepted API requirements. Gait direction again uses current intent and gait blend clears on loss of support. The later internal-pose fade documented separately does not retain a directional stride plan. Established impact tests and all four terrain/geometry regressions remain enabled and unchanged.

## Checkpoint before the support-loss pose correction

This earlier candidate passed **97/101 library tests** and **40/40 integration tests**, with zero ignored tests. [Internal pose continuity](support-loss-pose.md) records the subsequent source changes and current checkpoint. Workspace Clippy passes with warnings denied. The backward/cross-slope matrix passes **31/32 cases**. The library's four unresolved tests at that checkpoint were:

- Rotating-support compliance under sustained −X chest impacts at spin −0.4 rad/s.
- A +15° slope stop after 60 running ticks, crossing head clearance at total tick 143.
- Backward uphill running at −15°, quarter-turn 3: 12.29038 m versus the unchanged 12.6 m requirement; its release is not reached.
- Sustained rotating-deck traversal: spin +0.4 rad/s, spawn radius 0 m, run +Z for 240 ticks, falling at release index 48.

All short-deck stopping cases, including their 20-second idle observations, pass. Removing retained stride history restores the flat impact test. Earlier successful sustained-rotation runs before coherent COM feedback are not current acceptance evidence. Browser/WASM regeneration and human verification remain pending; no commit is accepted from this checkpoint.

Reproduce with `cargo test --locked --release -p brawler-engine --lib`, `cargo test --locked --release -p brawler-engine --test '*'`, and `cargo clippy --locked --workspace --all-targets -- -D warnings`.

Ignored evidence: `.cache/com-reference-final-library.txt`, `com-reference-final-integration.txt`, `com-reference-final-clippy.txt`, `com-drag-red.txt`, `com-feedback-red.txt`, `trunk-settle-red.txt`; independent traces under `.cache/transverse-review/`: `settle-three.log`, `measured-contact-suite.log`, `endpoint-contact-control.log`, `trunk-focus-red.log`, `trunk-focus-green.log`, `impact-parent.log`, `impact-unretained.log`, `deck-default.log`, and `deck-lateboth.log`. Source code and executable tests, not these local logs, are the maintained reproduction route.
