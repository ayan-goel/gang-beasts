# Slice 01: Native fixed-tick physics fixture

Status: verified natively, 2026-11-24. Specification: §§8–10, 26, 32 phase 0.

## Change and contract

Introduced a pinned Cargo workspace, strict versioned physics configuration, and a shared engine with no wall-clock or presentation dependency. `Simulation::new` validates configuration before creating the fixture; `step` advances exactly 1/60 second; `tick` reports completed steps from zero. A one-kilogram cuboid falls onto a fixed platform with top Y=0. This is diagnostic geometry, not a character implementation.

`PhysicsConfig::from_json` rejects unknown fields, missing fields, unsupported schema versions, nonfinite/out-of-range gravity, and solver counts outside 1–32. Public errors are structured. Internal Rapier handles remain private.

## TDD evidence

The four integration tests were added first. `cargo test --workspace --locked` failed on missing `PhysicsConfig` and `Simulation` exports before implementation. After implementation, all four passed:

- Body falls over 30 ticks and settles within 2 cm of Y=0.5 after 600 ticks.
- Two independently rebuilt identical worlds produce identical positions through 120 ticks.
- Invalid gravity, excessive/zero solver counts and unsupported versions are rejected.
- Committed JSON matches native defaults; unknown fields are rejected.

## Verification

Host: Apple Silicon macOS, Rust 1.88.0, Rapier 0.26.1, enhanced-determinism, debug build. Dependency identity is Cargo.lock.

Passed: `cargo test --workspace --locked` (4 integration tests), `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo build --workspace --locked`.

No performance benchmark, WASM parity, ragdoll stability, or gameplay acceptance is claimed. The next slice exposes this same fixture to WASM and a browser renderer.
