# Private-match round-trip delay

A private player sees “High network delay. Controls may feel delayed.” when their own connection has sustained high round-trip time. A peer with a healthy connection continues playing without the warning. The server keeps simulating through the delay. If snapshots stop altogether, the existing stall and reconnect paths take precedence.

## Measurement and thresholds

On a mutually supported match socket, the browser sends one sequenced probe per second of active snapshot delivery. The socket task echoes it directly. The browser uses its monotonic clock from send to matching reply. This includes browser scheduling and queued transport time. Simulation, interpolation and the display pipeline add their own time between a player's action and visible feedback.

The median of the last three samples must reach 180 ms to show the warning and fall to 140 ms or lower to clear it. These initial thresholds distinguish the 100 ms and 200 ms test profiles and suppress flicker from a single slow frame. The three-second probe cutoff discards obsolete samples after a browser suspension. Timing replies cannot satisfy the snapshot watchdog or renew an idle server session. Reconnecting discards the old measurements. [Transport records](../protocol.md#match-latency-probes)

## Verification

The [Chromium scenario](../../web/tests/match-latency.spec.ts) runs two real browser contexts against the native worker. A bounded, byte-order-preserving TCP relay delays the host's connection in each direction. The clock and steady snapshots continue at all four RTT settings. A return to the 20 ms setting clears the warning on the original socket. An unnegotiated connection exercises the original record set.

Run from the repository root with `cd web && npm run test:e2e -- match-latency.spec.ts`. The test attaches each profile's raw browser round-trip samples to the Playwright report. Each switch discards one mixed-profile probe and retains the next five. The regional test varies each direction by up to 5 ms, for up to 10 ms of added or subtracted round-trip delay. Packet loss, action-to-visible timing, and human playability have separate acceptance checks.

The production build on an Apple M3 Pro running macOS 26.6.2 and Chromium 159.0.8154.6 (ANGLE Metal) produced the measurements below. [Raw samples](../verification/match-latency.json)

| Configured RTT | Measured median RTT | Host warning |
| --- | ---: | --- |
| 20 ms | 23.4 ms | Clear |
| 60 ms | 66.0 ms | Clear |
| 100 ms | 103.6 ms | Clear |
| 200 ms | 203.6 ms | Displayed |
| 20 ms on recovery | 23.5 ms | Clear |
