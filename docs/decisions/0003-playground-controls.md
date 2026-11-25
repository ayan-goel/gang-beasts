# ADR 0003: Tick-boundary diagnostic commands and independent playback

Date: 2026-11-24. Status: accepted. Implements specification §§10, 26–27.

## Native ownership

Diagnostic impulses use one bounded command slot per fixture. Enqueue rejects nonfinite vectors and magnitude above 10 N·s before mutation; a second outstanding impulse is rejected. The next physics tick consumes the command once. A one-kilogram test body therefore receives at most 10 m/s of added speed per tick. This diagnostic limit is not a character jump/throw tuning decision.

Reset reconstructs the world from its validated stored configuration, clearing bodies, solver caches, tick and pending commands. Teleporting the existing body would preserve hidden solver/sleep/contact state and would not reproduce a fresh fixture.

Rapier's `debug-render` feature supplies collider line geometry from real shapes and transforms. The engine returns data without depending on Three.js. The debug pipeline is retained per world; the browser updates a line buffer only while the overlay is enabled. An independently drawn approximation of physics geometry was rejected.

## Browser boundary

`fixture_state()` returns an owned 14-float buffer: position XYZ, rotation XYZW, linear velocity XYZ, angular velocity XYZ, mass. `collider_lines()` returns owned XYZ endpoints in pairs. These fixture-specific diagnostics are not the future multiplayer snapshot protocol or replay checkpoint format. `queue_impulse(x,y,z)` throws the native validation error through WASM; `reset()` preserves the validated configuration.

React 19.3.1 owns the low-rate inspector; Three.js and the animation loop own per-frame transforms. The runtime frees the WASM instance, GPU resources and resize observer on teardown, including React StrictMode's development setup/cleanup cycle.

Playback speed scales accumulated wall time, never physics dt. Pause/speed changes clear fractional playback debt while preserving the cumulative discarded-time diagnostic. Reset clears both. Controls reanchor the frame timestamp so a frame after resume/reset cannot consume time preceding that action. The clock is injectable in unit tests to reproduce long suspended intervals without flaky waits. Explicit single-step is allowed only while paused.

## Scope

This slice exports the exact versioned configuration but does not yet edit it. Recording/replay, editable tuning, seeded scenario selection, contact/force overlays, and character-specific overlays remain separately tracked work. No game-fidelity requirement is reduced by the fixture UI.
