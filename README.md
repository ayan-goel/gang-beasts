# Physics brawler

A browser multiplayer physics brawler built around a shared Rust/Rapier simulation and Three.js presentation. Development is at the foundation stage; there is no playable MVP yet.

The [technical specification](TECHNICAL_SPEC.md) is canonical. See the [documentation index](docs/README.md) for implementation order, decisions, and acceptance evidence.

## Native development

Install Rust using rustup. The repository pins Rust 1.88.0 with rustfmt and Clippy.

```sh
cargo test --workspace --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
```

The engine has no network, rendering, or wall-clock dependency. `config/physics.json` is the initial versioned configuration.

## Browser playground

Use Node 20.18.0 (`.nvmrc`). Install project-local wasm-pack once, then build the shared engine before starting Vite:

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.88.0
cargo install wasm-pack --version 0.13.1 --locked --root .tools
npm --prefix web ci
npm --prefix web run wasm
npm --prefix web run dev
```

Open the localhost URL printed by Vite. During development, re-run `npm --prefix web run wasm` after changing Rust or configuration; generated browser files are ignored. Production `npm run build` always rebuilds WASM and copies the current canonical configuration first. wasm-pack installs the matching wasm-bindgen generator on first use. React manages the playground inspector; per-frame rendering stays imperative.

```sh
npm --prefix web run typecheck
npm --prefix web test -- --run
npm --prefix web exec -- playwright install chromium
npm --prefix web run test:e2e
npm --prefix web run build
```

Browser integration tests start their own local server if needed. They compare a 600-tick WASM trace against the native release fixture and test canvas resize containment. The production static bundle is in `web/dist`; it is a diagnostic fixture, not an online game.

## Playground controls

Pause to enable exact single-step. Reset reconstructs the fixture and preserves the paused/running mode. Slow motion changes playback cadence without changing physics dt. The upward impulse is queued once for the next tick. Enable **Show colliders** to inspect Rapier geometry.

**Apply tuning** validates gravity/solver settings and restarts paused. **Export configuration** downloads the running configuration. **Export recording** saves completed ticks and impulses; **Import recording** loads a compatible file paused at tick zero. Resume or single-step to its exact endpoint, then reset to return to live play. During replay, export preserves the entire imported recording. Logs are limited to 10 simulated minutes, 1,024 impulses and 256 KiB; reset starts a new log after a limit is reached.

Reproduce a browser recording with the same engine sources/toolchain natively:

```sh
cargo run --quiet --locked --release -p brawler-engine --example replay_fixture < fixture-recording-v1.json > trace.json
```

The trace includes initial state and every completed tick as position, XYZW quaternion, linear velocity, angular velocity and mass (14 floats). See [recording contracts and evidence](docs/slices/03b-recording-tuning.md). This validates the cuboid fixture, not full-game deterministic replay.
