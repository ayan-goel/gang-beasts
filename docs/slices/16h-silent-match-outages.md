# Silent match outages

A lost network path can leave a browser and its server socket open. During a running match, the client starts an eight-second timer at fighter or spectator assignment. Each full snapshot accepted by the interpolation timeline extends the deadline. An old or duplicate tick cannot keep a frozen match alive. On expiry the browser detaches the old handlers and clears the arena and inputs immediately, then starts the [existing seat recovery](16g-automatic-reconnect.md). A blocked close handshake can finish independently.

The server sends a standard WebSocket Ping two seconds after match attachment, and two seconds after each matching Pong. The four-byte little-endian sequence identifies the sole outstanding probe. A probe unanswered for six seconds ends that socket's match binding and starts the ordinary disconnect grace period. Each probe uses the same bounded write as the snapshot stream. Browsers answer at the WebSocket layer, including when an unfocused player or spectator sends no game input. A probe reply does not count as user activity for the session idle limit.

These thresholds allow a five-second transport stall to clear and leave time for reconnect before forfeiture. Short stalls still show the existing waiting message and use buffered snapshots when delivery resumes. The physics worker keeps its existing 250 ms stale-input rule throughout. A single player's outage has no effect on other players' clocks or room ownership.

## Verification

A loopback TCP relay between Chromium and the service drops bytes and shutdown signals in one direction of the original match socket. Replacement sockets use the healthy path. Two independent browser profiles start a match before the interruption. The downlink case leaves the original browser unable to receive either snapshots or an orderly close. The uplink case leaves it receiving snapshots while its replies cannot reach the server. Both cases require the original fighter assignment, the same round acknowledgement, automatic resume commands and a peer still in the playing phase.

The original implementation reached the downlink test's 18-second deadline with only the initial acknowledgement. Clock-controlled tests cover renewal, teardown, late and mismatched probe replies, and the exact expiry boundary. Run the browser cases with `npm --prefix web run test:e2e -- match-partition.spec.ts`.

Both one-way outages passed against the production browser build and native server. A separate production test held ordered snapshots for five seconds, then delivered them on the same connection. The waiting message cleared and both players stayed in round one. This run used Chromium on loopback. Real Wi-Fi, background suspension on mobile and the latency matrix are separate acceptance checks.
