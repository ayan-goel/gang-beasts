# ADR 0015 — Finish an airborne step and yield when the trunk folds

The native controller now passes the forward, backward and cross-slope traversal grids, the rotating-platform stopping grid and both flat and rotating shove tests together.

## Landing after release

In the failing rotating-platform run, releasing movement erased a forward ankle target while the foot was still airborne. The character landed, slid past its feet, and fell. A matched two-tick change using the bundled 0.25 radian hip amplitude moved the landing from 0.203 m to 0.234 m ahead of the center of mass. Minimum head height over the next six seconds rose from 0.153 m to 1.270 m. A separate fixture exposed the missing placement feedback with identical velocities and opposite offsets from the stance foot.

On release after an advancing gait sample, finish the nominal swing only while the other foot supplies support. Predict horizontal displacement from the stance foot using the current center of mass, its authored resting offset, and support-relative velocity over a horizon of `sqrt(height/gravity)`. The ideal inverted pendulum motivates this time scale. The pose correction begins beyond the projected half-width of the stance foot's bounding box. Rotate the hip toward the predicted displacement within its gait amplitude, preserve knee flexion, and counterrotate the ankle within its anatomical limits. Test the resulting rounded feet for intersection and halve the angle up to seven times if necessary. Failure to find a clear target leaves the nominal pose in effect.

Contact, support loss, a new movement command, controller reset, disable, expiration of braking, or the end of the original half-cycle closes the window. A stationary gait phase and a tap between simulation samples cannot arm it. The correction uses existing joint motors. The directional and timing tests exercise the normal movement input interface.

## Falling and recovery

Swing completion repaired the long rotating stop, but made three flat-ground shoves survivable at one release phase. By the first shove, the corrected swing had already ended. A matched diagnostic rerouted the stopping force to the body during the impacts, keeping net braking equal. Minimum head height fell from 1.119 m to 0.137 m. This located the excess recovery in the assistance acting on the falling body.

Use the lowest up-axis projection of pelvis, chest and head to detect a stumble. Crossing the existing minimum upright threshold disables walking assistance. The same controller waits for every segment to recover to the existing settling posture before assisting again. Typical configurations enter at 0.50 and recover at 0.98. This hysteresis lets a tilted character keep walking until it loses posture, and prevents a brief rebound from reactivating propulsion mid-fall. The standing balance actuator retains its separate budget. Physical get-up remains a later slice.

Scaling all assistance by segment tilt introduced rotating and slope falls. Scaling the stopping couple by articulation alone retained the shove failures. Applying continuous posture weighting to foot forces passed the shove tests but regressed reversal and rotating stops. The retained state transition uses physical posture across contacts and inputs.

## Verification

Run `cargo test --locked --release --workspace`. The release workspace passed 121 engine library tests and 40 integration tests, with zero ignored. This includes 48 traversal cases on ±15° slopes, 32 sustained rotating-platform traversals and stops, shove response, exact target foot separation, loss of support, zero cadence, and the posture transition for each trunk segment. The focused mutation test re-enabled drive at up-Y 0.8 after a stumble and failed as expected.

The preceding terrain corrections, their individual reproductions and alternatives are documented in the [slice investigation](../slices/06c2b4-transverse-slopes.md), [support geometry](../verification/support-face-normals.md), [pose continuity](../verification/support-loss-pose.md), [effort allocation](../verification/motor-effort-allocation.md) and [support-relative settling](../verification/support-relative-settling.md).

The production build rebuilt the WASM module and passed TypeScript checking. All 17 web unit tests and 30 Chromium tests passed, including flat movement, stopping and reversal agreement with native traces. Workspace Clippy with warnings denied and formatting checks passed. The browser terrain scenes remain a separate deliverable.
