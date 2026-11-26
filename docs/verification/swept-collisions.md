# High-speed hand and crate collisions

An attached hand moving at 7.5 m/s travels 12.5 cm per 60 Hz tick. The original simulation let its center pass a solid barrier 2 cm thick in the first tick. At 15 m/s it traveled the entire 25 cm step through the barrier. A 7 kg Skyline crate launched at 90 m/s could likewise move 1.5 m through a wall. The crate speed is a stress input.

## Solver decision

The shared engine registers the two hand spheres and each loose crate. Once a body's linear travel in the coming tick reaches its smallest collider half-extent, it tests its path against potentially intersecting solid colliders. The cast uses relative linear motion and the target collider's current pose. When the projected center would pass the target's center plane along the contact normal, Rapier receives a soft CCD prediction distance for that step. The registration assumes one centered collider on each tracked body. [Implementation](../../crates/engine/src/physics/swept_contacts.rs)

Enabling predictive contacts on every hand movement changed balance, steering and stair behavior in existing tests. Enabling hard CCD with two substeps also increased joint error in the impact fixture. Selecting contacts by the crossing test preserves ordinary near-side contact handling and the existing one-step, eight-iteration world. Sensors, disabled bodies and joint-suppressed contact pairs are excluded from selection.

## Physical regression

The tests give every limb of an airborne 15-body fighter the same initial velocity, leaving all fourteen joints in place. Each hand approaches the barrier from both directions at 7.5, 15 and 20 m/s. The fixture follows the hand for eight ticks, verifies its center stays on the approach side at impact and checks any later crossing against the finite face. Horizontal momentum summed across the rig measures the external collision impulse. Across the twelve approaches the peak momentum loss ranged from 7.91 to 25.09 N·s and the largest anatomical anchor separation was 9.61 mm. A sensor at the same location and a solid displaced 40 cm sideways allowed full first-tick travel with zero measured horizontal impulse. [Hand tests](../../crates/engine/src/character/fast_hand_tests.rs)

The crate fixture uses the actual Skyline dynamic prop and approaches from both directions. Its center stayed on the approach side for all twelve observed ticks. These fixtures cover linear approaches to thin stationary barriers. Rotation-only sweeps, other limbs and falling walkways are outside the scope of this change. [Crate test](../../crates/engine/src/arena/fast_crate_tests.rs)

Run the regressions with `cargo test --locked --release -p brawler-engine --lib fast_`.

## Release verification

The release workspace passed all 324 native tests. Formatting, warning-denied Clippy and the production web build passed. In Chromium the packaged game started a private rooftop match for two players, grabbed and released a real crate during a lift, and reproduced selected native stair-trace frames. The thin-barrier impacts themselves were exercised by the native engine tests. The revised engine fingerprint is `dc30b858e7d1bb693322b0eca058dfe0eeb99b6958dd77045b2fbe7bc9069df5`.

On the Apple M3 Pro, the same eight-fighter room profile ran five times per build with compact snapshots. Each run included 1,800 warmup ticks and 7,200 measured ticks. The server applies gameplay inputs and encodes snapshots for eight seats in the timed interval. The independent live endurance room continued during both series. [Profile source](../../crates/server/examples/room_profile/main.rs)

| Room build | p95 per run (µs) | Median run-level p95 | Worst tick | Ticks over budget |
| --- | --- | ---: | ---: | ---: |
| Before the collision change | 1,037 / 1,046 / 1,062 / 1,059 / 1,047 | 1.047 ms | 2.171 ms | 0 / 36,000 |
| With selective prediction | 1,045 / 1,030 / 1,038 / 1,041 / 1,053 | 1.041 ms | 2.062 ms | 0 / 36,000 |

All eight fighters survived throughout each measured run, with active hand grips and hits on both builds. Different hit counts are expected when collision response changes. The similar timings describe this workload on this host.

With [100 extra awake crates](hundred-prop-rooftop.md), the revised engine recorded a 2.264 ms median run-level p95 across five runs, compared with the earlier 2.162 ms reference (+4.7%). All 36,000 measured steps met the 60 Hz deadline. The slowest took 5.187 ms. All five runs passed the crate retirement and round reset checks.
