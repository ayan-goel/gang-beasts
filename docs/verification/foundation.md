# Foundation verification — 2026-11-24

Scope: slices 01–02 only. This is not a game-performance or release certification.

## Environment and identity

- macOS 26.6.2 (25G83), arm64; Rust 1.88.0; Node 20.18.0.
- Rapier 0.26.1, enhanced-determinism; wasm-bindgen 0.2.100; wasm-pack 0.13.1.
- Playwright 1.65.1, Chromium 159.0.8154.6, headless; 1440×900 baseline viewport.
- Physics configuration SHA-256: `514d8064d10f7046f8445bd9ccad203e121dfd6f41129daaa522344740d534fc`.
- Cargo.lock SHA-256: `1611989a19e28604ab146abc4688d85f55f37c958b01d8d335348ba0689aaa19`.
- Frontend lockfile SHA-256: `0a67962ad41f97edd0b79f7cff31acd1bd2286e10a2483d5db59f2bf61a426ef`.

## Automated evidence

| Check | Result |
| --- | --- |
| Native integration tests | 4 passed |
| Native formatting/strict Clippy | Passed |
| Native build and WASM release build | Passed |
| Clock unit tests | 3 passed |
| TypeScript strict typecheck | Passed |
| Playwright native/WASM trace and resize regression | 2 passed |
| Frontend production build | Passed |
| npm install audit after Vitest update | Zero vulnerabilities reported |

Run commands are in the root README. `web/tests/foundation.spec.ts` invokes the native release trace itself; it does not compare two copies of JS code or mock physics. Initial red failures and the resize diagnosis are recorded in the slice documents.

Review hardening: the browser test also requires the visible page to advance beyond tick 90 and settle near Y=0.5. A deliberate frozen-loop mutation failed this assertion; the restored source passed. Production builds were verified with generated assets removed: the new prebuild regenerated the WASM/glue/config and completed successfully.

## Production and visual smoke

Served `web/dist` using `vite preview` on loopback port 4173. A fresh Chromium context loaded the packaged WASM, reached more than 90 real simulation ticks, and produced no console/page errors. Resizing to 320, 768, and 1024 pixels produced no horizontal overflow after the containment fix. Desktop and narrow screenshots were visually inspected; scratch screenshots are ignored in `web/test-results`.

The development preview also returned HTTP 200 and loaded the actual simulation. No Chromium result is a substitute for the release-required Firefox/Safari tests. No hardware frame-time percentile, human playtest, or multiplayer behavior has been certified.
