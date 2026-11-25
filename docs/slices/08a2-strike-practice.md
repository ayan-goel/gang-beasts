# Strike practice

The practice scene starts paused with the fighter facing a suspended pad. Resume and use either mouse button, J/K or the hand buttons to strike. The inspector reports horizontal displacement relative to the point directly below the pivot. A close, oblique camera shows the fist, the pad and its suspension together.

## Physics and rendering

The pad is a 2 kg dynamic cuboid with half extents of 0.42 × 0.17 × 0.055 m. Its center starts at (0, 1.12, −0.54) m. A spherical joint connects a point 0.95 m above the pad’s center to a fixed body. Off-center contact can impart rotation as well as forward momentum. Gravity and damping govern the subsequent swing. The padded surface and suspension take their transforms from the running simulation.

The fixture owns 17 rigid bodies, 17 colliders and 15 joints, including the fighter and platform. Only the fighter’s 14 anatomical joints receive character motor targets. Reset rebuilds all handles and solver caches. Applying physics settings also preserves the selected scenario.

The native and WASM practice state contains 13 floats in order. Center XYZ and quaternion XYZW describe the pad. They are followed by collider half extents XYZ and the fixed pivot XYZ. The WASM array is an owned copy. The renderer derives the tether endpoints from the pad’s local top and the pivot and uses the same geometry dimensions at each reset.

## Evidence

- In the native control, the pad spends 120 ticks at rest. A hand input alone leaves its transform unchanged. Advancing the simulation after either tap moves the pad forward by more than 8 cm and keeps its center within 2 cm of the specified suspension radius. Reset restores the exact initial record and world counts.
- The native/browser comparison starts each hand test from the same resting state and taps one hand on tick 61 in each independent run. It compares all 13 fields in 151 consecutive records per side with a maximum absolute tolerance of 0.00001. Reconfiguring the browser fixture reproduces the left-hand trace exactly.
- In that native script, the pad reached 0.173 m forward displacement after the left punch and 0.166 m after the right. These figures describe the specified fixture and input timing.
- The Chromium UI test selects the scene, punches using the keyboard, observes the pad displacement, captures the rendered viewport, resets and reapplies tuning.

Run the native test with `cargo test --release --locked -p brawler-engine --test strike_practice`. Generate the reference traces with `cargo run --release --locked -p brawler-engine --example practice_trace`. Build WASM and run `npm --prefix web run test:e2e -- strike-practice.spec.ts` for the browser comparisons.

The November 25, 2026 checkout passed 41 Chromium checks, 19 web unit tests, the production build, Clippy and formatting. The native release workspace passed 188 checks and reproduced the two existing stair-ascent failures. The final captured practice viewport shows the fighter, marked pad, pivot and tether together.
