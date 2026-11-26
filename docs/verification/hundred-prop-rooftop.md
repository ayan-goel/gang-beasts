# Eight fighters and 100 additional crates

This stress fixture runs a match on the authored Skyline rooftop with 100 additional dynamic crates. Each extra crate uses the same 10 kg cuboid, friction, collision setup, grip surfaces and retirement path as the first authored crate. There are 229 rigid bodies at the start of play, including 120 fighter limbs, nine authored arena objects and the extra crates.

## Reproduce

~~~sh
cargo test --locked --release -p brawler-engine --lib eight_fighters_share_one_solver -- --nocapture
BRAWLER_PROFILE_CRATES=1 cargo test --locked --release -p brawler-engine --lib eight_fighters_share_one_solver -- --nocapture
~~~

The default regression exercises the first 360 playing ticks, then retirement and reset. The longer command runs five fresh matches. Each gets a 30-second warmup after the normal countdown and a 120-second measurement at 60 Hz. The release test executable can also run under `/usr/bin/time -l` on macOS. The per-run JSON uses microseconds and reports exact deadline misses before rounding. [Fixture](../../crates/engine/src/character/prop_stress/mod.rs) · [Measurements](../../crates/engine/src/character/prop_stress/measurements.rs)

## Setup and checks

The extra crates start clear of the fighters in a 5 × 5 rack, four crates high, above the center of the roof. Gravity drops them into play. Players steer toward the pile and operate both hands and lift at fixed intervals. A scripted impulse disturbs one crate per tick in rotation, adding up to 1 m/s toward the center and 1.5 m/s upward. The same body is disturbed again after 100 ticks. All interactions after each impulse use ordinary game physics.

Each tick checks finite transforms and velocities for the 100 crates and 120 limbs, continued play with eight surviving fighters, and enabled dynamic bodies. The measured window counts real contact pairs, fighter contacts with the extra crates, attachments to their IDs, solver joints and awake crates. The default run requires average awake occupancy of at least 75 crates and at least 50 simultaneous crate-to-crate contact pairs so losing the pile would fail the regression.

After the measured window the fixture kicks the outermost crate across the edge. It must fall below the retirement plane, become invisible, cease to offer a grip surface and have no remaining hand attachment. Reset must restore the nine authored arena objects, 129 bodies and 112 anatomical joints. The next countdown must enter play with all original props visible. The extra-crate constructor exists only in engine tests.

## Reference measurement

The reference host was an Apple M3 Pro with 12 logical CPUs, 18 GiB RAM and macOS 26.6.2 arm64 on AC power. Rust 1.88.0, Cargo release, locked Rapier 0.26.1, eight solver iterations. The worker was unpinned and shared the host with the independent eight-client endurance run. The one-minute load average was 3.66 before the initial series. Thermal measurements were unavailable. The production Skyline map SHA-256 was `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`; the engine fingerprint was `e484e2afe166c75f98e8c203bf495da5120661d1da6f55d0886360b710a3ef79`.

The timer surrounds a single full game step including controllers, contacts, Rapier, grip handling and match logic. Scripted input preparation, crate disturbances, assertions and reporting run outside that interval. The fixture produces zero network packets. The [eight-player room profile](eight-player-room.md) covers production snapshot encoding and traffic separately.

| Run | p50 | p95 | p99 | Worst | Steps over 16.67 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 1934 | 2155 | 2320 | 2741 | 0 |
| 2 | 1959 | 2152 | 2311 | 2623 | 0 |
| 3 | 1983 | 2165 | 2309 | 2670 | 0 |
| 4 | 1987 | 2193 | 2354 | 4936 | 0 |
| 5 | 1973 | 2162 | 2301 | 2901 | 0 |

The median run-level p95 was **2.162 ms**. All 36,000 measured solves finished within one 60 Hz tick. Every measured tick had 100 awake extra crates, a fighter touching at least one extra crate and at least one active hand grip on an extra crate. The peak was 502 active contact pairs, including up to 405 extra crate-to-crate pairs, and up to 116 joints. The fastest limb or extra crate across warmup and measurement reached 10.55 m/s. Retirement and reset passed in all five runs.

Across the five-run process macOS reported 8,372,224 bytes maximum RSS and 5,964,088 bytes peak physical footprint. Per-allocation traffic was not instrumented. The previous independent five-run series also passed, reaching a worst solve of 5.646 ms. These results cover native game stepping on this host. Dense multiplayer rendering, scheduler delays and other stress scenarios have separate acceptance requirements.

All 321 native workspace tests passed, along with formatting, warning-denied Clippy and the production web build. Chromium loaded the matching WASM and server, verified a two-browser rooftop round and grabbed, lifted and released an authored crate in local play.
