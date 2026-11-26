# Skyline camera turns and fighter direction

With the arena canvas focused, X turns to the next 45-degree view and Z turns back. Holding a camera key selects one view until released. Each online player or spectator owns a camera. Two people playing on one keyboard share the view. A paused local game ignores camera controls.

Skyline starts at its previous rooftop angle. Each rendered frame moves toward the selected yaw along the shortest arc. The orbit uses exponential smoothing at 8 per second and caps the elapsed interval at 120 ms after a render stall. The heading used to fit the scene also projects forward and sideways movement onto the roof. Turning while holding a key updates local movement for the next solve and outgoing online movement at the next input sample. The protocol carries the same quantized world-space intent as before.

## Verification

The unit fixture checks all eight views, the return across the angle boundary, an opposite-direction turn, smooth intermediate directions and an independent camera. It compares movement against the actual Three.js camera direction. The local Chromium fixture observes the ordinary WASM input calls for both fighters while they hold forward through a shared turn. The online fixture reads both browsers’ outgoing movement records and checks that a turn by the host leaves the guest’s forward direction unchanged.

Run `npm --prefix web test -- --run src/rendering/GroupCamera.test.ts` and `npm --prefix web run test:e2e -- skyline-orbit.spec.ts`. Camera collision, visibility through obstructions and human readability remain open.

All eight Chromium checks in the orbit, local grip and Skyline match suites passed together, including the existing crate grab, ledge release and rematch. All 44 web unit tests and the production build passed. The spectator unit check also verifies that camera keys send no fighter packet and the listener is removed on disposal.
