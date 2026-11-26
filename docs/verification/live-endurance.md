# Live eight-player endurance

The endurance workload runs eight isolated Chromium sessions against one dedicated release server on loopback. Each player creates or joins the room, verifies the catalog fingerprints, readies, and receives their own assigned fighter. The driver uses the production snapshot decoder, match timeline, round acknowledgement and input encoder. It uses an ordinary browser WebSocket and omits rendering and audio. The hour starts after all eight seats have received an applied input acknowledgement.

## Reproduce

Install the pinned web dependencies and Playwright Chromium. Run the commands from the repository root on macOS or Linux with `ps` available. The runner builds its server, chooses a free loopback port and closes the browser and server when the run ends. Standard output contains JSON lines, beginning with the hardware, browser, map and engine identity.

~~~sh
npm --prefix web test -- --run benchmarks/endurance
node --test web/benchmarks/endurance/memory.test.mjs
node web/benchmarks/endurance/run.mjs --quick
node web/benchmarks/endurance/run.mjs
~~~

The quick run measures 90 wall-clock seconds. The full run measures 3,600. Both drive fighters toward the roof center, operate the left and right hands, lift opponents and vary jump and headbutt inputs from a repeatable seed. After six seconds most fighters run east while one stays. The intended survivor rotates by match. Every third match, starting with the second, keeps its first round fighting through the scheduled gust before the runners leave. The host requests a rematch through the ordinary results command. The eighth seat drops its socket during play and resumes with its original ticket. [Driver and input schedule](../../web/benchmarks/endurance/client.ts)

## Interpreting the capture

- One row per minute reports each player's cumulative validated snapshot, sent input and changed acknowledgement counts. The host's score count records a winning round once, and grip and damage counts are samples containing the condition. A gust sample means play reached the map's gust tick. Each session checks that a gripping owner and any target fighter are still in play. The production decoder checks transforms, fighter conditions, hand phases and the existence of grip targets.
- Both runs require scored rounds, fresh input acknowledgements, contact, successful reconnection and at least 95% of the nominal 30 Hz publications at each client. A twelve-second gap fails the run. The hour additionally requires rematches and late hazard coverage.
- RSS is the operating system's resident memory reading for the server PID alone. The raw rows retain allocation peaks. For this single-room workload, the spread between the median RSS values for minutes 31–40, 41–50 and 51–60 must stay within the greater of 1 MiB or 10% of the first median. The allowance was chosen before the hour run to accommodate allocator page retention and sampling variation. All 61 minute readings, including the start, are required. A bounded spread over this interval is evidence about this workload and duration. [Memory calculation and fixtures](../../web/benchmarks/endurance/memory.test.mjs)

The [native 100-reset run](reset-endurance.md) separately checks every simulated limb velocity and spawn reconstruction. The [browser rendering benchmark](eight-player-rendering.md) covers GPU presentation.

## Pilot result

The first 90-second trial on an Apple M3 Pro reached four rounds across two matches and awarded three rounds. All eight clients applied between 511 and 655 distinct input acknowledgements. The eighth seat resumed once. The host observed 839 snapshots with a grip and 1,549 with a damaged fighter. The greatest observed gap across all eight clients was 334 ms, including reconnection. A second 90-second run with the final coverage checks also passed across two matches, including 838 snapshots with a grip and a worst delivery gap of 315 ms. [Raw pilot](live-endurance-quick.jsonl)

The reference machine has 12 logical CPUs and 18 GiB RAM, runs macOS 26.6.2 arm64 on AC power, and uses Rust 1.88.0 with Cargo's release profile and the workspace lockfile. The server has one unpinned match worker, sharing the machine with the clients. The headless clients draw no 3D scene. The JSON capture identifies the exact Chromium build and engine and map fingerprints.
