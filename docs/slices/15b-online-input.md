# Controlling a private Skyline fighter

An active player controls only their assigned fighter on the native worker. Both players use WASD or arrows, Shift to run, Space to jump or pull up, J/K or mouse buttons to punch or hold, and E to lift. Movement is rotated into world space with the group camera’s horizontal basis. Focus loss clears held input. A spectator receives the same world and has no input binding.

The browser samples focused controls on incoming authoritative snapshots, normally 30 per second while playing. Publications arriving less than 25 ms after the last transmission coalesce to prevent a burst after a stall. It retains short button edges through the existing input sampler. The server limits incoming gameplay messages to 120 per second, bounds each fighter’s pending inputs independently and expires stale commands after 250 ms. When a browser’s unsent socket queue exceeds 16 KiB, it closes the connection and offers reconnect.

Each socket acknowledges the exact round epoch from its first decoded full snapshot. The server clears that authorization before sending a new epoch. It attaches the acknowledged epoch to each input and the worker verifies it again during simulation. A fresh socket gets a fresh input sequence namespace. Socket ownership changes go through a per-fighter atomic generation, independently of the bounded command queue. Queued records with an old generation are discarded. The current binding is checked when a socket closes, so a late close cannot release the successor’s controls.

## Verification

The Chromium test first failed when the browsers displayed a live match but keyboard input could not change the score. The same test now reloads the guest, has the guest run off the roof in the first round, then the host in the second. Both browsers show the correct winner’s point and advance into round three. Run `npm --prefix web run test:e2e -- online-match.spec.ts`. The worker regression fills one fighter’s queue, replaces its socket generation, verifies all eight old commands are discarded and confirms the new socket can apply sequence zero.

A concurrent Chromium run showed the original interval sampler advancing about one input sequence per second while the native room kept ticking. The server repeatedly expired the held run command. Trace samples showed the host eventually reaching the lip after twenty seconds. The client now uses arriving snapshots as its sampling clock. A unit test disables the window timer and verifies directional input across two rounds. The browser test records bounded native pose, input-axis and acknowledgement history when a score check fails.

The current release path supports two controlling players. Live fighter transforms use full authoritative snapshots. Interpolation, more active fighters, timed disconnect elimination, independent latency measurements and human testing remain open.

The final release workspace run had 265 passing Rust tests and the two known low-stair failures. Formatting, Clippy, the production build, all 32 TypeScript tests and all 65 Chromium tests passed. Two more independent Chromium profiles on the local production preview created a room, played until a physical ring-out, saw the same point and left.
