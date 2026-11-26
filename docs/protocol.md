# Realtime protocol v1

## Input records

Each WebSocket binary message carrying player input contains exactly 18 bytes. Kind `0x01` and version `0x01` precede the sequence and advisory client tick (two little-endian u32s), signed X/Z movement bytes, then little-endian u16 held, pressed and released masks. The [canonical field table](../TECHNICAL_SPEC.md#174-input-wire-contract--exactly-18-bytes) defines offsets and button bits. This record carries no player ID. An authenticated connection must establish which slot may send it.

A decoder rejects extra bytes, truncated records, unknown kind/version, movement byte −128, reserved button bits and single edges inconsistent with the final held bit. Both edges on a button encode a press followed by release when finally up, or a release followed by press when finally down. A button held across samples needs no new edges. The receiver normalizes diagonal input and uses half-range modular sequence ordering.

The browser sampler retains repeated taps as up to four ordered records per interval. An overflowing burst clears unsent intent, queues a neutral record for previously transmitted holds, and reports an error. Focus-loss neutralization also cancels unsent taps. The sampler returns heartbeat records even with no edges. The room must expire stale input independently.

## Verification

Rust and TypeScript independently reproduce the same [golden bytes](../tests/fixtures/protocol/input-v1.json). The Rust suite also checks every pair of legal signed movement components, the half-range boundary, wraparound, and malformed records. Browser-side unit tests cover coercion and truncation hazards, analog rounding, short taps, bursts, neutralization and sequence wrap.

Run `cargo test --locked -p brawler-protocol` and `npm --prefix web test -- --run src/net`.

## Full Skyline snapshots

Kind 2, v1 uses little-endian integers and IEEE f32 values. The complete message is a 20-byte header, N consecutive 32-byte transforms, then a gameplay block. A full record replaces the previous world. All clients must have the compatible Skyline map and character definition before applying it.

| Header offset | Value |
| --- | --- |
| 0, 1 | Kind and version, u8 each |
| 2 | u16 flags, bit 0 full (required), bit 1 input acknowledgement valid |
| 4, 8, 12 | Completed room tick, recipient input acknowledgement, baseline tick (u32 each). Absent acknowledgement and full baseline are encoded as zero. |
| 16, 18 | Transform count and gameplay byte count (u16 each) |

Each transform contains body ID u32 at 0, XYZ position at 4 and XYZW rotation at 16. All components must be finite and squared quaternion length within 0.002 of one. Send at most 256 transforms, in strictly increasing ID order. Fighter slot S uses IDs 1 + 16S + body index (0–14 in the shared character order). Include all 15 bodies even for an eliminated slot. Arena objects use 65536 + map object ID (1–65535). Omit retired objects. Zero denotes an empty grip. Stable IDs are scoped to the round epoch.

| Gameplay offset | Value |
| --- | --- |
| 0–3 | Block version 1, match phase (countdown/playing/round results/match results = 0–3), winning slot (255 for none), fighter count (2–8), all u8 |
| 4, 6 | Remaining phase ticks u16 and reserved zero u16 |
| 8 | Round epoch u64 |
| 16, 20 | Round number starting at 1 and completed arena tick, u32 each. Map schedules use the latter for warning, detachment and gust progression. |
| 24 onward | One 24-byte record per slot in ascending order |

Fighter offsets 0–4 are slot, consciousness (active/knocked out/recovering/eliminated = 0–3), score, left and right hand phase (idle/wind-up/strike/recovery/reach/gripping = 0–5). Bytes 5–7 are zero. Health and stamina are f32 at 8 and 12 in [0,100]. Left and right grip target body IDs are u32 at 16 and 20. A gripping hand must name a body from this snapshot outside its own rig. Every other hand must have a zero target.

Room snapshots are due on even completed room ticks (30 Hz). A join or resync may request a full record on any tick. Reliable transient events have a separate lifecycle. A full snapshot carries the state needed to display the current world without replaying earlier hits or warnings.

The independent [binary fixture](../tests/fixtures/protocol/full-skyline-v1.bin) has 31 transforms, two fighters, a zero-valued valid acknowledgement, a crate grip and an epoch above the JavaScript safe integer limit. Run `cargo test --locked -p brawler-protocol` and `npm --prefix web test -- --run src/net` to check both endpoints.

## Lobby controls

The authenticated `/ws` endpoint uses binary messages with one-byte kind and version 1. Multibyte fields are little-endian. Slots and color IDs range from 0 to 7. The [lobby implementation](slices/16c-lobby-ready.md) defines ownership, rate and timeout behavior.

| Kind | Layout after the two header bytes |
| --- | --- |
| 16 Connect | Six ASCII room-code bytes, matching the authenticated seat |
| 17 Welcome | Own slot (u8), then room-state fields and a complete roster |
| 18 SetReady | A u8 Boolean, 0 or 1 |
| 19 RoomState | Room-state fields, followed by a roster when its revision changes |

Room-state fields are occupied-slot mask, ready-slot mask, host slot (u8 each), then u32 roster revision. The host and receiver must be occupied. Ready bits must belong to occupied slots. A full roster starts with capacity (u8, 2–8), then one record per occupied slot in ascending order. A player record contains color (u8), UTF-8 name length (u8, 1–96), then the name bytes. The server validates the 24-character limit during HTTP admission. The browser checks lengths before reading and requires the packet to end at the last declared field. A same-revision update has exactly nine bytes and keeps the preceding roster. Welcome starts a new connection and always includes the roster.

A readying client sends MapLoaded (kind 20, version 1, exactly 130 bytes). The payload consists of the 64 lowercase ASCII hexadecimal characters of the local engine SHA-256 followed by the 64 characters of its bundled Skyline map SHA-256. The recipient receives MapLoadResult (kind 21, version 1, one status byte). Zero acknowledges a match, one identifies incompatible content, and two rejects readiness before map loading. SetReady(false) always permits withdrawal. The [loading flow](slices/16d-map-loading.md) documents retry, timeout and connection replacement.

StartMatch is exactly two bytes (22, 1). StartRejected is three (25, 1, reason), where 1 means the ready roster or authority changed and 2 means worker capacity is unavailable. MatchStarting (23, 1) includes the recipient's native fighter index (u8, 255 for spectator) and two original fighter identities in native index order. For three to eight fighters, MatchStarting uses kind 26, version 1, recipient index, and fighter count (all u8), followed by that many identities. Slots must be strictly increasing and the recipient must be a valid native index or 255. Each identity has lobby slot, color and UTF-8 length (all u8) followed by the bounded name. Names and colors survive roster changes. The next kind-2 full snapshot supplies the round and world. [Startup rules and evidence](slices/15a-online-start.md)

A fighter sends RoundApplied (24, 1 followed by a little-endian u64 round epoch) after decoding the first full snapshot for a new epoch. The server accepts input only for the acknowledged published epoch and the bound fighter. The server checks that epoch again when the worker drains the record. No player ID or epoch is added to the original 18-byte InputFrame. Spectators have no fighter binding. [Remote controls and ownership](slices/15b-online-input.md)

MatchAvailable (28, 1) follows the initial Welcome when a match is already running. The browser automatically starts content verification on that socket. ReturnToLobby (27, 1) ends the match feed and prompts a fresh authenticated lobby connection. Both records are exactly two bytes. [Disconnect and return-to-lobby rules](slices/16e-disconnect-grace.md)

At MatchResults the server sends RematchPermission (29, 1, allowed), where allowed is zero or one. It republishes the permission on host transfer. ReturnForRematch (30, 1) is exactly two bytes, is host-only and requires the current match to be finished. Success produces ReturnToLobby on current members’ sockets. The next match has a fresh 32-bit serial in the high half of the u64 epoch, with the displayed round number in the low half. [Rematch authority and epochs](slices/16f-online-rematch.md)

Clients opting into automatic recovery connect to `/ws?resume=1`. The initial Connect stays unchanged. After successful admission, the server sends ResumeTicket (31, 1, little-endian u64) before Welcome. The browser retains those eight bytes in memory for that room connection. A retry starts with Resume (32, 1, six room-code bytes, original u64). The server requires the authenticated session, a disconnected unexpired seat, and the seat’s current ticket. A successful retry receives the same ticket and a fresh Welcome. MatchAvailable starts normal loading when a match exists. Otherwise ResumedLobby (33, 1) confirms the return to the lobby. Clients using plain `/ws` retain the original handshake. [Recovery lifecycle](slices/16g-automatic-reconnect.md)
