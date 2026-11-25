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
