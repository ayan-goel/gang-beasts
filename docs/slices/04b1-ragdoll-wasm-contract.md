# Slice 04b1: Ragdoll WASM and native trace contract

Status: verified on November 24, 2026.

## Contract

`LocalRagdoll` constructs the same validated rig as the native diagnostic scenario at spawn `[0, 2, 0]`. Its constructor accepts strict physics JSON; `definition_json()` exports the actual rig definition for rendering. `configure()` validates and constructs a replacement before swapping, while `reset()` reconstructs the current configuration. The browser-facing class owns the native rig; `free()` releases it. No browser wall-clock dependency enters the engine.

Body states are copied/owned `Float32Array` buffers in canonical `BodyPart::ALL` order: 15 records ×14 floats. Each record is position XYZ, rotation XYZW, linear velocity XYZ, angular velocity XYZ, mass. Joint states are copied/owned arrays in authored joint insertion order: 14 records ×17 floats.

| Joint float offsets | Field |
| --- | --- |
| 0, 1 | Parent and child anatomical indices (0–14, exactly representable) |
| 2–4, 5–7 | Parent and child world anchor positions |
| 8–11, 12–15 | Parent and child world joint-frame rotations, XYZW |
| 16 | Anchor separation in meters |

Frames use `body_world_transform × joint_local_frame`; this preserves correctness if joint-local rotations are extended later. Rendering consumes these actual frames rather than reconstructing anchors from stale bind positions. Collider lines remain Rapier's debug geometry. These are local diagnostic buffers, not a network wire format or complete checkpoint.

`queue_impulse(part, x, y, z)` accepts the limb argument as a `JsValue`, requires a JavaScript number, then validates finite/integral/range constraints before narrowing. Negative, fractional, nonfinite and out-of-range indices are rejected. Numeric strings, null, booleans and arrays are also rejected: a raw WASM numeric signature would coerce them before Rust validation. Existing impulse magnitude/one-pending-command rules still apply. Exported arrays survive later allocation, configuration, reset and object destruction.

## TDD and verification

Both browser tests initially failed with `LocalRagdoll is not a constructor`. Implemented bindings now reproduce all body and joint-frame fields for initial state plus 240 ticks, with head and left-hand impulses at ticks 1 and 90. The reference is the independent native `ragdoll_trace` executable; maximum absolute field deviation is below 1e-5 under the pinned builds. This remains a finite scenario check, not universal cross-platform determinism.

A second browser test verifies invalid indices/types, rejected tuning throwing and preserving live state, full reset, valid tuning resetting a nonzero tick, and retained exported arrays after `free()`. Review reproduced the numeric coercion bug with a failing browser test before the `JsValue` fix. Native diagnostics tests independently reconstruct both absolute anchors from authored bind offsets and current body poses, and verify current rotated frames, anatomical IDs and buffer offsets. To reproduce the native trace:

```sh
cargo run --quiet --locked --release -p brawler-engine --example ragdoll_trace > trace.json
```

22 native tests, strict Clippy, production WASM/frontend build and all 11 Chromium browser tests passed. Follow-up review also executed the corrected release WASM against the previously coerced values and confirmed rejection, with a valid numeric limb index still usable. No consequential findings remain. The build retains the previously documented Vite chunk advisory and wasm-pack notices; WASM is about 512.29 KB gzipped. The browser UI still renders the cuboid in this contract slice; 04b2 adds scenario selection, rig meshes and joint overlays using these tested buffers.
