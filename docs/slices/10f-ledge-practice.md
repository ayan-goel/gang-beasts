# West walkway rescue practice

In the physics playground, select **Ledge rescue**, then **Start rescue**. Hold J and K to reach for the main roof as the walkway falls. Keep the hand buttons held and hold Space to climb. Releasing both hands lets Coral fall. **Retry rescue** starts another attempt from the same approach.

## What the player controls

The playground runs the ordinary two-fighter Skyline match in WebAssembly. It completes the countdown, walks Coral west at arena tick 6800, turns east at 7050 and stops at 7078. It pauses at tick 7150 with both hands free. The west panel drops at 7200. Preparation yields between batches of simulation ticks so the player can change scenes. Each attempt owns its simulation, and leaving the scene or retrying releases it.

The camera follows Coral. The exercise recognizes a hang from a hand attached to the main roof while the pelvis is outside and below the west edge. After a catch, the pelvis must stay inside and above 0.55 m for 60 ticks with the main roof grip released before the exercise reports success. A scored fall ends the attempt. A player can retry either outcome. Normal local and private matches still play the full approach with their ordinary cameras.

## Twenty timing combinations

All twenty native runs start in a fresh match with the same map, spawn, controls and approach. Both hands start reaching 50, 43, 36 or 29 ticks before the walkway drops. The player starts pulling 25, 40, 55, 70 or 85 ticks after the first main-roof catch. The catch must still hold a hanging fighter when the pull begins. At tick 7750 the fighter must be back inside at standing height with both hands free. A fresh walking input must then take Coral another 0.8 m inward within 90 ticks in the same round. [Native test](../../crates/engine/tests/skyline_rescue.rs)

| Reach before collapse | Successful pull delays, ticks after catch | Missed pull delays |
| --- | --- | --- |
| 50 ticks | 25, 40, 55, 70, 85 | — |
| 43 ticks | 25, 40, 55, 70, 85 | — |
| 36 ticks | 25, 40, 55, 85 | 70 |
| 29 ticks | 25, 40, 55, 70, 85 | — |

Originally all twenty caught the roof, but only fifteen could walk away. Several fighters rolled back over the edge during get-up. The controller now aims 0.78 m inward during the crossing phase and requires the pelvis to clear 0.38 m before handing control back to the grounded fighter. Both changes fit within the existing 0.82 m landing clearance check. The original values were 0.62 m and 0.22 m. A supported body part is still required for an assisted release. The grip, speed, stamina and force budgets use their existing configuration.

Nineteen attempts completed the full recovery. In the remaining case the pelvis was at x = −9.70 m, y = 0.75 m after release at tick 7530, then crossed outward to x = −10.40 m by tick 7560 and fell. This case stays in the grid. The result meets the proposed 18-of-20 threshold for this approach. Perceived control still requires human playtesting.

## Reproduction

- Run `cargo test --release --locked -p brawler-engine --test skyline_rescue -- --nocapture` to print the score and each missed timing.
- Run `cargo test --release --locked -p brawler-engine --lib mantle` for foot support, neighboring stop timings, single- and two-hand pulls, interrupted effort, obstructions and passenger load.
- From `web`, run `npm run test:e2e -- ledge-practice.spec.ts skyline-mantle.spec.ts online-ledge.spec.ts` for normal keyboard controls in the exercise, the local match and a two-client private match. [Practice checks](../../web/tests/ledge-practice.spec.ts)

Build verification used engine fingerprint `7a8f0ccc55e95ecb14e021c91b5ea37fbfe19a5af70fbba7bb0de34981c66d39` and the bundled Skyline map on an Apple M3 Pro. The release workspace suite passed 312 Rust tests. All 94 web unit tests, TypeScript, Clippy and the production build passed.

In Chromium, keyboard input recovered a hanging fighter in the practice scene and the local match. A fresh attempt with neither hand held fell, and releasing both hands during a hang also produced a fall. Changing to the cuboid scene during retry returned to one working canvas. The packaged private-match test ran the full scheduled collapse with 60 ms of application-message delay in each direction on the climber’s connection. Both clients saw the catch, the return to standing and another 0.8 m of inward walking with the round still playing and both scores at zero. [Two-client rescue](../../web/tests/online-ledge.spec.ts)
