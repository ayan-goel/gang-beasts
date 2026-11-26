# Regional headbutt responsiveness

A focused private-room player pressing Q can use the same immediate input send as a jump or hand press. The client allows one expedited send per fresh server publication, valid for 100 ms. A second action pressed in that interval keeps its ordered input edge. Changing rounds clears pending actions. The server decides whether the headbutt starts, spends stamina and applies the neck and chest motor targets. [Client controls](../../web/src/online/RemoteControls.ts) · [Input regression](../../web/src/online/RemoteControls.test.ts) · [Physical headbutt](08c1-headbutts.md)

## Measurement

The host and guest join a fresh private match for each trial. The host plays through an ordered TCP relay with 60 ms configured round-trip delay and up to 5 ms variation per direction. The guest uses direct loopback. The host has full stamina, takes no other action, and waits for the head orientation relative to the chest to stay within one degree of its resting reference for 250 ms.

The observer starts at the trusted Q keydown and timestamps three events on the same browser clock. It records the first outgoing input carrying the headbutt press, the first snapshot that acknowledges its sequence and shows stamina spent, and the first fighter draw in which the head has rotated at least five degrees relative to the chest from its pre-input orientation. The renderer drives bone matrices directly, so the observer reads the matrix used for skinning after draw submission. Measuring head rotation relative to the chest excludes camera and whole-body movement. [Draw observer](../../web/tests/support/headbuttDraw.ts) · [Private-room scenario](../../web/tests/headbutt-latency.spec.ts)

Each release series starts 20 independent rooms and uses nearest-rank p95. Run `cd web && BRAWLER_CHECK_REGIONAL_HEADBUTT=1 npm run test:e2e -- headbutt-latency.spec.ts`. The ordinary browser test verifies one complete press, server acceptance and drawn response. The measurement covers standing windup. Hit confirmation, display scanout and perceived responsiveness require separate evidence.

## Production results

| Keydown to observed event, p95 | Previous build | Expedited run 1 | Expedited run 2 |
| --- | ---: | ---: | ---: |
| Headbutt press sent | 41.4 ms | 0.6 ms | 0.5 ms |
| Server acceptance received | 134.1 ms | 93.6 ms | 100.7 ms |
| Five-degree head rotation drawn | 182.7 ms | 148.1 ms | 149.0 ms |

All 60 trials recorded the input, the native stamina change and the drawn rotation in order. Median measured RTT was 65.7, 64.0 and 66.2 ms respectively. Both updated series met the 150 ms p95 target on this host. The slowest updated trial took 150.3 ms. The three series ran serially in Chromium 159.0.8154.6 with ANGLE Metal at 1280 × 720 on an Apple M3 Pro running macOS 26.6.2. The separate eight-player endurance room was active. The evidence includes every trial, RTT sample and a SHA-256 fingerprint covering all release files. [Raw measurements](../verification/regional-headbutt-feedback.jsonl)

The input regression failed on the previous implementation and passed with the shared immediate send. Four native combat tests and the room ownership test passed. They cover hit and miss, obstruction, simultaneous hand input, stamina, cooldown and countdown. The web suite passed 124 unit tests, TypeScript checks and the production build.

Nine packaged Chromium regressions passed across headbutts, match menus and grappling, including a free-hand strike from each side under regional delay. Both shared-keyboard and private headbutt checks also passed in Firefox and Playwright WebKit over HTTPS. Five additional checks passed on the updated local preview, including a private headbutt through the relay.
