# Online movement interpolation

The renderer keeps up to 32 decoded full snapshots per match. A render frame selects one server tick for the entire scene. Fighter limbs and rooftop objects use linear position interpolation and shortest-path quaternion SLERP. The HUD and object visibility come from the earlier authoritative state until the next snapshot’s tick is reached. This keeps a retired walkway, knockout or point synchronized with the displayed scene.

The original slice used the timing policy below. The [current presentation clock](15i-snapshot-clock.md) sizes the buffer from the observed delivery window.

The initial render delay is five simulation ticks (83 ms). The estimated jitter is a running average of the difference between arrival spacing and authoritative tick spacing. The delay moves at most 0.08 ticks per accepted snapshot within four to six ticks (67–100 ms), tending toward four after sustained stable delivery. Presentation time cannot run backward when a delayed packet arrives. Ticks unwrap across u32 rollover. Duplicate, backward and stale-round records cannot move the display.

## Missing and discontinuous state

After the latest snapshot, the renderer extends the last observed position velocity for at most six simulation ticks (100 ms) and holds the last orientation. It then freezes the poses and shows a waiting message. A new round, more than twelve missing simulation ticks between samples or more than 250 ms between arrivals starts a fresh history. Spawns and deletions appear at their authoritative boundaries. A same-epoch displacement larger than the greater of 1.5 m and 0.4 m per elapsed simulation tick holds the older pose until the boundary. A discontinuous limb suppresses blending for its entire fighter. These are initial rendering parameters pending latency and human playtests.

Input and round acknowledgements still use the newest authoritative state. The display delay applies to presentation alone. The HUD identifies the player's own fighter and shows the server-derived round clock.

## Verification

TypeScript fixtures cover a sign-flipped quaternion, fractional motion, shared limb discontinuities, removal and spawn boundaries, epoch replacement, tick rollover, monotonic presentation time after a delayed sample, adaptive delay, bounded history and freezing after loss. The Chromium stream test holds ordered full snapshots on one browser connection, checks that the arena reports a stall while its peer remains in play, and then releases those snapshots to verify recovery within the same round. The physical ring-out and shared scoring test also runs with interpolation enabled. The controlled stall does not measure Internet latency or establish the human acceptance threshold.

The production build, 37 TypeScript tests and all 66 Chromium tests passed with interpolation enabled. A separate check against the built preview reproduced an independently controlled ring-out and matching scores in two browser profiles.
