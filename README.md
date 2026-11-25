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

Open the localhost URL printed by Vite. During development, re-run `npm --prefix web run wasm` after changing Rust or configuration; generated browser files are ignored. Production `npm run build` always rebuilds WASM and copies the current canonical configuration first. wasm-pack installs the matching wasm-bindgen generator on first use. React UI arrives with the interactive playground slice; per-frame rendering stays imperative.

```sh
npm --prefix web run typecheck
npm --prefix web test -- --run
npm --prefix web exec -- playwright install chromium
npm --prefix web run test:e2e
npm --prefix web run build
```

Browser integration tests start their own local server if needed. They compare a 600-tick WASM trace against the native release fixture and test canvas resize containment. The production static bundle is in `web/dist`; it is a diagnostic fixture, not an online game.
