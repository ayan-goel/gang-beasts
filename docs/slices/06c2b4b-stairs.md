# Low stair traversal

Status — failing native regression on verified controller commit `f3ceacc`. The staircase has three adjacent 0.10 m risers with 0.80 m spacing, a 3 m half-width, and a final landing extending to Z = −12 m. The ordinary flat fixture starts with 0.08 m of spawn clearance, settles for 60 ticks, then walks toward −Z for 150 ticks. The fixture checks finite body state, the existing joint and assistance limits, head height above 1.1 m, per-tick pelvis travel below 0.15 m, and final pelvis Z below −3 m and Y above 0.85 m.

## Reproduction

Run `cargo test --locked --release -p brawler-engine --lib walking_climbs_three_low_risers`. The first failing assertion occurs at tick 143 with head height 1.080 m. The 121 established library tests and 40 native integration tests passed on the preceding traversal checkpoint.

| Tick | Measured motion on the verified controller |
| --- | --- |
| 92 | The left foot center reaches Z −1.018 m at Y 0.158 m. Both feet report support at different elevations. |
| 104 and 108 | Neither foot reports support. Pelvis height reaches 0.967 m at tick 108. |
| 128 | Foot centers are near Z −1.64 m. Their leading edges approach the second riser at Z −1.80 m. Pelvis Z is already −1.87 m. |
| 129 | Both feet first register second-riser normal impulses above 0.01 N·s. The right foot receives 13.408 N·s with a straight requested knee. The left receives 2.141 N·s with a −0.391 rad requested knee. Their sole bounding boxes reach Y 0.098 and 0.113 m, respectively, below the 0.20 m tread top. |
| 136 | Both feet report no support. The pelvis has passed Z −2.13 m while foot centers remain near −1.65 m. |
| 140 | The left foot reports support again. Head Z reaches −2.63 m. The feet remain at the second riser. |

## Rejected changes

A bounded downward ray ahead of the foot detected solid, fixed tread tops above the current support plane. Adding the measured rise to the nominal swing ankle target, solving two-link IK and clamping all joint angles still failed at tick 143. Limiting approach velocity to nominal stride length times step frequency caused a fall near the first riser at tick 208. Accounting for two different measured support elevations in the target calculation caused an earlier fall at tick 126. All three trial implementations were removed.

A separate per-tick contact-manifold trace identifies the second-riser interaction at tick 129. The larger impulse goes to the right leg while it is commanded into stance. Its contact normal has absolute horizontal Z component 0.906. Through ticks 130–135 the right knee target stays at zero and the contact continues to oppose forward travel. This explains a limitation of the swing-only trial. Its clearance change cannot directly lift the more heavily obstructed leg in stance. A subsequent controller experiment should coordinate support transfer with clearance of that leg. The fixture remains red until physical traversal and all established locomotion tests pass together.

## Browser reproduction

Select **Test scene → Low stairs** and resume to drive the character with the usual movement controls. Enable **Show colliders** to compare rendering with the physics geometry. The scene starts with a fresh ragdoll at the native regression's 0.08 m spawn clearance. Reset and physics tuning rebuild the same scene.

Both runtimes construct the staircase in the engine. The renderer reads an owned buffer of enabled static cuboids, six floats per box in center XYZ and half-extents XYZ order. The existing flat fixture collider remains disabled in the stair world to preserve the original contact ordering. The original and shared native constructors produced byte-identical JSON for all 211 frames of the default approach, including the initial state and 14 fields per anatomical body.

| Verification | Result |
| --- | --- |
| Native geometry | Ground and all three treads match the specified upper/lower elevations, width and front/back faces. Reset restores the layout. |
| Render geometry | Three.js world-space bounds match the supplied centers and half extents across all axes. A Chromium capture shows all three tread edges. |
| Native/WASM | The initial state and every completed tick meet the 1e-5 maximum absolute difference bound across all 210 body-state fields. Browser reset and configuration replacement both reproduce the full browser trace exactly. |
| Playground | The new scene, movement, pause and reset pass alongside the established Chromium suite. |
| Physical traversal | The existing head clearance assertion still fails at tick 143. |

Run the exact native trace with `cargo run --quiet --locked --release -p brawler-engine --example stair_trace`. Run the browser comparison with `npm --prefix web run test:e2e -- stairs.spec.ts` after building WASM.

## Further controller experiments

A leading-face query with three tread-top samples identified a reachable landing and rejected narrow treads, walls and sensors in isolated fixed-geometry tests. A persistent world-space swing target overextended the leg early in the approach and the rig fell at tick 131. Adding a smooth horizontal swing and a vertical clearance arc delayed the fall to tick 151. Routing drive to the opposite supporting foot also failed at tick 151. The leading foot landed on the first tread while the trailing foot stayed below it and the pelvis advanced.

Sampling the floor directly below each foot still failed at tick 149. A speed limit derived from braking distance to the leading sole delayed failure to tick 180. Synchronizing a prescribed stance sweep with the planned swing failed at tick 133. These experiments ran against the same posture, travel and effort bounds. Their controller changes were discarded.
