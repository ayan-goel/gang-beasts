# Physical get-up

The locomotion and low-stairs scenes can recover a fallen character with the normal physics solver. Disable character control, shove the chest, let the body fall, then re-enable control. **Get-up assistance** shows when the controller is active. **Gait and support** reports the commanded force in newtons and moment in newton-metres.

## Entry, effort and handoff

1. A fallen trunk and an upward load-bearing contact on a fixed or kinematic surface permit preparation. Pelvis speed must remain within 0.6 m/s and spin within 1 rad/s relative to that surface for 18 consecutive ticks. This gives the initial impact time to play out. Dynamic props alone do not authorize bracing.
2. Relative pose motors tuck the legs and vary shoulder pitch with the direction the character faces relative to gravity. Side falls retain neutral shoulder pitch. As the pelvis becomes upright, the tuck fades. The gait motor blend and the extra assistance ramp over 30 ticks.
3. The upper body and legs form two mass-weighted groups. Their separation and relative velocity determine tilt and angular rate. A critically damped tilt response and a gravity-moment estimate about the feet request a righting moment. Dividing the cross product of this moment and the group separation by squared separation yields a realizable force pair. A 5 cm minimum separation excludes the singular case.
4. The solver receives equal and opposite linear impulses across the two groups, distributed by each body's mass. The pair can rotate the character while its total linear impulse is zero. Ordinary contacts provide the reaction needed to rise. The resulting moment and each side's force are capped separately and reported after clamping. The default 360 N·m moment accommodates the gravitational moment of the 40 kg horizontal rig about its feet. The 600 N force ceiling limits loads as the effective lever shortens. The nested schema-three recovery tuning allows lower caps, including zero, and a response time from 0.2 to 1 second.
5. Loss of eligible support suppresses the ground-assist pair that tick. Four seconds bound one attempt, followed by one second of rest after timeout. Releasing an obstruction permits a later attempt. An upright trunk on two feet for twelve ticks yields control to ordinary locomotion. Disabling or resetting the rig clears recovery state.

## Constraint ownership

A fixed-pelvis restraint test initially panicked when the pose motor enumerated every constraint in the world and indexed the fourteen-joint character definition with the fifteenth index. Rig creation now retains the fourteen generational joint handles in anatomical order. Motor setup, enable/disable, explicit torque application, solver reporting and the joint inspector use those handles. A world constraint can be inserted or removed independently of the skeleton.

## Verification

| Scenario | Evidence |
| --- | --- |
| Front, back and either side | Begin with a physically settled, disabled ragdoll. Re-enable it, check finite states and joint separations throughout, then require both feet and an upright head, chest and pelvis for the final 61 ticks. |
| Upper obstruction | A fixed ceiling at 0.60 m blocks standing. A 390-tick trial checks actual collider clearance and bounded assistance. The character stands after removal. |
| External restraint | A separate fixed constraint holds the pelvis in its fallen pose while the controller attempts recovery. Position error stays below 0.05 m and rotation error below 0.1 rad. Releasing the restraint allows a supported stand. |
| Actuator contract | Per-body velocity differences reproduce the reported moment and force, preserve transforms during actuation and sum to zero net linear impulse. Losing the floor or disabling control clears effort. A dynamic prop alone supplies no bracing permission. |
| Native and browser | One 10 N·s chest shove with the controller disabled produces a real fall. The scripted 480-tick trace covers falling, release, recovery and standing. The browser UI exposes the active state and head height through the same transition. |

Run `cargo test --release --locked -p brawler-engine --lib recovery_tests` for the physical fixtures. Build WASM, then run `npm --prefix web run test:e2e -- recovery.spec.ts` for the browser trace and visible controls. This verifies conscious recovery in local fixtures. Combat knockout timing, actual player grip constraints, online play and human assessment remain open.
