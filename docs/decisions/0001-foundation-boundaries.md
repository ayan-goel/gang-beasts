# ADR 0001: Shared simulation with pinned native/WASM builds

Date: 2026-11-24. Status: accepted, implements specification §§8–10 and 26.

## Context

The local playground and authoritative server must exercise the same physics/gameplay implementation. The installed Rust toolchain is 1.88.0. The first slice must isolate physics behavior from rendering, transport, scheduling, and uncalibrated character control.

## Decision

Use a Cargo workspace with `brawler-engine` first; add WASM, protocol, server, and benchmarks when their slices need them. Pin Rust 1.88.0 and Rapier 0.26.1, enable enhanced determinism without parallel/SIMD features, and commit Cargo.lock. Rapier's downloaded source is the version-specific API reference. Its nalgebra re-export avoids incompatible math types.

The initial dependency resolution selected ordered-float 5.5.1, which requires Rust 1.90. Lock the compatible 5.0.0 release instead. Dependency upgrades require native and WASM checks.

Use strict versioned JSON configuration, shared by native and browser consumers. This chooses the specification's permitted JSON format instead of its illustrative `.ron` paths. The fixed 60 Hz gameplay timestep is an engine constant, not a user-controlled rate. Bounds on gravity and solver work reject invalid/nonfinite or excessive configuration before allocation.

## Alternatives and consequences

Duplicating physics in TypeScript would invalidate parity and multiply tuning work. A generic ECS or broad scaffold would add interfaces before behavior exists. Neither is needed. Native/WASM exact parity remains unproven until the same fixture is run on both targets; the feature flag alone is not evidence.

One falling cuboid on a fixed platform is a foundation fixture, not the playable character. Replace no ragdoll requirement with this diagnostic geometry. The browser will display its development status explicitly.

## References

- [Pinned Rapier source](https://docs.rs/crate/rapier3d/0.26.1/source/)
- [Rapier determinism requirements](https://rapier.rs/docs/user_guides/rust/determinism/)

API signatures were inspected in the downloaded 0.26.1 crate; the web documentation endpoint was unavailable during initial setup.
