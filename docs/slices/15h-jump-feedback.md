# Jump feedback under regional network delay

A focused player can send a jump press immediately after it enters the input sampler. The previous input path waited for a snapshot to trigger transmission. The new path allows one expedited press per accepted snapshot and expires that permission after 100 ms. Later edges remain ordered in the existing input sampler and the next eligible snapshot flushes them. A lost focus or round change expires the permission. The server still decides whether the fighter can take off or mantle. [Control path](../../web/src/online/RemoteControls.ts) · [Edge, stale-stream and epoch checks](../../web/src/online/RemoteControls.test.ts)

## How the measurement works

The test starts a fresh two-player private match for each observation, gives the host's match socket 60 ms of configured round-trip delay with up to 5 ms variation per direction, and leaves the guest on loopback. The host presses Space once from a settled spawn. The test captures the browser timestamp of the trusted key event, the first outgoing input record carrying the jump edge, and completion of the first local fighter mesh draw with its pelvis at least 3 cm above the last pre-input pose. The threshold separates an upward response from ordinary idle motion. A fresh spawn prevents a prior landing, loss of balance or hazard from changing the starting conditions. Unanswered inputs are retained and fail the test. [Browser scenario](../../web/tests/jump-latency.spec.ts) · [Draw observer](../../web/tests/support/jumpDraw.ts)

This measures browser input through draw submission. Physical keyboard scanning, display scanout and perceived responsiveness require a separate setup. The result applies to a standing jump. Running, grabs and other attacks need their own observations. Reproduce with `cd web && npm run test:e2e -- jump-latency.spec.ts`. The Playwright attachment includes every jump and RTT observation.

## Production results

On macOS 26.6.2, two Chromium 159.0.8154.6 clients used ANGLE Metal on an Apple M3 Pro. Each build completed 20 of 20 independent first jumps. Percentiles use nearest rank. One observation per independent match avoids weighting a particularly steady room more heavily. [All 40 trials, raw RTTs and bundle identifiers](../verification/regional-jump-feedback.json)

| Browser measurement | Previous median | Expedited median | Previous p95 | Expedited p95 |
| --- | ---: | ---: | ---: | ---: |
| Keydown to sending jump | 18.3 ms | 0.4 ms | 36.7 ms | 0.8 ms |
| Keydown to first drawn rise | 163.3 ms | 146.2 ms | 196.6 ms | 162.8 ms |
| Match socket round trip | 66.8 ms | 67.0 ms | 76.6 ms | 76.9 ms |

The measured jump p95 remains 12.8 ms above the §29.2 target of 150 ms. The socket send now accounts for less than 1 ms at p95. Work on the remaining delay should measure server scheduling, snapshot publication and presentation before changing the interpolation policy. The [follow-up presentation-clock measurements](15i-snapshot-clock.md) separate authoritative confirmation from drawing. Human acceptance remains open.

In a separate two-player regression, both clients saw the host catch the roof after the west walkway collapsed and use Space to pull back onto the deck. The private reconnect and four graphics-recovery scenarios also passed with the expedited input path.
