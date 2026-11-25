# Slice 03b: Fixture recording, replay and editable tuning

Status: verified on November 24, 2026.

## Delivered behavior

The playground records completed-tick impulses automatically from reset, exports a versioned JSON file, imports compatible recordings paused, and stops at the recorded end. A gravity/solver form validates through the native engine and replaces the fixture transactionally. Configuration export reflects the running tuned world. File imports are byte-limited, strictly decoded and checked against engine/config hashes. Old asynchronous file reads cannot replace newer selections, reset/tuning/import operations or an unmounted inspector. Export during replay preserves the full imported run.

See [ADR 0004](../decisions/0004-fixture-recording.md) for contracts, identity and limits. Existing playback/diagnostic behavior remains covered.

## Evidence

- Native tests first failed on missing `FixtureSession`; implemented replay then matched all completed states of a 240-tick run with three impulses exactly on native builds.
- Invalid schemas, build/config hashes, seed, tick order, duplicate/out-of-range commands, impulses and oversized/malformed files preserve the current world.
- Tests exercise command and duration recording limits, reject truncated export, preserve simulation usability and restore recording after reset.
- The browser test failed on the missing Apply tuning control before implementation. It now tunes gravity/solver values, exports real WASM input, runs the native CLI on that file, imports it into the visible page, and verifies automatic exact-end pause and incompatible-import rejection.
- A separate browser/native test compares all 14 state fields over 241 states with gravity −7, 12 solver iterations and three nonvertical impulses, within 1e-5 per field. This is fixture evidence only, not universal cross-platform determinism.
- Playback tests cover paused import, exact replay completion, reset to live mode, and rejection preserving pending input/playback.

## Verification status

13 native integration tests, 10 TypeScript tests and 9 Chromium browser tests passed. Native format/strict Clippy checks, TypeScript checking and production WASM/frontend builds passed. Commands are the README verification commands; browser tests are in `web/tests/recording.spec.ts`, with native cases in `crates/engine/tests/recording.rs`. The environment remains the pinned foundation toolchain, macOS arm64 and Playwright Chromium.

Review found two issues and both were reproduced with failing browser tests before fixing: a delayed file read could overwrite a newer tuned world, and the HTML gravity step rejected valid imported −9.815 when changing only solver iterations. A live session revision and unrestricted fractional number step fix those cases. Follow-up review found no remaining consequential issues in this slice.

Visual inspection at 1440×900 confirmed the tuning/recording controls and persistent scene; checks at widths 320, 768 and 1024 found no horizontal overflow, and the browser reported no page errors. The local inspection artifact is `.cache/recording-tuning.png` (ignored). Production output is about 463.29 KB gzipped WASM and 195.41 KB gzipped JavaScript. Vite's >500 KB raw chunk advisory remains; these sizes are diagnostic-fixture measurements, not full-game performance acceptance. wasm-pack reports its known prebuilt-generator fallback and newer-version notice; the pinned build succeeds.

## Remaining scope

No random fixture selection, ragdoll inputs, game checkpoints, public replay library or video export is claimed. The next slice builds the physical skeleton and extends the same playground with character diagnostics. The full MVP acceptance record remains incomplete.
