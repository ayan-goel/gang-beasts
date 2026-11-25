# Jump, flight and landing

**Scope.** The locomotion and low-stairs playgrounds accept Space while the canvas has focus. The Jump button provides a standard keyboard and pointer action. Pause, focus loss, disabling the controller and resetting the rig clear pending input. The browser passes input to the shared engine. The physics tick owns takeoff timing.

## Input and physics decisions

| Decision | Behavior and rationale |
| --- | --- |
| Capture a press edge | A quick press and release between physics ticks survives for up to six samples, including the first. Keeping Space down through landing cannot produce a second launch. A genuine second press may buffer just before landing. |
| Use the existing foot-support test | At least one supporting foot and an eligible upright posture authorize a launch. After support ends, six unsupported ticks retain eligibility. Grounding after an observed departure rearms the episode. |
| Apply one vertical velocity change | Each body's impulse is its measured mass times 4.5 m/s at the default setting. The 40 kg rig receives 180 N·s total. Existing platform velocity and limb spin carry through takeoff. |
| Bound airborne steering | After an intentional jump, directional input may spend up to 20% of the configured ground acceleration each tick. An independent 1 m/s absolute velocity-change allowance bounds steering across the flight, including reversals. No steering is applied with neutral input. |
| Keep older tuning usable | The nested jump settings in locomotion schema 3 default as a group when absent. Present settings must be complete, finite and within their validated limits. |
| Keep public buffers compatible | Jump input is a separate WASM method. Existing movement and body-state array layouts retain their published lengths. |

## Verification

- The pre-solver takeoff check measures every body. Positions, orientations and angular velocities are preserved while each linear velocity gains 4.5 m/s vertically. The integrated test checks the gravity-adjusted impulse, peak rise, finite state, joint separation and a subsequent two-foot landing.
- An actual departing-platform fixture checks the first, sixth and seventh unsupported ticks. Separate timing tests check each press-buffer boundary, duplicate contact samples, release/repress, consumed holds, cancellation and reset.
- A moving deck supplies 0.8 m/s horizontal carry before takeoff. The test measures less than 0.01 m/s of unintended horizontal velocity change. Alternating steering directions exhaust one finite airborne budget.
- Chromium checks Space, the Jump button, disabled input while paused and native/WASM agreement across all 210 body-state fields for 220 ticks. In the native scripted trace, pelvis vertical speed was 4.339 m/s immediately after the first takeoff and 4.337 m/s after the second.
- These checks cover local movement. Playtest acceptance, physical get-up and online inputs remain open. The ordinary stair-walking regression is tracked independently.

Run the native checks with `cargo test --release --locked -p brawler-engine --test jumping` and `cargo test --release --locked -p brawler-engine --lib jump`. Build WASM with `npm --prefix web run wasm`, then use `npm --prefix web run test:e2e -- jumping.spec.ts` for the browser comparison.
