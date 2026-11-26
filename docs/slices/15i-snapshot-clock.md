# Sizing the presentation buffer from snapshot delivery

A private match starts with an 83 ms presentation buffer. The client records each accepted snapshot's unwrapped server tick and local arrival time. Over the latest 32 records, the fastest observed delivery sets the reference clock. Using that shared reference keeps the entire scene moving at a steady rate when individual packets arrive early or late.

After 16 snapshots, the client estimates how much authoritative history it needs to draw between arrivals. For each consecutive pair it measures how old the earlier state was when the next one arrived, relative to the fastest delivery in the window. The largest observed gap plus half a simulation tick determines the target. When the route gets slower, the buffer also covers the difference while the clock catches up to the new reference. A regularly spaced 30 Hz stream needs about 42 ms in total. A skipped publication increases the target even if the remaining packets are perfectly punctual.

| Parameter | Limit | Purpose |
| --- | ---: | --- |
| Buffer | 2.5–6 ticks (42–100 ms) | Cover normal arrivals while capping added latency under sustained irregular delivery |
| Buffer adjustment | 0.08 ticks/publication | Change presentation speed gradually |
| Clock adjustment | 0.1 ticks/publication | Keep one slow packet from changing every limb's playback time |
| Extrapolation | 6 ticks (100 ms) | Bound motion while waiting for fresh state |

Every fighter, grip, prop and scoreboard uses the same presentation timeline. The server decides each result. Round changes and long gaps discard the old clock. [Implementation](../../web/src/online/SnapshotTimeline.ts) · [Timeline tests](../../web/src/online/SnapshotTimeline.test.ts)

## Measuring a standing jump

The [regional jump scenario](../../web/tests/jump-latency.spec.ts) timestamps the first client input, the arrival of the first snapshot that acknowledges the press and raises the host pelvis by at least 3 cm, and the first drawn rise of the same size. The server and drawn baselines are each recorded before takeoff. All three timestamps use the same browser clock. Time from keydown to the confirmed snapshot includes both transport directions, server scheduling and snapshot publication. The remaining interval includes interpolation and frame scheduling. A draw timestamp records CPU draw submission. Human response and physical display scanout require separate measurement.

To enforce the §29.2 standing-jump p95 locally, run `cd web && BRAWLER_CHECK_REGIONAL_JUMP=1 npm run test:e2e -- jump-latency.spec.ts` on an idle host. The opt-in assertion uses nearest rank over 20 independent room starts. The ordinary browser suite still requires all 20 presses to be sent, confirmed and drawn. The flag also builds and serves the production web bundle against the release room worker.

## Production results

Two Chromium 159.0.8154.6 clients ran on an Apple M3 Pro with macOS 26.6.2 and ANGLE Metal. The host used a 1280 × 720 viewport and a loopback relay configured for 60 ms RTT with up to 5 ms variation in each direction. The guest had a direct loopback connection. Each column covers 20 separate matches. The final build passed the 150 ms assertion in both runs. [All trials, observed RTTs, and bundle fingerprints](../verification/snapshot-clock-jumps.json)

| Browser measurement | Previous clock | Final run 1 | Final run 2 |
| --- | ---: | ---: | ---: |
| Median keydown to drawn rise | 147.4 ms | 113.6 ms | 115.4 ms |
| p95 keydown to drawn rise | 164.1 ms | 130.5 ms | 147.4 ms |
| p95 keydown to confirmed snapshot | 103.2 ms | 100.0 ms | 105.3 ms |
| p95 confirmed snapshot to drawn rise | 72.4 ms | 46.4 ms | 49.4 ms |

Each percentile is computed independently. The final two runs include the slower-route correction. The raw file also retains the two development runs that preceded it. Standing jumps meet this measured profile on the tested hardware. The overall multiplayer gate still requires broader action and network coverage and human playability.

The final production bundle also passed the attached-fighter punch, five-second stream stall and recovery, and eight-player scoring and rendering scenarios. The unit fixtures cover steady motion between uneven arrivals, missing publications, route changes, epoch resets and capped extrapolation.

## Recovering from queued publications

On an M3 Pro with two default headless Chromium clients, a stall during initial rendering allowed more than a second of server snapshots to queue. The original clock could correct its origin by only 0.1 physics tick per publication. On the guest, the server had reached timeline tick 422 while the oldest retained snapshot was already at 360. The client was still reporting buffering and drawing the oldest pose. Both clients received the same landing record, but this buffering state kept the guest's audio cursor inactive.

The clock now limits how far its origin can trail the fastest arrival in the current history to one nominal 30 Hz publication (33.3 ms). Small delivery changes retain the gradual correction. The adaptive interpolation delay and extrapolation ceiling still apply, and the shared presentation tick moves monotonically. A deterministic test drains 55 queued packets in about one millisecond after a long client stall, then checks live status, fighter position, round state and continued progress. The previous implementation failed at the live-status assertion. [Timeline regression](../../web/src/online/SnapshotTimeline.test.ts)

Both browsers then produced one landing cue for the same server-owned jump on the default Chromium renderer. A separate stall during the first WebGL draw can still suppress an early countdown beep when the interval between observations exceeds 500 ms.
