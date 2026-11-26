# Compact rotations in full match snapshots

Skyline can use a 23-byte body transform. On the eight-fighter native workload, reducing the rotation field from 16 bytes to seven lowered per-player snapshot traffic from 130.72 to 95.95 KB/s. Body IDs and positions remain exact at all distances. The server continues to simulate the original physical rotations. Compact decoding affects only the presentation sent over the network.

## Design

The browser requests `compact=1` on each new or resumed room socket. The server prepares both encodings once per published tick and applies input acknowledgements only to each recipient's copy. A spectator receives the same world with its acknowledgement flag cleared. An older client can share the room and receive full-precision rotations. A new client can reconnect to a server that only provides the original records.

The [wire contract](../protocol.md#full-skyline-snapshots) defines component order, sign, rounding, offsets and validation. Keeping each frame self-contained lets the server discard an outdated publication and lets a client recover from a stream interruption on the next frame.

## Verification

The original golden packet remains byte-identical. A second fixture covers all four omitted-component indices, negative largest components, ties, signed retained components, nontrivial gameplay fields and an exactly represented wide epoch. Rust produces its bytes, and the browser decoder verifies poses against the original positions and source rotations. A deterministic 50,000-orientation test includes slightly non-unit input and checks angular error below 0.01 degrees. Decoder tests reject malformed extents, invalid indices, reserved signed values and impossible rotations.

On the [baseline M3 Pro and Skyline scene](../verification/eight-player-room.md), run the same release example with `--compact`. Each run measured 7,200 ticks and 3,600 snapshots per client. All five again produced 402 hits and 7,098 opponent-grip ticks with eight survivors. The measured bytes per player fell from 15,686,752 to 11,513,803 in 120 simulated seconds, a 26.6% reduction. Across eight players that is 767,587 B/s of application snapshot payload. At the initial 129-body count a packet occupies 3,203 bytes.

| Run | Total p50 (µs) | Total p95 (µs) | Total p99 (µs) | Worst (µs) | Publication p95 (µs) | Ticks over 16.667 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 866 | 957 | 1,013 | 2,106 | 4 | 0 |
| 2 | 880 | 961 | 1,003 | 1,137 | 4 | 0 |
| 3 | 878 | 954 | 994 | 1,122 | 4 | 0 |
| 4 | 880 | 965 | 1,010 | 1,925 | 4 | 0 |
| 5 | 881 | 957 | 1,002 | 1,104 | 4 | 0 |

The release server and browser use separate timing and delivery paths. The client frame-pacing matrix, scheduling under sustained load and transport overhead remain open.

## Browser delivery

Ten production-preview Chromium cases passed, including physical grabs, a stalled stream, spectator promotion, grace expiry, consecutive rematches and a mixed-format eight-player ring-out. The native capture reconstruction fixture imports the TypeScript decoder from Vite's development server. It passed there. Attempting that fixture on the static preview returned a missing source module.

In the eight-player case, seven current Chromium clients requested compact frames and one requested the original layout. All eight entered the match and observed the same elimination. The host renderer drew eight fighter skeletons. For roughly six seconds starting at the first Playing snapshot, the test counted incoming WebSocket application bytes. Four compact connections received all 180 expected snapshots, each taking 576,540 bytes (96.10–96.11 KB/s). The original-format connection received all 180 for 785,520 bytes (130.92 KB/s). The other three compact connections received 133, 165 and 172 snapshots over their windows. Their lower byte rates also reflect fewer delivered frames. Production worker scheduling, per-client publication gaps and eight renderers sharing a machine need separate timing evidence. These figures exclude WebSocket, TCP and TLS overhead.
