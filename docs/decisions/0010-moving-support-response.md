# ADR 0010: Bounded response to support acceleration

Date: 2026-11-25. Status: accepted for native fixtures. Specification §§12.1, 12.2, 14.4. Extends [locomotion](0008-native-locomotion.md).

## Problem and decision

The original idle response chased support-relative velocity over 0.5 s. A rotating deck carried the feet faster than the torso caught up, eventually tipping the character. Replacing idle damping with the active 0.05 s response fixed the ride but prevented the standard chest impulses from toppling it. Stronger joint motors were unnecessary.

Keep the existing idle gains, damping and impact response. Add a small `SupportMotion` module that follows changes in horizontal support point velocity. It tracks a reference velocity, not a desired position or accumulated position error. No new tuning or dependency is introduced.

For each supported upright tick, with timestep `h`, existing maximum acceleration `A`, horizontal support velocity `s`, and reference `r`:

```text
d = clamp_magnitude(s − r, A h)
r = r + d
predicted_velocity = COM_velocity + ordinary_controller_acceleration h
fraction = clamp(dot(r − predicted_velocity, d) / dot(d, d), 0, 1)
carry_delta_velocity = d × fraction
```

Zero `d` returns no carry correction. Clipping against the updated bounded reference accounts for motion already supplied by contact and ordinary damping during that increment. For example, a +0.2 m/s reference increment with +0.15 m/s already supplied needs only +0.05 m/s. Using the final deck speed instead over-counted partial transfer during starts/stops; a review regression guards the corrected expression.

Only idle control adds `carry_delta_velocity / h`; active movement/braking already uses the fast support-relative response. The reference advances in both modes so switching to idle cannot replay old deck motion. The final combined acceleration remains bounded by `A`; idle application remains mass-proportional across the rig. With the bundled 40 kg/60 Hz/12 m/s² configuration, total assistance stays at most 8 N·s per tick. Neither this term nor existing locomotion adds vertical lift or modifies transforms. Joint and balance torque caps are unchanged. The reference clears on reset, disable, support loss or insufficient upright posture.

This remains game movement assistance, with no equal/opposite reaction on the support. It is not a no-slip attachment, a physically complete traction model or permission to cancel arbitrary external impacts. Once the bounded reference matches a constant horizontal support-point velocity, the carry term is zero; the existing softer idle damping remains responsible for relative motion.

## Support lifetime

Rapier 0.26.1 updates body/collider enabled flags and removes collider arena entries before the next physics pipeline rebuilds narrow-phase pairs. Controller queries occur before that rebuild. Therefore support sampling checks the current foot and peer collider flags, current parent-body enabled state, and fallible peer lookup. Disabled/sensor feet, disabled surfaces/parents and deleted colliders cannot provide one stale tick of assistance or cause a missing-handle panic.

These semantics were checked in the pinned owning sources `geometry/collider_set.rs::remove`, `geometry/collider.rs::{set_enabled,is_enabled}`, and `dynamics/rigid_body.rs::{set_enabled,is_enabled}`. A test-only world helper removes colliders through Rapier with the world's actual island/body sets. Gameplay deletion APIs and full entity-reference cleanup remain later work.

## Consequences

Native tests verify bounded start/stop increments, partial/full contact transfer, cardinal deck rides, rotating support in platform-local coordinates, knockdown under chest impacts, reset, and momentum-preserving departure after disabling or deleting support. The browser still exposes only the flat locomotion fixture. Slopes, steps, repeated reversals, browser terrain scenes and human playability remain open gates; this decision does not mark terrain acceptance complete. [Slice evidence](../slices/06c1-moving-supports.md).
