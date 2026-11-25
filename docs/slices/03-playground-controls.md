# Slice 03: Interactive diagnostic fixture

Status: verified, 2026-11-24. Specification: §§26–27.

## Behavior

The browser now offers pause/resume, one-tick stepping, 0.5×/0.25× slow motion, full reset, one queued upward 4 N·s impulse, real collider wireframes, and export of the current physics JSON. It displays authoritative height, velocity, mass, tick, measured step duration, discarded playback time, and collider segment count. The desktop inspector scrolls independently so controls cannot enlarge the physics viewport.

The native API rejects invalid impulses and duplicates before state changes, applies accepted impulses at the next tick, and recreates the complete fixture on reset. Rendered body orientation now comes from the real quaternion, including small contact-induced rotations.

## Tests and corrections

Native tests were added before implementation and failed on the missing API. All eight engine integration tests now pass, including impulse timing/limits, fresh-reset parity and debug geometry extents. An initial test wrongly demanded an exactly identity quaternion after 600 contact ticks; actual solver contact produced rotations of roughly 1e-6. The corrected test checks initial XYZW ordering exactly, then unit norm and a bounded near-upright pose with physical tolerances.

Playback unit tests first failed on the missing controller. They verify pause/single-step, slow motion, command-slot handling, reset, preserved discarded-time evidence, and controlled-timestamp transitions. The counter regression failed with 0 instead of 29.9333 seconds before the fix. A separate desktop viewport test failed with the canvas bottom at 1110 px in a 900 px window; independent inspector scrolling fixes it.

Browser tests exercise the actual controls and WASM state, export the JSON file, and compare paused canvas images before/on/after the overlay. The on image must differ; the off image must exactly restore the original. The existing 601-state native/WASM trace and visible-loop tests remain in place.

## Remaining playground work

Recording/replay and editable versioned configuration are slice 03b. Spawn/remove, joint/contact/force overlays and live motor tuning arrive alongside the actual ragdoll/controller slices. Completing this fixture inspector alone does not complete all of specification §27 or the MVP.

## Final verification and review

Passed: 8 native integration tests, 8 TypeScript unit tests, 5 Chromium browser tests, strict TypeScript, Rust formatting/Clippy and release WASM/frontend builds. The build reports a >500 KB JS-chunk advisory (715.87 KB raw, 194.27 KB gzip); chunk budgeting remains part of the later performance slice. WASM is 1,342.57 KB raw / 445.89 KB gzip. These fixture sizes are not full-MVP budgets.

Environment: macOS 26.6.2 arm64, Rust 1.88.0, Node 20.18.0, Chromium 159.0.8154.6 via Playwright 1.65.1. Physics JSON SHA-256 remains `514d8064d10f7046f8445bd9ccad203e121dfd6f41129daaa522344740d534fc`; frontend lockfile is `5558760e8348559e26f53f795940a0ce2a4497b0c7855e46c97a926ed1261741`. React/runtime types were installed with zero npm audit findings.

Independent review found the discarded-time reset, pre-transition wall-time consumption, and insufficient visual coverage. All were corrected and reviewed again with no consequential new findings. Deliberately removing timestamp rebasing made the regression report four steps instead of one. Deliberately hiding the collider overlay made the canvas-image test fail. Sources were restored and the affected suites passed. Desktop screenshots and narrow-layout overflow checks were inspected.
