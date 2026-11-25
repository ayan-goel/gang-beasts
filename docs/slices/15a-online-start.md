# Two-browser match startup

Two ready players can enter a native Skyline match from the private lobby. The host requests a start. The service takes a snapshot of the seats, sessions and live connection generations, opens a simulation worker outside the directory lock, then validates the same ready roster before publishing the match. Unavailable worker capacity leaves both seats in the lobby. Closing a rejected reservation releases the worker.

The first native fighter belongs to the lower occupied lobby slot. Identities and selected colors are frozen at launch. Later arrivals have a spectator assignment. The server sends recipient-specific full snapshots over the existing authenticated socket at up to 30 Hz. Each browser uses the fighter definitions and map it already loaded, draws the authoritative body transforms with Three.js, and reads scores and phases from the same snapshot. Reloading retains the room seat and requires a fresh compatibility check before returning to the live round.

The current native fighting world has two rigs. The host may start with exactly two occupied, ready seats in a lobby of any supported capacity. More fighters require the multi-rig simulation work. A room releases its match when the last original participant leaves. Timed disconnect forfeits, eight active fighters and online rematch remain open.

Run `npm --prefix web run test:e2e -- online-match.spec.ts` for the two-browser scene and round progression. Run `cargo test --locked --release -p brawler-server only_a_ready_host` for host authorization, capacity recovery and cleanup. The [remote control slice](15b-online-input.md) adds seat-bound gameplay inputs to the same stream.
