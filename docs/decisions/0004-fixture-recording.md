# ADR 0004: Bounded, versioned fixture recordings in the shared engine

Date: 2026-11-24. Status: accepted. Specification §§26–27, 29, 34.

## Ownership and replay

`FixtureSession` wraps the existing simulation for playground operations. It records only commands actually consumed by completed ticks; an impulse waiting for the next tick is not part of an exported run. Each command identifies the completed tick it affects. Replay validates the entire file, constructs a fresh world, applies commands in order and stops exactly at the recorded endpoint. External impulses are rejected during replay. Reset returns to live mode using the current configuration and starts a new log.

The Rust core handles recording, decoding and replay for both native and WASM. JavaScript controls playback cadence and file selection; it does not reinterpret or simulate recorded physics. A native stdin runner (`replay_fixture`) reproduces browser recordings without a browser. Imported runs start paused at tick zero. Applying tuning also starts paused with a new world/log.

Configuration/import changes construct a replacement first and swap it only after validation succeeds. Failed operations preserve the old world, pending input and playback state. Exported configuration comes from the running session, not from the original bundled defaults.

Export during replay retains the complete source recording, including ticks not yet played. Each successful reset, tuning change or import increments a synchronous session revision. A pending browser file read checks that live revision as well as its selection generation before applying data or reporting an error; older reads cannot replace newer work or affect an unmounted inspector.

## Identity and constraints

The format includes schema version, engine source/compiler fingerprint, producer target/profile, canonical configuration/hash, seed, total ticks and impulse commands. SHA-256 fingerprints the compiler version output plus sorted, length-prefixed engine source paths/content, build script, workspace/engine manifests, lockfile and pinned toolchain file. It excludes checkout location, timestamps and producer target/profile so the tested native/WASM builds can exchange recordings. Configuration hashing uses Rust serialization of the validated configuration. This is source/build compatibility metadata, not a claim that arbitrary compiler flags or targets produce identical physics.

The current fixture has no randomness: seed zero is the only accepted seed. Adding random scenarios requires an explicit versioned seeded implementation. Input-only reproduction is validated for this fixed fixture under the pinned builds; complete game checkpoints/state tracks remain future work. Do not apply this limited result to prediction islands or whole-world rollback.

Slice 04a extends the fingerprint inputs to include the embedded character JSON; see [ADR 0005](0005-passive-character-rig.md). Recordings from earlier engine sources remain intentionally incompatible.

Limits: 256 KiB JSON, 36,000 ticks (10 simulated minutes), 1,024 impulses, strictly increasing nonzero command ticks no later than the recorded end, the existing finite 10 N·s impulse bound, and 4 KiB configuration JSON. The browser checks file size before reading; Rust checks UTF-8 byte size before parsing. Exceeding a live recording budget leaves simulation usable but disables export until reset; incomplete logs are never silently presented as complete.

## Alternatives

JavaScript-only recording would duplicate command semantics and provide no native reproduction. Transform-only playback would hide engine regressions. Unbounded input logs or synchronous replay-to-end on file import would risk memory growth or UI freezes. Per-frame bounded stepping avoids those costs. Full checkpoints are deferred until the simulation actually has controller/RNG/lifecycle state to preserve.
