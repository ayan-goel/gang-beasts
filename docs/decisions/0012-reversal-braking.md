# ADR 0012: Brake existing motion before reversing

Date: November 25, 2026. Status: implemented and verified within the fixture scope recorded in [slice 06c2b1](../slices/06c2b1-reversals.md). Specification §12 remains canonical.

## Evidence

Eight alternating +X/−X legs, each lasting 45 ticks, exposed a fall at tick 225 while walking and tick 127 while running after 60 settling ticks on a flat fixture spawned 0.08 m above its plane. In the running case, pelvis yaw speed reached about 8 rad/s within five reversal ticks while the body was still travelling in its original direction. Turning and stepping immediately against that momentum lost the supported stance.

Changing the drive allocation or adding yaw compensation alone did not resolve the physical tests together. Reusing the existing neutral brake before turning passed the original posture and direction-response gates. This does not require a higher acceleration, motor or balance limit.

## Transition

`ReversalBrake` samples the latest normalized request once per supported physics tick. It remembers the last accepted direction and a bounded neutral-history lifetime. An obtuse request relative to that direction arms at most the configured `braking_ticks`; holding the same request does not renew the timer.

The braking weight is `max(−departure_direction · requested_direction, 0)²`. Squaring the cosine makes the transition continuous at a right-angle turn. A full reversal has weight one; ordinary acute steering has zero. During the transition:

- Scale effective movement intent and supported yaw by `1 − weight`; blend the gait toward that effective intent.
- Blend the friction-direction estimate toward release braking and allow foot share down to `−weight`.
- Retain the absolute impulse normalization from ADR 0011: the bundled 40 kg character receives at most 8 N·s of summed absolute commanded linear impulses per tick, with no vertical command.
- Resume the latest requested direction, analog amount and run flag when opposing support-relative COM speed is no more than `maximum_acceleration × tick_dt` (0.2 m/s bundled), or the timer expires. Accept the latest direction on the final timer tick so a held request cannot immediately rearm it under a persistent external load.

This is a temporary controller transition, not a queued movement command. Neutral returns zero braking weight, cancels the pending transition, and leaves the ordinary explicit release brake in charge. Direction history survives at most `braking_ticks` neutral ticks; the current requested vector remains authoritative. Returning to the departure half-plane cancels the catch. Support loss, insufficient upright posture, reset and disable clear both transition and history. Disabled public control resets the controller before later re-enabling.

## Keyboard boundary

Releasing one movement key and pressing its opposite can produce `+X → neutral → −X`, sometimes entirely between two physics ticks. Event-based detection incorrectly erased history on the intermediate neutral value. Detect transitions at the physics boundary instead. Brief neutral gaps also retain recent direction, while an expired gap begins a fresh request.

The browser publishes `running=false` for neutral input even when Shift remains held. Applying that flag to the decaying gait changed cadence for one tick and exposed a late fall at default-spawn tick 416. Retain the last nonzero movement cadence through neutral braking; the next nonzero request immediately selects its requested walk/run cadence. Neutral still commands no movement. Native and browser traces use the actual false neutral flag. No DOM or render-frame clock enters the simulation.

## Limits and alternatives

This adds no dependency, configuration field, public API or buffer-layout change. History is deterministic simulation state and will need inclusion in future complete checkpoints; the existing diagnostic transform buffers are not rollback state.

The tests cover eight 45-tick reversals, four rotated world orientations, walk/run, the default spawn, short neutral gaps, updated analog requests and bounded-state contracts. They do not establish arbitrary turning on stairs/slopes or human-tested feel. A distinct full-release test after a 45-tick sideways run already failed without any reversal; its feet remain behind the pelvis after braking. That stopping/capture deficit stays required in [terrain scenarios](../verification/terrain-gaps.md), along with cross-slope travel and steps. No posture or speed threshold was relaxed.
