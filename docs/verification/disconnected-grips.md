# Disconnects during a Skyline grip

The server releases a disconnected fighter's own grip immediately after the room observes detachment. A connected opponent may keep holding the disconnected fighter during the 20-second grace period. Forfeiting the target releases that incoming grip and eliminates the target. An unresponsive connection also loses held input after 15 simulation ticks. [Room input lifecycle](../../crates/server/src/room.rs) · [Physical release](../../crates/engine/src/character/duel_grips.rs) · [Forfeit cleanup](../../crates/engine/src/character/duel_match.rs)

## Native verification

Three fighters enter an ordinary Skyline room. The first player walks toward the second with a held left hand until actual contact attaches it to a limb. The third player keeps the round alive after a forfeit, so a round reset cannot mask a lingering grip. [Release regressions](../../crates/server/tests/disconnected_grips.rs)

| Interruption | Observed result |
| --- | --- |
| Grabbing player goes silent | The existing grip holds through 15 solves following the last input. The first stale solve releases it and returns the hand to idle. The opponent stays in play. |
| Grabbing player disconnects with another held-input packet queued | Detach clears the grip. After reattachment and 60 neutral ticks the hand is still idle. The opponent then walks for 120 ticks and the previously held limb ends more than two horizontal metres from the palm. |
| Held opponent disconnects and forfeits | The connected player still holds the opponent one solve after detach. The forfeit clears the incoming grip. It stays clear for 60 ticks while the holder keeps sending grip input. The two remaining players stay in the same round with zero points. |

## Browser verification

Two independent Chromium profiles play a private match. Both see Mint's left hand attached to Blue. The test cuts Mint's socket with the game canvas focused and the grab key still down. The peer sees the grip release. Mint resumes fighter slot zero with neutral movement and buttons on the first input. Both views remain in round one, and the peer receives at least another 60 authoritative ticks after release without the grip returning. This case passed three consecutive times. [Browser scenario](../../web/tests/grip-reconnect.spec.ts)

The related browser suite covers a held opponent reloading while the owner's grip survives, a failed first retry, separate uplink and downlink outages, grace-period expiry, spectator re-entry, and transfer of the host role. The directory test separately checks the deadline boundary and confirms that the remaining member becomes host. [Grace-period browser scenario](../../web/tests/support/disconnectScenario.ts) · [Host expiry boundary](../../crates/server/src/admission/grace_tests.rs) · [One-way outages](../../web/tests/match-partition.spec.ts)

All 41 native server tests and all eight targeted browser scenarios passed. Tests used the release server and production web bundle on an Apple M3 Pro, macOS 26.6.2, Chromium 159.0.8154.6, ANGLE Metal, and loopback. Engine fingerprint `58f614835f343e435536a7dbf678ac3c51b4ea387d9057ab3eccff3926339f4b`. Skyline SHA-256 `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`.
