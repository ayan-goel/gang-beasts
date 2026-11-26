# Stamina during a grip

The scoreboard places a meter underneath each fighter who holds a grip or has spent stamina. The number belongs to the fighter even when both hands are occupied. A full, idle fighter takes up only the score row. Eliminated fighters, countdowns and results have no meters. Local pause also hides the meters.

The shared meter reads the local WASM state or the remote snapshot used for grip and condition feedback. The displayed percentage rounds upward, with 100 reserved for a full pool and zero for actual exhaustion. At 25 points or less the label changes to “Low stamina.” This threshold controls the label only. Time remaining on a grip depends on the hand count, load and climbing effort. The percentage stays outside the live announcement regions.

## Verification

- The [meter tests](../../web/src/app/StaminaMeter.test.tsx) cover a full fighter with and without an attachment, the warning boundary, and fractional values near empty and full.
- In [local Skyline](../../web/tests/skyline-grips.spec.ts), Coral uses the keyboard to grab and lift a crate. Coral’s meter drops while Blue’s stays hidden. After release and recovery, Coral’s meter disappears.
- In a [two-browser room](../../web/tests/online-stamina.spec.ts), the test steers Mint toward Blue and holds the normal left-hand key. Both players see the warning while the grip is still attached, then see the grip disappear and the meter refill. Both packet observers check that held stamina fell below one point, the pool returned to full, and the hand stayed free while its button was still held.

Both stamina scenarios passed against the packaged build in Chromium on loopback. The web unit suite passed 67 tests. Human judgments about how much warning a player needs during a fight remain open.

In a separate reconnect regression, the macOS software-rendered browser delayed the holder’s outgoing input for 345 ms during peer reload. The trace showed a release with about 90 stamina remaining, after the server’s 250 ms stale-input deadline. The multi-browser grip fixture now uses the Mac GPU, as the eight-player fixture already does. Four consecutive traced reconnect runs preserved the grip through reload.
