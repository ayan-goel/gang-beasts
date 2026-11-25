# ADR 0006: Bounded relative pose control before standing assistance

Date: 2026-11-24. Status: accepted. Specification §§11.3, 12, 27, 29.

## Decision

The native `PoseRagdoll` diagnostic composes the existing passive world with tick-scoped rotational PD actuation. It targets the identity relative joint rotation (the authored bind pose) and zero relative angular velocity. It adds no world-upright torque, linear force, transform changes, or extra constraints. Standing support and selectable targets are subsequent slices. The browser continues to expose the passive rig.

All joints sample the same pre-actuation body state. Each computes `current = inverse(parent_frame) * child_frame` from actual world joint-frame rotations and transforms `child_angular_velocity - parent_angular_velocity` into the parent frame. The orientation error is `target * inverse(current)`, normalized and hemisphere-canonicalized, with a lexicographic tie-break at an exact half-turn. Its shortest-arc rotation vector and the relative velocity therefore share coordinates.

The PD request is `Kp * rotation_error - Kd * relative_velocity`. Locked-axis components are removed before limiting the vector magnitude to the joint's torque cap. The result is transformed to world coordinates and multiplied by the fixed 1/60-second tick. Equal and opposite angular impulses are applied to the connected bodies. This preserves total angular momentum during actuation; relative pose control cannot stabilize absolute airborne orientation. Contact and joint-constraint reactions are separate solver effects and are not bounded by this actuator cap.

`motor_torques()` reports the last tick's commanded world-space child torque in character-definition joint order, in N·m. Parent torque is its negative. It is zero initially, after reset/enable changes, and after a disabled tick. Disabling leaves no persistent actuation. Reset rebuilds the world, clears pending commands and diagnostics, and preserves the enabled setting. The fixed diagnostic topology preserves joint insertion order; later mutable joint lifecycles must use an explicit anatomical registry.

## Tuning contract

`config/motors.json` is strict schema 1 JSON, limited to 8 KiB. Model metadata is `explicit_pd`; every anatomical joint appears exactly once, addressed by child body part. Fixed wrists require zero gains and effort. Fields carry units in their names:

| Field | Units | Accepted range |
| --- | --- | --- |
| `stiffness_nm_per_rad` | N·m/rad | 0–200 |
| `damping_nm_s_per_rad` | N·m·s/rad | 0–20 |
| `max_torque_nm` | N·m, whole vector | 0–50 |

Values must be finite. These ceilings constrain imported effort; they do not promise stability for every accepted combination. The bundled modest gains are tested on this anatomy at 60 Hz. Explicit damping can become unstable on low-inertia bodies if increased without testing. Standing, impact feel and recovery require additional calibration. No reference game's physical gains are claimed. The engine fingerprint includes the embedded configuration; no dependency changes are required.

## Alternatives and consequences

Rapier 0.26.1's generic-joint angular motor evaluates separate quaternion-component axis errors. This slice uses an explicit rotation-vector controller to meet §11.3's full-orientation convention and a single vector effort budget. If supported control later requires an implicit solver, preserve the frame/effort contract and verify the replacement against these fixtures.

Persistent torques require clearing and could leave stale effort when control switches off. Tick-scoped impulses give exact effort lifetime. Applying impulses while evaluating later motors would make shared-body damping depend on iteration order; evaluation completes before application. Nonfinite calculated effort fails closed to zero, but this does not repair or certify a nonfinite world; broader invalid-state handling remains a later engine requirement.

The [Rapier force/impulse guide](https://rapier.rs/docs/user_guides/rust/rigid_body_forces_and_impulses/) documents persistent forces and impulse semantics. APIs and solver behavior were checked in pinned **0.26.1** sources: `dynamics/rigid_body.rs::apply_torque_impulse`, `dynamics/joint/generic_joint.rs::JointMotor::motor_params`, and `dynamics/solver/joint_constraint/joint_constraint_builder.rs::motor_angular`. Current online examples may target newer versions; native tests exercise the pinned version.
