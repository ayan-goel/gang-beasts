# Open terrain and movement scenarios

Recorded November 25, 2026, starting from commit `f36979a`. These are failed exploratory acceptance scenarios, not completed features or tests being waived. Slices 06c1 and 06c2a cover moving supports and uphill/downhill travel; the reversal work and remaining failures are distinguished below. Open cases remain required by subsequent slices.

All use Rust 1.88.0, Rapier 0.26.1, bundled physics/character/standing/balance/locomotion configurations, 60 Hz fixed ticks, and no RNG. The preserved temporary source is `.cache/terrain-exploration.rs`; the parameters below are the persistent reproduction record. Future tests must retain meaningful displacement, posture, finite-state and effort gates rather than accepting collapse or slowing movement until the fixture passes.

## Slopes: native uphill/downhill fixtures addressed, other terrain open

[Slice 06c2a](../slices/06c2a-native-slopes.md) and [ADR 0011](../decisions/0011-slope-gait-and-braking.md) record the native correction. Committed tests use a 0.08 m spawn and explicitly prove both soles start clear; the original 0.04 m spawn could penetrate the uphill toe. They cover 60-second stands and four-second walk/run traversal on ±15° slopes, rotating the character and plane together into four world orientations. They also cover stopping, chest impacts, removed support and the command budget. This is uphill/downhill coverage, not cross-slope or browser acceptance.

The original controller failed the corrected +15° idle fixture at tick 115. Following actual shin tilt instead of the desired leg pose only delayed collapse, failing the extended stand at tick 374. The successful pose generator keeps gravity-up/heading as its reference, adjusts leg reach for the slope and preserves bounded ankle targets. The initial downhill release failed at tick 277; bounded counteracting foot impulses corrected the release-braking case without increasing the absolute impulse ceiling.

The 06c2b probes confirmed a cross-slope running failure on +15° about world Z, yaw 0, using the same clear spawn and 120-tick settling period: head clearance fell below 1.1 m at total tick 256. This plane is tilted sideways relative to −Z travel. Reducing the extension reserve or adding bounded yaw feedforward also broke earlier terrain cases and was not retained.

Next: cross-slope travel with plane rotation about world Z while the character initially faces −Z; entry/exit between flat and tilted surfaces; different supported normals under the two feet; start/stop at other gait phases; native/WASM slope scenes; human movement assessment. These scenarios remain required and must retain meaningful posture/travel/contact gates. No result is yet claimed for them.

## Low steps

Spawn at the origin, settle 60 ticks, then walk toward −Z for 150 ticks. Three risers begin at Z = −1.0, −1.8 and −2.6 m, with heights 0.1, 0.2 and 0.3 m. Half-width is 3 m. Use adjacent tread boxes and extend the final landing to Z = −12; a short landing ending at −6 could eliminate a successful walker before the last assertion.

The initial overlapping-slab probe tripped by movement tick 78. A corrected adjacent-tread/long-landing probe on the 0.08 m-clearance flat fixture still failed: head height was 1.094 m at total tick 111 (movement tick 51). Its geometry no longer contains overlapping stair slabs, so tread sensing/clearance remains an implementation gap. Retain head >1.1 m, final pelvis Z <−3 and Y >0.85 m, and per-tick pelvis displacement <0.15 m alongside effort/finite-state gates. The flat gait's ideal symmetric 1 rad knee flexion raises a 0.62 m leg by only about 0.076 m, less than the riser. A bounded forward tread query and swing-clearance correction need implementation and verification.

## Repeated reversals: addressed within the documented fixture scope

On the ordinary flat fixture, settle 60 ticks. Alternate +X and −X input every 45 ticks for eight legs, separately walking and running. Require head >1.1 m throughout and final pelvis X velocity in each new direction >0.5 m/s, while maintaining the existing acceleration/effort caps and joint-error checks.

The original walk case failed on the fourth leg, at tick 43 of that leg. The later flat-plane fixture reproduced walking/running falls at total ticks 225/127. [Slice 06c2b1](../slices/06c2b1-reversals.md) adds tick-sampled reversal braking and preserves the eight-leg, 45-tick, posture and signed-speed requirements. Its evidence includes four world orientations, default spawn, analog updates, keyboard neutral gaps and native/WASM traces. This does not certify arbitrary curved turns or reversing on terrain.

## Varied stopping phases: addressed within the recorded scope

On the 30 m half-extent flat fixture with spawn elevation 0.08 m, settle 60 ticks, run +X for 45 ticks, then release to `(0,0,false)` for 180 ticks. The original controller falls even without a reversal. A related case reverses to −X for five ticks before releasing; it failed the head >1.1 m gate at total tick 223 (head 1.095 m). Keep both cases and the existing effort/finite-state bounds.

Diagnostics showed speed nearly reaching zero with both feet behind the pelvis, followed by forward tipping. [Slice 06c2b2](../slices/06c2b2-stopping.md) and [ADR 0013](../decisions/0013-stopping-force-pair.md) add a horizontal force pair with separately measured/capped moment. The original cases, 32 phase cases and 84 broader flat/forward-slope cases retain posture, speed, finite-state and force/effort gates. Sixty-four short deck cases additionally require motor and moment release. This evidence does not certify arbitrary terrain or human feel.

## Rotating-deck traversal: addressed within recorded native fixtures

The original `longer_rotating_deck_runs_keep_posture_when_released` regression was initially ignored and failing. [Slice 06c2b3](../slices/06c2b3-rotating-traversal.md) now enables it in the default suite with bounded support-moment compensation. The following describes the original failure: Spawn at [0,0,0] on a kinematic velocity-driven deck with half extents [6,.5,6], center [0,-.5,0], angular velocity [0,.4,0]. Settle 60, run each world cardinal for 60 ticks, then release 360. Require head >1.1 m, finite state, bounded command/moment, anchor error <.05 m and supported relaxation. Baseline `c4f2c7f` +X failed head clearance at release tick 11.

Independent comparison against `d9e6533` reproduced failure at release tick 10. Keeping run input held instead of releasing also failed after another 10/11/10/15 ticks for the four directions. This is an existing moving traversal deficiency that appears around the same motion phase, not caused by the new settling predicate. Shorter stops do not establish general moving traversal. The subsequent correction also retains 32 sustained four-second traversals, six-second post-traversal stops and 64 short-stop cases extended to twenty seconds. These finite fixtures do not establish arbitrary moving terrain or human acceptance.

```sh
cargo test --locked --release -p brawler-engine --lib longer_rotating_deck_runs -- --nocapture
```

Independent backward-slope probes also retain failures, including some during movement before release. They remain distinct from the verified forward uphill/downhill cases; do not describe these as general slope support.

## Browser and human gates

After native terrain fixtures pass, expose matching browser scenarios, compare native/WASM trajectories, inspect soles/targets/contacts visually, and obtain human movement feedback. Current screenshot inspection and flat-floor automation do not establish terrain or game-feel acceptance. All specification §33 release gates remain open.
