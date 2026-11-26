# Disconnect grace and forfeits

The server reserves a seat for 20 seconds after it observes the owning WebSocket close. A superseded socket cannot start its replacement’s timer. Controls are neutralized and outgoing grips are released when the worker observes the lost connection. The body stays vulnerable during the grace period. An authenticated return before the deadline keeps the seat and fighter. The new socket repeats content verification, receives a full snapshot and acknowledges the current epoch before it can control the fighter.

The deadline uses the server’s monotonic clock. A reconnect checks expiry under the directory lock even between maintenance sweeps. One service task checks deadlines every 100 ms while the router is live. Expired and explicitly vacated seats become available, and host authority passes to the lowest connected seat, falling back to an occupied seat if everyone is offline. An explicit leave releases the seat immediately.

## Round outcome and cleanup

All seats expiring in one directory sweep publish one forfeit mask to the simulation. At the next solve, the worker clears queued input for those slots. The engine eliminates their bodies and releases incoming and outgoing grips. Physical falls and forfeits settle together. A forfeited fighter stays eliminated across round reconstruction. A player who rejoins the room after expiry spectates the ongoing match.

When fewer than two eligible competitors remain, the server shows the round result for the usual five seconds and returns the room to the lobby. Scores cannot accrue from uncontested resets. Remaining members keep their names and seats and ready up again. An empty room releases its worker and its code expires 60 seconds after the final seat is released. Cleanup runs without additional HTTP requests.

## Verification

A clock-controlled test exercises a stale close, reconnect one nanosecond before expiry, and rejection at the exact deadline. Native room tests exercise simultaneous forfeits, a draw, no duplicate points, and a real ring-out followed by a round reset with an absent fighter. A timer-driven service test checks that the abandoned worker and room are reclaimed without HTTP traffic.

In Chromium, a three-player match restored the original host’s live fighter after a quick reconnect. A subsequent sustained disconnect forfeited that fighter. The two peers saw the elimination, and the former host rejoined as a spectator. An explicit leave awarded the survivor one point. The survivor and spectator returned to the same lobby and the host controls transferred to the survivor. This scenario tests a server-observed transport close; a silent network partition also depends on transport failure detection.

A production-preview check started a two-player match and closed both browser profiles without leaving the room. It polled only the readiness endpoint, which bypasses the room directory. The worker became available after 20.24 seconds and room metadata then reported zero seats. The full Chromium suite passed all 70 tests, including recovery links for an expired seat and a retired room.

The reconnect scenario now checks the three server assignments in order: fighter zero at initial entry, fighter zero after a quick reconnect, and spectator after expiry. It waits for assignment before checking the visible spectator screen. The earlier assertion could time out while the arena was still loading. This scenario passed alongside the rooftop warning and eight-player elimination scenarios after the assertion changed.
