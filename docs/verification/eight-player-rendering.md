# Eight-fighter Skyline browser rendering

The M3 Pro rendered 108,000 frames in 30 measured minutes across the three presets. Median run-level p95 animation callback time was 1.7–1.8 ms. A separate wall-clock sample measured 18.0–18.5 ms p95 between submissions, above the 16.667 ms target. The results support keeping the current renderer and presets while frame delivery is investigated on additional browsers and devices.

## Workload

The native release room drives eight fighters toward the center of Skyline. At 10 Hz each driver steers from its current pelvis position and cycles through holding, lifting and tapping. Rapier advances at 60 Hz. Every second tick the recorder saves fighter one's personalized compact snapshot. The reference capture has 4,530 records over 151 simulated seconds, 522 confirmed hits, and eight surviving fighters at every publication. The capture SHA-256 is `51eba97a134a350c80431744c7508b54d66778a54e329df1bd2c79909f130003`. [Capture source](../../crates/server/examples/render_trace/main.rs) · [Shared combat inputs](../../crates/server/examples/support/combat.rs)

One Chromium page loads the production private-room bundle, WASM map and Roof Crew GLB. The fixture supplies the initial session, lobby and assignment and replays the capture over Playwright's routed WebSocket at 30 Hz. The shipping code handles snapshot decoding, interpolation, local input sampling, group camera, skinning, shadows, score and stamina UI. Only the observing client renders. Each run checks the shadow target, eight visible skeletons, eight scoreboard entries, page visibility, measurement duration and an outgoing input referring to the final 100 ms of the capture. [Room replay](../../web/benchmarks/replayRoom.mjs) · [Measurement driver](../../web/benchmarks/renderProfile.mjs)

## Timing

The browser's animation timeline supplies the interval between consecutive match draws. The callback timer covers pose interpolation, camera updates, Three.js submission and synchronous UI notification. A nested timer covers the Three.js render call alone. Both report elapsed main-thread time and exclude asynchronous GPU completion. The draw-call and triangle counts come from Three.js renderer info for the main pass. Cold start occurs during warmup. The runner calculates nearest-rank p50, p95, p99 and maximum within each run. It also counts intervals above 25 ms, which distinguishes a missed 60 Hz refresh from timestamp rounding near 16.67 ms. [Browser observer](../../web/benchmarks/renderObserver.mjs) · [Statistics](../../web/benchmarks/frameSamples.mjs)

Each preset runs in a fresh browser context with a 1440 × 900 CSS viewport and device pixel ratio 2. The standard matrix rotates the order of Low, Medium and High over five repetitions. Each run replays 30 seconds of warmup and measures 120 seconds, covering the west walkway failure. Quick mode warms for five seconds and measures 15 seconds starting at the same simulated time as the full matrix.

## Reproduction

From the repository root, using the pinned Node and Rust toolchains, installed web dependencies and Playwright Chromium:

~~~sh
node --test web/benchmarks/frameSamples.test.mjs
npm --prefix web run benchmark:render -- --quick
npm --prefix web run benchmark:render
~~~

The npm command builds the release web client before launching a loopback Vite preview. The driver prints one JSON record per completed run. To save a JSON-lines report after building, run `node web/benchmarks/renderProfile.mjs > report.jsonl` from the root. The native trace is generated afresh in memory. The profile uses the Metal ANGLE backend on macOS.

## Reference device and build

Apple M3 Pro with 12 CPU cores (6 performance, 6 efficiency), 18 GPU cores and 18 GB unified memory. macOS 26.6.2 build 25G83 on AC power. Chromium headless shell 159.0.8154.6 used ANGLE Metal. The browser and replay sender used normal desktop process scheduling without core pinning or thermal controls. Thermal telemetry and display scanout were outside this measurement.

The production bundle came from gameplay revision `d1cde96` with Three.js 0.180.0, React 19.3.1, Vite 6.4.6 and Node 20.18.0. Rust 1.88.0 built the native release capture with locked Rapier 0.26.1. Engine fingerprint `58f614835f343e435536a7dbf678ac3c51b4ea387d9057ab3eccff3926339f4b`, map SHA-256 `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`. The replay driver checks the browser's advertised content identity against the capture header before sending gameplay frames.

## Results

The table gives the median of the five run-level p95 values. The range shows the smallest and largest p95. The last two columns include every measured interval across all five runs.

| Preset | Buffer | Timeline interval p95, ms (range) | Callback p95, ms (range) | Three.js render p95, ms | Longest timeline interval, ms | Intervals >25 ms / total |
| --- | --- | --- | --- | ---: | ---: | ---: |
| Low | 1080 × 438 | 16.7 (16.7–16.8) | 1.7 (1.6–1.8) | 1.4 | 16.8 | 0 / 35,995 |
| Medium | 2160 × 877 | 16.7 (16.7–16.8) | 1.8 (1.7–1.8) | 1.4 | 16.8 | 0 / 35,995 |
| High | 2880 × 1170 | 16.7 (16.7–16.8) | 1.8 (1.8–1.9) | 1.4 | 16.8 | 0 / 35,995 |

A separate 15-second pass also measured the wall-clock gap between successive Three.js submissions. It used the same capture bytes and checked that the release WASM engine and map matched the native capture header before entering each match.

| Preset | Submission interval p50, ms | p95, ms | p99, ms | Worst, ms |
| --- | ---: | ---: | ---: | ---: |
| Low | 16.7 | 18.5 | 18.7 | 19.2 |
| Medium | 16.7 | 18.0 | 18.6 | 19.0 |
| High | 16.7 | 18.4 | 18.8 | 19.9 |

Chromium exposed timeline timestamps in tenths of a millisecond. The 16.667 ms target falls between two distinguishable readings at this cadence. Use the frame counts, tail intervals and callback costs alongside the strict threshold. GPU completion, physical display presentation, Firefox, Safari, other hardware and regional network play need separate measurements. [Full per-run results](eight-player-rendering.jsonl) · [Wall-clock submission pass](eight-player-rendering-quick.jsonl)
