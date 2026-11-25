# Slice 02: Native fixture running in the browser

Status: verified, 2026-11-24. Specification: §§8, 20, 26, 32 phase 0.

## Behavior

The browser loads the actual Rust/Rapier WASM module and canonical physics JSON, simulates the falling cuboid, and renders it with Three.js. A diagnostic panel shows completed tick, height, and measured step time. The fixed render clock remains independent of physics. Startup errors produce an accessible error message. Renderer geometry, materials, shadows, observer, animation loop, and WASM instance have explicit disposal.

Modules remain focused: Rust binding, native trace example, build script, render clock, Three.js scene, and page entry. See ADR 0002 for ownership and pinned dependencies.

## TDD and debugging evidence

- Clock tests first failed on the missing implementation, then passed: 60 simulation steps at 30/60/144 display Hz, capped catch-up after suspension, invalid delta rejection.
- The integration test first failed because the browser stayed at its loading state before the app entry point existed. It then passed after the real engine/renderer integration.
- The 601-state browser trace agrees with the native release trace on all position axes within 1e-5 m. Browser console/page error collection was empty.
- A live desktop-to-320px resize exposed temporary canvas overflow before ResizeObserver ran. A synchronous regression test failed before adding CSS max-width containment, then passed after the fix.
- Desktop and 320px screenshots were visually inspected. The desktop platform, falling body, shadows and diagnostics were readable. Narrow layouts stack the diagnostics below the scene. Full dynamic camera framing belongs to slice 11.

## Review corrections

An independent code review found that the parity test could pass even if the visible page froze, and that production builds could bundle stale generated WASM. The test now requires the visible tick to advance beyond 90 and the displayed body to settle near 0.5 m. Temporarily disabling the page's initial animation frame made the strengthened test fail at tick zero; restoring it passed both browser tests. The production build now runs WASM generation automatically. Moving all generated assets aside reproduced a build failure before this change; the corrected build regenerated them and passed. Both corrections were reviewed again.

## Verification

Passed: four Rust integration tests, three clock unit tests, two Chromium browser tests, TypeScript typecheck, strict Clippy, native build, WASM release build, frontend production build. `npm install` after the test-runner update reports zero known vulnerabilities.

Production bundle at this slice: WASM 1,250.33 KB raw / 420.46 KB gzip; JS 484.50 KB raw / 122.78 KB gzip. These are Vite's fixture build sizes, not final-game download or performance measurements. No eight-player performance or release readiness is claimed.

## Next

Add reproducible playground controls (pause/single-step/reset/impulse), structured diagnostics and configuration export. Then implement and stress-test the fifteen-body skeleton before actuation.
