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

The engine has no network, rendering, or wall-clock dependency. `config/physics.json` is the initial versioned configuration. Browser commands will be documented when that slice is verified.
