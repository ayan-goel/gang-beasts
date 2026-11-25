# ADR 0005: Versioned anatomy and a passive native rig before motors

Date: 2026-11-24. Status: accepted. Specification §§10–12, 27, 29.

## Decision and boundary

`config/character.json` defines the fifteen bodies and fourteen joints. The shared engine validates it and creates dynamic Rapier bodies with one mass-bearing collider each. No auxiliary collider contributes mass. The native `PassiveRagdoll` scenario owns a finite floor, one rig, a fixed tick counter and one pending diagnostic impulse. It shares `PhysicsWorld` with the cuboid fixture. The internal rig builder is separate so later character/world management can reuse it without adopting the diagnostic scenario.

This slice proves passive anatomy and collision behavior before enabling motors. The browser still displays the cuboid until slice 04b exposes the rig and its diagnostics. No locomotion, multiplayer, knockout, gripping or release-ready stability is implied.

## Version-one character contract

| Field/convention | Contract |
| --- | --- |
| Schema | Strict JSON, version 1, at most 32 KiB, unknown fields rejected |
| Units/frame | Meters, kilograms, radians; right-handed +Y up and −Z forward |
| Bind pose | All body rotations and both local joint-frame rotations are identity; positions/anchors use the same character-local frame; spawn adds translation |
| Body order | `BodyPart::ALL`: head, chest, pelvis, paired upper arms, forearms, hands, thighs, shins, feet |
| Body identity | Anatomical enum/index inside a rig, not a global network ID or exposed Rapier handle |
| Shapes/mass | Spheres for head/hands, capsules for torso/limbs, rounded boxes for feet; fixed per-part masses from §11.1 total 40 kg |
| Geometry | Finite centers/anchors within ±2 m; positive shape dimensions in 0.01–0.4 m; anchor within each body's bounding radius plus 5 cm |
| Topology | Each non-pelvis body has exactly its anatomical parent; duplicate, missing and foreign joints rejected |
| Joint axes | Translation locked. Elbows rotate about X in the positive direction; knees about X in the negative direction. Wrists fixed. Other joints have bounded motion about X/Y/Z |
| Limits | Finite angles within ±3 radians containing the bind pose; hinge/locked-axis pattern cannot be changed through data |
| Contact material | Initial friction 0.7 and restitution 0; calibration candidates, not recovered reference-game constants |
| Motors | None in the passive rig |

Identity frames are a versioned limitation, not missing quaternion data. Nonidentity bind rotations/local axes require extending the definition and deriving anchors/axes consistently for both physics and rendering. Full-body skinning offsets remain a later asset concern. Geometry dimensions and angular ranges are original tuning proposals; the bundled rig is 1.6 m tall with feet at local Y=0.

Local anchors are the authored anchor minus each body's bind position. Generic impulse joints support the varied angular constraints without adding multiple joint construction paths. Exact zero intervals lock an axis rather than installing degenerate limits. Only adjacent body contacts are disabled through joint settings; other same-character limbs and different characters retain contacts. This preserves future physical fighting and avoids disabling all character collisions through a shared category mask.

The drop scenario validates configuration before world creation. Spawn origins are finite, X/Z within ±4 m and Y within 0–10 m. Actual transformed collider AABBs must fit the 12×12 m floor footprint and stay above its surface, allowing 5 mm contact tolerance. This catches assets whose origin is valid but whose bodies are embedded below the floor. These checks do not claim general arena spawn or self-overlap validation.

Queued impulses use the existing finite 10 N·s diagnostic cap and are consumed once before the next step. Reset rebuilds bodies, joints, solver caches and the pending command. The shared engine fingerprint now includes the embedded character JSON so changing the asset invalidates recordings from a different source build.

## Alternatives and evidence sources

A capsule controller would bypass the required anatomy. Enabling all motors immediately would hide whether failures originate in geometry, anchors or actuation. Multibody joints add lifecycle complexity when future grips form closed loops; impulse joints follow the existing spec decision.

The [Rapier joints guide](https://rapier.rs/docs/user_guides/rust/joints/) explains joint degrees of freedom and local anchors. Its current examples target 0.36, so API signatures were verified against the installed **0.26.1** crate sources: `dynamics/joint/generic_joint.rs` (`GenericJointBuilder`, locked axes, limits, local anchors, contacts), `dynamics/joint/impulse_joint/impulse_joint_set.rs` (insertion), and `geometry/collider.rs` (shape builders, explicit mass, AABB). Versioned docs.rs URLs were unavailable through the browsing tool. Tests exercise the compiled pinned API directly.
