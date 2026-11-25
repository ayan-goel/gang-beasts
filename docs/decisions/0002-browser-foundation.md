# ADR 0002: Owned WASM fixture data and an independent render clock

Date: 2026-11-24. Status: accepted. Specification: §§8, 20, 26–27.

## Decision

Expose the native `Simulation` through wasm-bindgen 0.2.100, built using project-local wasm-pack 0.13.1. The wrapper accepts the same validated JSON, advances one tick per call, returns a bigint completed tick, and returns an owned Float32Array position. The caller frees the instance on disposal. The future render-state buffer will replace the fixture-specific position method when entities arrive.

Owned arrays avoid stale borrowed views after WASM memory growth. At one fixture body, the copy cost is negligible; future batched transform buffers must preserve explicit lifetime semantics. Render snapshots will remain separate from replay checkpoints.

The local render clock uses fixed 1/60 steps, caps catch-up at four steps, records discarded time, and preserves fractional debt. It does not pass render deltas into physics. Dropping suspended time is a local diagnostic policy; the authoritative server has the distinct overload rules in §18.3.

The build script copies canonical JSON into ignored generated assets after the WASM build. Vite bundles it with the application. Production `prebuild` always invokes this script so changes cannot silently ship stale physics. Development changes to Rust/config require an explicit WASM rebuild; never edit the generated copy.

## Frontend versions and tooling

Pin the installed Node 20.18.0, Vite 6.4.6, Three.js/types 0.180.0, TypeScript 5.9.3, Playwright 1.65.1, and Vitest 4.1.12. npm registry metadata confirms the Vite/Vitest versions support this Node major. Commit the lockfile. The initial Vitest 3.2.7 audit reported vulnerabilities; upgrading to 4.1.12 resolved them (zero reported on installation).

The first diagnostic shell uses static markup and Three.js. React is reserved for the interactive UI slice, not for per-frame transforms. No proprietary assets are used; the fixture consists of procedural cuboids and a grid. This does not fulfill the final original-character asset gate.

Chrome DevTools MCP is unavailable in this session. Use Playwright Chromium for live WASM/WebGL, console checks, screenshots, and regression tests. Current fixture parity uses a 1e-5-meter tolerance across 601 states; it does not prove all native/WASM gameplay is bit-identical.

## Alternatives

Variable-delta physics makes simulation depend on display refresh. A second JavaScript physics implementation defeats shared-engine verification. Borrowed WASM buffers add unnecessary lifetime risks before batching is needed. These alternatives were rejected.
