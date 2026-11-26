# Changing the lineup between Skyline rounds

A fighter index identifies a body in the current round. A player’s points and connection can outlive that body. When a new lineup starts, the room supplies one entry for each spawn in order. An entry identifies the returning fighter’s previous index or identifies a new entrant with zero points and a detached input channel.

The final results tick is the single opportunity to change the lineup. The engine checks for two through eight entrants and verifies that each returning fighter exists exactly once. The room rejects forfeited returnees. A new entrant can use their former spawn. The old round owns its result and its winner until the transition succeeds. A match-winning result closes the match to further rounds.

The engine constructs the next world with its fresh bodies, grips, health, stamina and arena before replacing the current state. The room consumes one clock tick, advances the round epoch, remaps connection flags and clears every input queue and acknowledgement. The countdown then runs for 180 ticks. Ordinary local and online round resets use the same engine reconstruction.

## Verification

- A two-player room tests an early rotation, duplicate and nonexistent previous fighters, a forfeited returnee and invalid room sizes. Each failure at the boundary preserves the captured result byte-for-byte. Two entrants then replace a departure, the winner moves to a different compact index with one point, and the disconnected winner and newcomers require attachment. A prior-round packet fails and a fresh packet reaches only its new owner.
- Another fixture grows the room to eight fighters with 120 limbs and nine arena objects, then shrinks to two. The same player carries two points through the second change and earns the third normally. The match-winning result rejects a further lineup.

[Room regressions](../../crates/server/tests/round_roster.rs) · [Engine round transition](../../crates/engine/src/character/duel_match.rs)

All 301 native workspace tests, warning-denied Clippy, TypeScript and the production build passed. Seven of eight Chromium cases passed together, including local and online scoring, rematches and reconnect. The narrow-screen start button missed its five-second readiness check once and passed three traced repeats. The local preview server and browser report engine fingerprint `c7f509a7225ff049e341584554951ab9e1914e03057a24d0902f31e8d9a7fd6c`.

This slice covers the native room transition. The [private-room promotion slice](16j-spectator-promotion.md) connects that transition to waiting spectators and their browsers.
