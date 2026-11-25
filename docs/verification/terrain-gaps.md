# Open terrain and movement scenarios

Recorded November 25, 2026, starting from commit `f36979a`. These are failed exploratory acceptance scenarios, not completed features or tests being waived. Slice 06c1 addresses moving supports only; the scenarios below remain required by subsequent terrain slices.

All use Rust 1.88.0, Rapier 0.26.1, bundled physics/character/standing/balance/locomotion configurations, 60 Hz fixed ticks, and no RNG. The preserved temporary source is `.cache/terrain-exploration.rs`; the parameters below are the persistent reproduction record. Future tests must retain meaningful displacement, posture, finite-state and effort gates rather than accepting collapse or slowing movement until the fixture passes.

## Slopes

Replace the flat collider with a cuboid of half-extents (6, 0.5, 10), rotated about world X by +15° and −15°, translated to Y = −0.5/cos(angle). Its top plane is Y = −tan(angle)×Z. Spawn at (0, 0.04, 0), settle 120 ticks, then walk toward −Z for 120 ticks.

The original +15° run had fallen during settling: its head was approximately (0.00024, 0.300, 1.488) at the first movement tick, only 0.698 m above the plane. Require head clearance >1.1 m during settling and walking, and more than 2 m of commanded travel measured from the post-settle pelvis position. Confirm initial toe clearance separately so spawn penetration is not mistaken for a controller failure. The next implementation needs a supported sole target derived from the current slope normal while retaining gravity-upright balance.

## Low steps

Spawn at the origin, settle 60 ticks, then walk toward −Z for 150 ticks. Three risers begin at Z = −1.0, −1.8 and −2.6 m, with heights 0.1, 0.2 and 0.3 m. Half-width is 3 m. Use adjacent tread boxes and extend the final landing to Z = −12; a short landing ending at −6 could eliminate a successful walker before the last assertion.

The initial overlapping-slab probe tripped by movement tick 78. Repeat with the corrected adjacent-tread/long-landing geometry before selecting a fix. Retain head >1.1 m, final pelvis Z <−3 and Y >0.85 m, and per-tick pelvis displacement <0.15 m alongside effort/finite-state gates. The flat gait's ideal symmetric 1 rad knee flexion raises a 0.62 m leg by only about 0.076 m, less than the riser. A bounded forward tread query and swing-clearance correction need implementation and verification.

## Repeated reversals

On the ordinary flat fixture, settle 60 ticks. Alternate +X and −X input every 45 ticks for eight legs, separately walking and running. Require head >1.1 m throughout and final pelvis X velocity in each new direction >0.5 m/s, while maintaining the existing acceleration/effort caps and joint-error checks.

The original walk case failed on the fourth leg, at tick 43 of that leg. Smoothing the gait direction delayed but did not resolve collapse, so that prototype was not retained. Gait phase/contact transitions and near-antipodal heading behavior need diagnosis. Do not replace the fixture with slower turns or weaken the posture gate.

## Browser and human gates

After native terrain fixtures pass, expose matching browser scenarios, compare native/WASM trajectories, inspect soles/targets/contacts visually, and obtain human movement feedback. Current screenshot inspection and flat-floor automation do not establish terrain or game-feel acceptance. All specification §33 release gates remain open.
