# Delivering snapshots when the room publishes

In the initial eight-browser measurement, three connections received 133, 165 and 172 snapshots in windows spanning 180–181 publications. Both compact and original snapshots came from the same room. The server advanced the room at 60 Hz and published at 30 Hz, while each socket independently polled the latest frame at 30 Hz. A socket could poll just before publication and see the same tick twice, then miss the intervening tick when its timer next ran.

## Design

The room worker replaces its latest immutable frame in a Tokio watch channel. Each socket subscribes to that room and wakes on publication. Initial subscriptions mark the retained frame as new, so late joins and reconnects can immediately receive state. Slow consumers see the newest frame available when they resume. A connection holds an independent reference while its asynchronous write is in progress. A separate 250 ms timer allows connection ownership, room shutdown and heartbeat checks to continue during a publication stall. The wire format, fixed simulation rate, recipient-specific acknowledgement and bounded socket writes remain as specified.

A native regression verifies immediate state on subscribe, independent consumption, progress to the next frame, and two readers sharing the same retained frame after one skips a publication. Server simulation never waits for a network write.

## Verification

Three consecutive packaged Chromium matches repeated the [original eight-client scenario](19b-compact-snapshots.md#browser-delivery) on the same M3 Pro. Seven connections requested compact snapshots and the eighth used the original format. Each window began at a Playing snapshot and ended 360 authoritative ticks later. The first snapshot defined the boundary. Each client received all 180 subsequent snapshots in each window. All eight players then observed the ring-out in each match.

| Match | Delivered / expected per client | Compact application KB/s | Original application KB/s |
| --- | ---: | ---: | ---: |
| 1 | 180 / 180 | 96.13–96.14 | 130.97 |
| 2 | 180 / 180 | 96.00 | 130.81 |
| 3 | 180 / 180 | 96.10–96.11 | 130.94 |

These are local WebSocket application payloads over measured wall time. A remote network, a stalled consumer or multiple busy rooms can still cause individual clients to skip superseded frames. Endurance, browser rendering deadlines and regional action latency require separate measurements.

Six additional packaged Chromium cases passed. They covered a silent upstream connection, a silent downstream connection, a stalled stream, reconnection before grace and spectator fallback after expiry, and round-boundary promotion through both supported lineup handshakes. The release workspace suite passed 306 tests. The three room-profile example tests run separately.
