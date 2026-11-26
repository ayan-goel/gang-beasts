# Low stair traversal

Current results appear under [Verified ascent and landing](#verified-ascent-and-landing). The original native regression failed on controller commit `f3ceacc`. The staircase has three adjacent 0.10 m risers with 0.80 m spacing, a 3 m half-width, and a final landing extending to Z = −12 m. The ordinary flat fixture starts with 0.08 m of spawn clearance, settles for 60 ticks, then walks toward −Z for 150 ticks. The fixture checks finite body state, the existing joint and assistance limits, head height above 1.1 m, per-tick pelvis travel below 0.15 m, and final pelvis Z below −3 m and Y above 0.85 m.

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

## Landing and sustained walking

An isolated controller experiment paired a tread-targeted swing with a bounded horizontal force pair. While planning the swing, speed was capped by the available braking distance from the center of mass to the leading sole. The pair retained its existing moment cap. During transfer alone, the net command used the greatest requested-direction acceleration compatible with the fixed foot command and the absolute impulse budget. Ordinary stopping retained its prior allocation.

On the 60-tick settled approach the experimental controller reached pelvis Z −3.254 m and Y 1.028 m at tick 210. The contact readings show one foot supporting the rig on the upper 0.30 m tread at tick 195. At tick 200 the minimum head/chest/pelvis upright projection crossed the stumble threshold. Both feet reported upper-tread support at tick 213, but the forward rotation continued. Head height fell below 1.1 m at tick 220. The original ascent assertion had passed ten ticks earlier.

The worktree now also checks 240 walking ticks followed by 180 ticks at rest. Posture and joint/force limits apply during both intervals. At the end, the pelvis must remain over the landing with speed below 0.2 m/s and both feet must report contact on the 0.30 m surface within 0.015 m. Run both assertions with `cargo test --locked --release -p brawler-engine --lib slope_tests::stairs`.

| Initial settling | Candidate result under the same forward input |
| --- | --- |
| 59 ticks | First head-height failure after 82 walking ticks. |
| 60 ticks | Met the original ascent assertion. First head-height failure after 160 walking ticks. |
| 61 ticks | Remained upright but stopped short. After walking and release the pelvis was at Z −1.471 m. |
| 75 ticks | First head-height failure after 64 walking ticks. |

The candidate passed 123 native library tests and 41 integration tests before the extended landing check was added. Its sensitivity to start time and its failed landing exclude it from the playground. For local comparison, the ignored development cache contains the apply-checked candidate diff and traces under `.cache/stair-transfer-candidate/`. At that checkpoint the playable controller still reproduced the tick-143 failure. Further work needs to coordinate support transfer before the stance foot hits the next riser and arrest forward upper-body rotation through actual landing contact.

## Verified ascent and landing

The shared fighter controller now senses a reachable fixed tread and applies a bounded hip and ankle pose to both legs. The [design decision](../decisions/0016-low-step-pose.md) gives the geometry and tuning. With 60 settling ticks, the fighter walked forward for 240 ticks, stopped for 180 and finished with pelvis Z −7.867 m, Y 1.112 m and speed 0.007 m/s. Both feet had physical support on the 0.30 m landing. The old controller fell at total tick 143.

The committed native tests enforce the established posture, finite-state, displacement, anchor and effort bounds throughout the approach and the stop. They exercise 59, 60, 61, 69, 75 and 84 settling ticks in the default orientation, plus the other three cardinal headings after 60 ticks. Geometry tests reject walls, short and narrow treads and sensors. Chromium compares all 210 anatomical state fields with native on every one of the 481 frames of the default traversal. Reset and configuration replacement each reproduce the complete browser trace.

A wider native sweep used 1 through 120 settling ticks plus 180. In 116 of the 121 cases the head stayed above 1.1 m, the pelvis passed Z −3 m by walking tick 150 and the fighter remained beyond that point at rest with speed below 0.2 m/s. Settling for 19, 25, 26, 43 or 108 ticks produced a fall. With 25 settling ticks, the right foot stayed at the second face as the pelvis moved 0.289 m past it by total tick 97. These cases remain open. Additional swing flexion, retaining the gait blend across support loss and velocity-based braking did not eliminate those failures in isolated comparisons.

Use the default native command below for the accepted approach. Append a settling tick count after `--landing` to reproduce another start, such as `--landing 25`. The command emits completed body states from spawn through the stop.

```sh
cargo test --locked --release -p brawler-engine --lib slope_tests::stairs
cargo run --quiet --locked --release -p brawler-engine --example stair_trace -- --landing
npm --prefix web run test:e2e -- stairs.spec.ts
```

## Build verification

On the Apple M3 Pro development Mac, engine fingerprint `d56bc5bf8973293a8f28e7da7df06fe65f496b82b93fc2eda5e00f5757a7ee4b` passed 282 native tests, 48 web unit tests, the production build, formatting and warning-denied workspace Clippy. The local preview served that fingerprint in both the shipped WASM and the multiplayer map catalog.

The synchronized Chromium suite passed 74 of 77 scenarios, including all stair cases, rematches and disconnect grace. The two-player hazard deadline and a local countdown expired during a run with host sleep. The hazard screen showed 2:04 remaining when the wall-clock wait expired, and the host power log recorded a 52-second sleep in that interval. In the eight-player case, seven sockets kept progressing past tick 1200. The eighth had closed after tick 740, before its movement input. Separate production-preview runs passed all three scenarios using the same engine and map fingerprints. All eight profiles received and displayed the elimination. Two profiles saw each hazard warning, including after a guest reload, and the local match awarded the point and started its next round. The source of the intermittent socket closure is unresolved.

## Inspect a starting timing

Open `/playground/?scene=stairs` or select **Low stairs** in the development playground. In **Inspect a stair approach**, enter the number of settling ticks and the number of ticks since forward movement began. **Inspect frame** rebuilds the current fixture, replays the input and pauses with the selected limb, joints and contacts available for inspection. The first 240 movement ticks hold forward. The next 180 release it. Selecting an earlier frame repeats the solve from spawn. Resuming gives control back with movement released. The two input bounds limit a single inspection to 660 physics ticks.

| Settling ticks | First movement tick with head height at or below 1.1 m | Total tick |
| --- | --- | --- |
| 19 | 115 | 134 |
| 25 | 81 | 106 |
| 26 | 161 | 187 |
| 43 | 126 | 169 |
| 108 | 121 | 229 |

Inspect settling tick 25 at movement ticks 80 and 81 to see the threshold crossing. With 60 settling ticks, movement tick 420 shows the supported stop. Chromium checked the displayed head height and pelvis Z against the native trace at those frames, in reverse order, at movement onset and on both sides of release.

The native integration test runs all 121 start times, including the five above. It reports the first fall per start and fails on any newly affected start. Survivors must pass Z −3 m by walking tick 150, finish beyond that point with pelvis height above 0.85 m and speed below 0.2 m/s, and report both feet supported. The posture, displacement and joint-effort checks on the original cases remain in the native library suite.

```sh
cargo test --locked --release -p brawler-engine --test stair_fixture -- --nocapture
npm --prefix web run test:e2e -- stair-inspection.spec.ts stairs.spec.ts
```

Reducing the ankle response rate to 0.8 times its usual value near a reachable step produced eight falls across 121 starts. Increasing lower-body damping to 1.35 times its usual value within the configured caps produced five falls at a different set of starts. Both comparisons used the same inputs and default terrain. The shipped controller retains the existing motor settings.

On the M3 Pro, 316 native workspace tests, 105 web unit tests and all four Chromium stair scenarios passed. The production bundle passed type checking, and workspace formatting and warning-denied Clippy passed. The local production playground opened directly to Low stairs and displayed a head height of 1.081 m at total tick 106 for the 25-tick start. The browser and room server shared engine fingerprint `781eca65941f285cbf4e7ae2dfe7fd82b5af1b4f33b1b912f78f6f4da14087cd`.
