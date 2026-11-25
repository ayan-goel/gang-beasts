# Open terrain and movement scenarios

Recorded November 25, 2026, starting from commit `f36979a`. These are failed exploratory acceptance scenarios, not completed features or tests being waived. Slice 06c1 addresses moving supports only; the scenarios below remain required by subsequent terrain slices.

All use Rust 1.88.0, Rapier 0.26.1, bundled physics/character/standing/balance/locomotion configurations, 60 Hz fixed ticks, and no RNG. The preserved temporary source is `.cache/terrain-exploration.rs`; the parameters below are the persistent reproduction record. Future tests must retain meaningful displacement, posture, finite-state and effort gates rather than accepting collapse or slowing movement until the fixture passes.

## Slopes: native uphill/downhill fixtures addressed, other terrain open

[Slice 06c2a](../slices/06c2a-native-slopes.md) and [ADR 0011](../decisions/0011-slope-gait-and-braking.md) record the native correction. Committed tests use a 0.08 m spawn and explicitly prove both soles start clear; the original 0.04 m spawn could penetrate the uphill toe. They cover 60-second stands and four-second walk/run traversal on ±15° slopes, rotating the character and plane together into four world orientations. They also cover stopping, chest impacts, removed support and the command budget. This is uphill/downhill coverage, not cross-slope or browser acceptance.

The original controller failed the corrected +15° idle fixture at tick 115. Following actual shin tilt instead of the desired leg pose only delayed collapse, failing the extended stand at tick 374. The successful pose generator keeps gravity-up/heading as its reference, adjusts leg reach for the slope and preserves bounded ankle targets. The initial downhill release failed at tick 277; bounded counteracting foot impulses corrected the release-braking case without increasing the absolute impulse ceiling.

Next: cross-slope travel with plane rotation about world Z while the character initially faces −Z; entry/exit between flat and tilted surfaces; different supported normals under the two feet; start/stop at other gait phases; native/WASM slope scenes; human movement assessment. These scenarios remain required and must retain meaningful posture/travel/contact gates. No result is yet claimed for them.

## Low steps

Spawn at the origin, settle 60 ticks, then walk toward −Z for 150 ticks. Three risers begin at Z = −1.0, −1.8 and −2.6 m, with heights 0.1, 0.2 and 0.3 m. Half-width is 3 m. Use adjacent tread boxes and extend the final landing to Z = −12; a short landing ending at −6 could eliminate a successful walker before the last assertion.

The initial overlapping-slab probe tripped by movement tick 78. Repeat with the corrected adjacent-tread/long-landing geometry before selecting a fix. Retain head >1.1 m, final pelvis Z <−3 and Y >0.85 m, and per-tick pelvis displacement <0.15 m alongside effort/finite-state gates. The flat gait's ideal symmetric 1 rad knee flexion raises a 0.62 m leg by only about 0.076 m, less than the riser. A bounded forward tread query and swing-clearance correction need implementation and verification.

## Repeated reversals

On the ordinary flat fixture, settle 60 ticks. Alternate +X and −X input every 45 ticks for eight legs, separately walking and running. Require head >1.1 m throughout and final pelvis X velocity in each new direction >0.5 m/s, while maintaining the existing acceleration/effort caps and joint-error checks.

The original walk case failed on the fourth leg, at tick 43 of that leg. Smoothing the gait direction delayed but did not resolve collapse, so that prototype was not retained. Gait phase/contact transitions and near-antipodal heading behavior need diagnosis. Do not replace the fixture with slower turns or weaken the posture gate.

## Browser and human gates

After native terrain fixtures pass, expose matching browser scenarios, compare native/WASM trajectories, inspect soles/targets/contacts visually, and obtain human movement feedback. Current screenshot inspection and flat-floor automation do not establish terrain or game-feel acceptance. All specification §33 release gates remain open.
