# Slice 04a: Native passive physical character

Status: verified on November 24, 2026.

## Scope

Versioned original 1.6 m anatomy, 15 dynamic bodies, 14 anatomical impulse joints and 40 kg actual body mass. The native diagnostic world uses the existing 60 Hz physics configuration, a 12×12 m floor, a bounded tick-boundary impulse and full reconstruction on reset. [ADR 0005](../decisions/0005-passive-character-rig.md) defines geometry, identity frames, axes, limits, materials, validation and alternatives.

Slice 04 is split at a testable boundary: 04a constructs and verifies the native rig; 04b exposes the same rig in WASM/Three.js, compares its trace and provides visible anatomy/anchor diagnostics. Motors and standing remain slice 05.

## Tests and evidence

- Initial integration tests failed because the character asset and native types did not exist.
- Actual Rapier sets contain 15 bodies, 16 colliders (including floor) and 14 joints. Each body has its specified mass and total mass is within 1e-4 kg of 40 kg. Initial joint anchor errors are below 1e-6 m.
- A 1,200-tick drop from 2 m above the floor, with a `[3, 0, 1]` N·s head impulse, stays finite, keeps all body centers above −5 cm and component speeds below 20 m/s. Maximum measured anchor separation is **0.009195 m**, below the 5 cm fixture gate. The unpowered head ends below 1 m, confirming collapse.
- Actual joints have no motors, locked translation, the authored angular limits and fixed/revolute axes. A separate 600-tick two-ragdoll contact scenario observes nonadjacent self-contact and inter-character contact while adjacent contacts remain absent.
- Reset clears queued impulses and reproduces fresh construction followed by 120 ticks exactly on this native build.
- Strict parsing rejects schema/field errors, malformed topology, wrong mass, invalid dimensions/anchors/limits and oversized input.
- Review reproduced two bugs with failing tests: a capsule head/incorrect hinge axes were accepted, and shifted geometry could spawn below the floor despite a valid origin. Anatomical shape/axis checks and transformed collider AABB checks now reject those cases. Version-one identity bind/joint frames are explicitly documented.

Reproduce native evidence with `cargo test --locked -p brawler-engine -- --nocapture`. These finite fixture runs do not certify all editable configurations, eight-player stress, tunneling under arbitrary actuation, whole-game replay parity or human playability. No runtime velocity clamp or technical-abort recovery is claimed yet.

## Verification

21 native tests (including eight new anatomy/validation/contact tests), native formatting, strict Clippy, production WASM/frontend build and all nine Chromium browser regressions passed. The final browser suite verifies the cuboid's native/WASM trace and recording compatibility after rebuilding the changed engine. No browser ragdoll behavior is claimed yet. Follow-up review confirmed both validation fixes and the documented frame convention, with no remaining consequential findings.

The character asset SHA-256 is `e9814cf89e7ca7ae2504d236982af45f6e1dbd69c84239d5d4d8d5101a1cdcd1`. Verification used the pinned foundation toolchain on macOS arm64. The build retains the known Vite chunk-size advisory and wasm-pack generator fallback/update notices; these are not new failures. No UI changes are included.
