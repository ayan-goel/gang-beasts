# Live lobby readiness

A seated player opens the native server’s same-origin WebSocket through the browser proxy. The handshake validates the origin, session cookie and existing lobby membership before accepting the upgrade. The first binary message names the already joined room. The server binds subsequent commands to that seat. A ready command contains one Boolean and has no player ID.

The current connection owns the ready flag. Opening a second tab resets that flag and replaces the previous connection. Each change and disconnect checks both the session and a server-assigned generation. A late close from the old tab cannot clear the new tab’s choice or change a replacement player in the same slot. An active command renews the idle session expiry. A closed connection leaves a waiting lobby seat that can reconnect explicitly.

## Roster delivery

The server sends the current seat, role, ready flags, names and colors in its welcome. A membership change increments the roster revision and sends replacement names and colors together with the flags. Ready-only updates carry the same revision and omit names. All recipients see a consistent mapping when a slot is reused. The browser keeps a member-only HTTP fallback for the roster after a socket disconnects. It offers **Reconnect** to regain ready controls.

The initial implementation fetched the roster separately from each browser for every membership change. A combined browser run exhausted the existing 120-request peer budget. Sending the names in the already authenticated room-state message removes those HTTP requests. The admission limit continues to apply.

The socket checks for changed state every 150 ms, verifies membership and sends only changes. The HTTP service reserves up to 512 socket permits. An incomplete Connect expires after three seconds. Incoming frames and messages are limited to 4 KiB, outgoing writes have a two-second deadline and each attached lobby socket permits 32 incoming messages per second. These bounds concern low-rate lobby controls. Match input and state need their own transport budgets.

## Verification

The Rust protocol and TypeScript decoder test exact bytes, ownership and lengths. A directory test covers replacement, late disconnect and slot reuse. Two isolated Chromium sessions toggle and cancel readiness and see a closed guest return to waiting. A separate same-profile case replaces the first connection and explicitly reconnects from the earlier tab. Both cases use the native server via Vite’s WebSocket proxy.

Run `cargo test --locked --release -p brawler-protocol -p brawler-server`, `npm --prefix web test -- --run src/lobby` and `npm --prefix web run test:e2e -- room-ready.spec.ts`. Ready expresses a player's intent to play. Compatible-map acknowledgement, host start, match input and live fighter snapshots are separate implementation steps.

On November 25, 2026, the production build and 31 TypeScript tests passed. All 62 Chromium tests passed together with the final socket roster. The Rust release workspace reported 262 passing tests and the two tracked [low-stair traversal failures](06c2b4b-stairs.md). Rust formatting and Clippy passed. A separate pair of browser profiles also verified a guest's ready flag through the production-preview proxy.
