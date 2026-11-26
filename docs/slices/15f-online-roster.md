# Private multiplayer rosters

The host can start once two through eight occupied seats have finished loading and marked themselves ready. Allocation happens outside the directory lock. Before the worker can become the room’s game, the server verifies that the sessions, connection generations, names, colors and seat order still match the ready roster.

The worker uses consecutive fighter indices in ascending lobby-seat order. Removing a player before a match can leave a gap in lobby seats. The assignment explicitly carries the original seat for each fighter, and the server binds input to the authenticated session. A spectator receives index 255. The established two-player assignment stays byte-compatible. The [wire format](../protocol.md#lobby-controls) defines the counted assignment for larger rosters.

Each browser builds one articulated rig per participant and uses one interpolation clock for every fighter and arena object. The camera frames all visible fighters. Eliminated fighters drop out of framing, and the scoreboard retains their color, name and score and marks them out.

## Verification

Eight isolated Chromium profiles created and joined one room. Start stayed disabled with seven ready. After the eighth readied, each profile saw its own fighter name, an eight-fighter canvas and the eighth player’s scoreboard entry. The eighth player ran off the roof using the keyboard. All eight sockets received the elimination, each scoreboard marked the fighter out and the round continued with survivors. This loopback test uses small viewports. Target-device frame time, high-latency play and extended match endurance remain open.

A separate production-preview run played a whole round using eight independent browser profiles. From the eight authored spawns, slots one through seven held Run and respectively S, D, A, D, W, D and W until each fell. Slot eight stayed on the roof. All eight scoreboards showed its point, entered round two, retained the point and cleared every elimination marker. The build passed the full 67-test Chromium suite and all 40 TypeScript tests.

Native and TypeScript fixtures cover sparse lobby seats, spectator assignment, invalid ownership, duplicate or unsorted identities, truncated or extended payloads, and rejection of a snapshot with the wrong roster before acknowledgement. The [worker fixtures](15e-worker-roster.md) verify slot-specific acknowledgements and every replicated limb ID.

At the starting scene’s 129 bodies, the full snapshot is 4,364 bytes (20-byte header, 129 × 32-byte poses, 24-byte game header and eight 24-byte fighter records). At the nominal 30 publications per second, sending every snapshot would use 130,920 application bytes per second per client. This calculated rate exceeds the 100 KB/s design target before transport overhead. The online roster passes its functional checks while bandwidth optimization remains open.

During the [camera framing](12e-skyline-framing.md) suite, seven of eight browsers observed the eighth fighter’s elimination before the test deadline. Three consecutive isolated runs and a follow-up run alongside the hazard and reconnect scenarios passed. The cause of the original miss remains unknown. On failure the roster test now saves `roster-delivery.json` with the snapshot and socket observations and visible screen for each seat before browser teardown.
