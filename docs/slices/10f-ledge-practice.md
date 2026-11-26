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
| 36 ticks | 25, 40, 55, 70, 85 | — |
| 29 ticks | 25, 40, 55, 70, 85 | — |

Originally all twenty caught the roof, but only fifteen could walk away. Several fighters rolled back over the edge during get-up. The controller now aims 0.78 m inward during the crossing phase and requires the pelvis to clear 0.38 m before handing control back to the grounded fighter. Both changes fit within the existing 0.82 m landing clearance check. The original values were 0.62 m and 0.22 m. A supported body part is still required for an assisted release. The grip, speed, stamina and force budgets use their existing configuration.

That change recovered nineteen of the twenty attempts. The remaining miss exposed a separate problem transferring the legs onto the roof.

## Headfirst landings

In the failing replay Coral began reaching at arena tick 7164, caught the main roof at 7228 and started pulling 70 ticks later. The head settled on the deck while the pelvis crossed the lip and the feet trailed outside. After release, standing up carried Coral backward. At tick 7530 the pelvis was 30 cm inside the roof. Half a second later it was 40 cm outside.

During the crossing phase, the controller checks whether the head is lower than the pelvis, over the inner side of the lip and within 2 cm of the roof at the bottom of its collider. In that pose, each shin uses its existing 20% share of the pull force to approach a point 78 cm inward and 66 cm above the lip. The vertical clearance lets the legs swing over the lip before the grounded get-up. The same 2.2 m/s target-speed limit, hand force caps, clearance tests and release conditions apply. [Mantle drive](../../crates/engine/src/character/mantle_drive.rs)

All twenty reach/pull combinations in native Rust and in the packaged WebAssembly module now catch, return to standing with hands free and walk at least 0.8 m toward the center. The regression requires every recorded combination to pass. In the formerly failing WebAssembly replay Coral released at tick 7464, stood at x = −8.705 m and y = 0.81 m at tick 7750, then walked to x = −5.716 m after 90 ticks. Both scores stayed at zero in round one. [WebAssembly results for all twenty timings](../verification/skyline-rescue-timings.jsonl)

The corrected build passed 316 release workspace tests, 110 web unit tests, Clippy, formatting, TypeScript and the production build on an Apple M3 Pro. Chromium covered keyboard rescue, voluntary release, a missed catch and the two-client private match with 120 ms of added round-trip delay. The production preview also passed the practice and lobby readiness checks. Engine fingerprint `cf4b0559e46bedaae7e6886a44953df7351b682214f339bfa1f5f29713f0afa2`. Human assessment of deliberate control remains open.

## Reproduction

- Run `cargo test --release --locked -p brawler-engine --test skyline_rescue -- --nocapture` to print the score and each missed timing.
- Run `cargo test --release --locked -p brawler-engine --lib mantle` for foot support, neighboring stop timings, single- and two-hand pulls, interrupted effort, obstructions and passenger load.
- From `web`, run `npm run test:e2e -- ledge-practice.spec.ts skyline-mantle.spec.ts online-ledge.spec.ts` for normal keyboard controls in the exercise, the local match and a two-client private match. [Practice checks](../../web/tests/ledge-practice.spec.ts)

The initial practice build used engine fingerprint `7a8f0ccc55e95ecb14e021c91b5ea37fbfe19a5af70fbba7bb0de34981c66d39` and the bundled Skyline map on an Apple M3 Pro. The release workspace suite passed 312 Rust tests. All 94 web unit tests, TypeScript, Clippy and the production build passed.

In Chromium, keyboard input recovered a hanging fighter in the practice scene and the local match. A fresh attempt with neither hand held fell, and releasing both hands during a hang also produced a fall. Changing to the cuboid scene during retry returned to one working canvas. The packaged private-match test ran the full scheduled collapse with 60 ms of application-message delay in each direction on the climber’s connection. Both clients saw the catch, the return to standing and another 0.8 m of inward walking with the round still playing and both scores at zero. [Two-client rescue](../../web/tests/online-ledge.spec.ts)
