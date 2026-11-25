# ADR 0013: Bounded force pairs while stopping

Status: accepted November 25, 2026 for the documented stopping fixtures. This does not close terrain or human-playability acceptance.

## Problem

After some short runs, braking removes horizontal velocity while the feet remain behind the pelvis. A force allocation constrained to the net acceleration direction loses its righting effect as speed reaches zero. The +X run45 and run45/reverse5/release cases reproduce a fall. Earlier successful stop fixtures did not cover these phases.

## Decision

While requested movement is zero, support is valid, posture is eligible and active motor strength remains positive, allocate a horizontal force pair independently of net deceleration. Use existing support friction/normals and pelvis lean/rate feedback. Fade velocity-direction feedback below one tick's acceleration quantum to avoid a discontinuous drag reversal near rest.

For mass `M`, maximum acceleration `A`, strength `s`, and tick interval `h`:

1. Bound the foot command `f` to `A/2 × s`.
2. Measure the rig's mass-weighted COM `c` and the arithmetic mean of supported foot COMs `p` (equal foot force shares).
3. The external moment is `M × (p − c) × f`. Scale `f` to bound the full moment vector, including yaw, to `stopping_moment_cap_nm × s`.
4. Limit net acceleration `a` to `max(A − 2|f|, 0)`.
5. Apply `(a − f) × body_mass × h` to each body, and `f × M × h / support_count` to each supported foot.

The triangle inequality gives `|a − f| + |f| ≤ |a| + 2|f| ≤ A`. The bundled 40 kg rig therefore retains the 8 N·s absolute linear impulse ceiling per tick. No vertical impulse, position target or transform override is introduced. The moment is additional to the existing 30 N·m pelvis balance and 20 N·m yaw budgets; it must be reported separately.

The bundled moment cap is 120 N·m, with strict finite range [0,120]. Locomotion configuration becomes schema 2 with a mandatory field. Schema 1 is rejected rather than silently adding an actuator. To migrate an experimental config, set schema 2 and explicitly choose the cap; zero disables the pair. This changes the stopping controller, not the existing walking/reversal allocation arithmetic.

## Evidence and tradeoffs

The one-third foot allowance was rejected: independent +15° run74 testing reached 1.099714518 m head clearance, below the unchanged 1.1 m gate. The one-half allowance with the strength-scaled cap passed all 84 standard independent cases (minimum 1.118013263 m, maximum terminal speed 0.051071741 m/s at 240 release ticks). The parent regression grid also requires bounded actuation and complete motor relaxation after 360 release ticks.

This controller is more resistant to disturbance during braking. Three consecutive 10 N·s forward chest impulses still topple it at release ticks 0 and 15. At tick60, twelve such impulses topple it in each cardinal direction. After settling, three impulses topple it in each direction. These scripted doses establish finite compliance, not human approval of feel or equivalence to the former controller.

Geometric capture-step, retained-gait-direction and velocity-bias experiments did not preserve the combined flat/slope/reversal fixtures and were rejected. The force pair is an assisted game controller; it is not a claim of biomechanically accurate foot placement.

## Moving-support frame correction

Settling compares pelvis angular velocity with the mean selected-support angular velocity. A rotating deck can carry a settled character at .4 rad/s, above the .3 rad/s quiet threshold in world coordinates. Subtracting support spin fixes that mismatch without weakening the threshold.

For the drag estimate, evaluate each support's surface velocity at the rig COM: add `support_spin × (COM − sampled_surface_contact_point)` to the sampled contact velocity. Comparing different points on a rotating deck produced fictitious drag even when the character co-moved with it. This correction is zero for fixed/translating surfaces. Ordinary propulsion and the exposed support-velocity buffer retain their existing contact-point convention.

Raw relative spin alone left one radius-2 stop at strength .3 indefinitely. Estimating spin from successive solved orientations removed that residual but introduced late posture failures, so it was rejected. Correcting the drag reference resolved the residual while retaining the original raw angular-velocity gate. Sixty-four short deck cases now test both centered/radius-2 starts, translation/rotation, walking/running and cardinal directions.

## Remaining gates

The existing long rotating-deck traversal failure is preserved as an explicitly ignored reproduction and remains a movement-acceptance blocker. The baseline also falls with input held, so changing stopping forces alone does not resolve it. See the exact scenario and command in `docs/verification/terrain-gaps.md`. Cross-slope travel, steps, terrain transitions, backward slope travel, and human movement/impact assessment remain open. Full regression and source review are recorded in the slice record.
