# ADR 0014: Bounded compensation for rotating support

Status: accepted for the native traversal fixtures, November 25, 2026. Browser terrain and human feel remain open.

## Problem

A turning platform supplies changing horizontal contact velocity below the character COM. Sustained cardinal movement can retain forward speed while developing lateral lean and falling. After short stops, idle carry can also tip the character hundreds of ticks after active motors relax. Replacing all propulsion with vector feedback changed established slope and stopping behavior; prolonging active motors did not address the load.

## Decision

Retain the existing propulsion and stopping allocation. Add a separate horizontal foot/body force pair for the estimated acceleration of a rotating support's velocity field. For each selected support, evaluate `omega × (v_COM − v_support_body)`, average the values, then discard the vertical component. This derivative assumes constant angular and linear support velocity. It is a feedforward estimate, not measured contact force and not a guarantee for arbitrary accelerating or tilting mechanisms.

Let its horizontal magnitude be `q`, direction `d`, the existing net acceleration `a`, and existing total foot command per rig mass `f`:

1. The existing distributed body command is `a − f`. Subtract its projection along `d` from the estimated contact load.
2. Clamp the residual to `[0,q]`. The added foot command is `e = −d × residual`; equal opposite distributed body assistance preserves the requested net acceleration before normalization.
3. Use actual mass-weighted rig COM `c`, total mass `M`, and mean supported foot COM `p`. Limit `|M × (p − c) × e|` to `support_moment_cap_nm`.
4. Define `f_total = f + e` and `scale = min(1, A / (|a − f_total| + |f_total|))`, for maximum acceleration `A`.
5. Apply `(a − f_total) × scale × body_mass × dt` to each body and `f_total × scale × M × dt / foot_count` to each supported foot. Scale both moment diagnostics consistently.

The sum of commanded absolute linear impulses is at most `M × A × dt`, still 8 N·s for the bundled rig. Combining both applications at a foot can reduce its measured net impulse further. No vertical assistance or transform override is added. Zero angular velocity naturally gives no correction; the clamped residual also tends to zero continuously with rotation. No scene identity or held-input gate selects the correction.

The new moment cap bounds only this incremental pair, not the entire preexisting propulsion moment. It is additional to the stopping pair (up to 120 N·m times active strength), 30 N·m pure pelvis balance and 20 N·m intentional yaw. The support pair can remain active at settled idle. It clears on reset, disable, missing support or ineligible posture. Human assessment must account for its added resistance to impacts.

## Configuration and inspection

Locomotion schema 3 requires `support_moment_cap_nm`, finite in [0,120], bundled 120 N·m. Schemas 1/2 are rejected. Migrate experimental schema-2 configurations by setting version 3 and explicitly choosing this cap; zero disables the new pair. The existing stopping field remains mandatory.

Native `LocomotionState.support_moment` and WASM `support_moment()` report the incremental world-space COM moment after combined normalization. The WASM buffer is owned, has three floats for locomotion, and is empty for other fixtures. The eleven-value movement buffer is unchanged. The playground shows the support-moment magnitude separately from stopping and yaw; its current flat scene correctly reports zero.

## Alternatives and limitations

Unsubtracted support acceleration passed held traversal but caused late falls after release. Signed residuals could remain nonzero as rotation approached zero and regressed ordinary fixtures. Clamping to the estimated remaining contact load avoids both defects. A general vector propulsion replacement was rejected because it regressed existing uphill/downhill and stopping fixtures.

Force-pair isolation verifies actual changes in body momentum and orbital angular momentum, including one-foot support and saturated caps. Translation-invariance tests first prime the idle carry reference: translating instantaneous velocities without transforming that history compares different controller states. These tests establish the compensation behavior after matched history, not invariance of a fresh controller's zero-reference startup.

The exact centered-deck twelve-shove fixture topples in all eight spin/direction cases. Shifting the shove by one tick or spawning at radius 2 leaves one of those cases standing. This phase-dependent resistance is recorded, not generalized into an all-impact or human-playability claim. See the slice record for exact gates and evidence.
