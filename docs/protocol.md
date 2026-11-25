# Realtime protocol v1

## Input records

Each WebSocket binary message carrying player input contains exactly 18 bytes. Kind `0x01` and version `0x01` precede the sequence and advisory client tick (two little-endian u32s), signed X/Z movement bytes, then little-endian u16 held, pressed and released masks. The [canonical field table](../TECHNICAL_SPEC.md#174-input-wire-contract--exactly-18-bytes) defines offsets and button bits. This record carries no player ID. An authenticated connection must establish which slot may send it.

A decoder rejects extra bytes, truncated records, unknown kind/version, movement byte −128, reserved button bits and single edges inconsistent with the final held bit. Both edges on a button encode a press followed by release when finally up, or a release followed by press when finally down. A button held across samples needs no new edges. The receiver normalizes diagonal input and uses half-range modular sequence ordering.

The browser sampler retains repeated taps as up to four ordered records per interval. An overflowing burst clears unsent intent, queues a neutral record for previously transmitted holds, and reports an error. Focus-loss neutralization also cancels unsent taps. The sampler returns heartbeat records even with no edges. The room must expire stale input independently.

## Verification

Rust and TypeScript independently reproduce the same [golden bytes](../tests/fixtures/protocol/input-v1.json). The Rust suite also checks every pair of legal signed movement components, the half-range boundary, wraparound, and malformed records. Browser-side unit tests cover coercion and truncation hazards, analog rounding, short taps, bursts, neutralization and sequence wrap.

Run `cargo test --locked -p brawler-protocol` and `npm --prefix web test -- --run src/net`.
