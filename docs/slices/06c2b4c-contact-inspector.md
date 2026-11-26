# Inspecting external contacts on a limb

Choose a single-fighter scene, select a body and turn on **Show external contact normals**. Cyan lines originate on the selected limb and point in the direction the surface pushes it. Each line is 0.20 m long so small impulses remain visible. Pause to inspect the same solve, select another limb or single-step. The inspector displays the loaded point count and the sum of their individual normal impulse magnitudes in N·s.

The native query reads the completed Rapier contact manifolds for the selected body’s primitive colliders. It includes points with positive normal impulse against external bodies or world geometry. The limb’s local witness is transformed into the displayed world pose. Pair ordering determines the sign of the world normal. Sensor intersections, internal rig contacts and unloaded manifold points contribute no impulse. Friction and joint motors are separate forces. Reading the buffer does not advance physics. The browser requests it only when the overlay is enabled.

## What happens on the stairs

On the default 60-tick settled approach, the right foot experiences 13.408 N·s of normal impulse against the second riser at tick 129. Its total external normal impulse vector, including contact with the tread, has a +Z component of 12.140 N·s. The fighter is walking toward −Z. Its right knee is commanded straight and the solved joint is flexed by 0.075 rad. The pelvis is already 0.264 m ahead of that foot in the walking direction. These are measurements of the failing controller and can change as locomotion improves.

## Verification

A native test propels a passive rig toward an isolated riser and verifies the impulse direction, witness position, loaded limb and unloaded opposite foot. A separate test checks upward sole loads on level ground and empty readings after reset. The rendering test checks world-space endpoints and removes old lines as points disappear. Chromium switches from a supported foot to the head while paused and resets the fixture. Native/WASM body-trace parity still passes across all 211 frames of the stair approach. Physical ascent remains open in the [stair traversal record](06c2b4b-stairs.md).
