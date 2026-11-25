# Slice 04b2: Visible ragdoll playground and inspection

Status: verified on November 24, 2026.

## Behavior and design

The Test scene selector switches between the cuboid fixture and passive ragdoll. A scene change disposes the old world/view and creates a fresh default-config scene. The ragdoll starts paused at its 2 m diagnostic drop offset, preserving an inspectable bind pose. Pause, single-step, slow motion, reset and physics tuning reuse the tested playback controller. Reset preserves pause/rate/selected body; tuning reconstructs the world paused. Neither action invents actuation or moves body poses in JavaScript.

`RagdollSimulation` adapts the shared native rig to the existing playback contract. Rig inspection is an explicit optional capability; recording methods are optional and absent from the ragdoll adapter. Cuboid recording controls remain available, while ragdoll recording awaits a character-input format. Scene generation and session revision jointly invalidate pending file reads when the runtime changes. This keeps future extensions separate from the proven cuboid recording format.

The solid rig uses geometry from `LocalRagdoll.definition_json()`, not a second hand-authored skeleton. Three.js capsules use twice Rapier's half-height for their straight segment. Rounded-box total dimensions include twice the Rapier border radius. Body transforms come from the owned 14-float records, directly as world XYZ/XYZW. These meshes are diagnostic collider geometry, not the final original skinned character.

Selecting a body highlights it, supplies its height/vertical velocity/mass readings, and selects the target for a queued impulse. Upward and forward (−Z) impulses are 4 N·s, captured for the selected limb at queue time and applied once at the next tick. While any impulse is pending, both impulse buttons are disabled. Total rig mass is calculated from actual body states. Joint count and maximum anchor separation come from actual joint buffers; selected joint limits come from the validated definition.

The joint overlay shows both world anchor points and both rotated local XYZ frames, plus an anchor-separation segment. Axis length is a 0.15 m presentation aid. X/Y/Z colors are accompanied by text labels. Collider lines remain Rapier-generated geometry. Overlay buffers are reused; hidden overlays do not request joint/collider data for rendering. A close fixed inspection camera frames the drop and landing; it is not the future production group camera.

## Resource ownership and failure handling

Each scene owns its RAF loop, WASM object, renderer/context, resize observer, geometries and materials. Normal disposal cancels RAF, disconnects the observer, disposes scene resources, explicitly loses the WebGL context, removes its canvas and frees WASM. Disposal is idempotent at the runtime boundary. Constructor failures clean up partially acquired scene resources. Initialization rollback covers the first render/snapshot/RAF setup, before the app has a runtime handle to dispose. A failed scene leaves the selector usable for a clean restart.

## Verification

- The visible-flow browser test initially timed out because the scene selector did not exist; it now verifies paused startup, counts/mass, body selection, visible joint/collider overlay changes, pending impulse controls, stepping, visual passive fall, reset, tuning and return to the cuboid with one canvas.
- Geometry tests first failed on the missing renderer module; they now verify capsule and rounded-foot outer dimensions independently.
- A browser resource test reproduced a surviving old GPU context after scene replacement; explicit context release fixes it.
- Review added injected constructor/first-render failures and complete cleanup assertions. Removing both rollback paths temporarily made both tests fail with unreleased resources; restoring them passes. Fault injection targets the actual loaded Vite module URL so it reaches the app instance rather than a separate uninitialized WASM module.
- A selected-left-hand forward impulse through the actual adapter/playback controller matches all 210 body-state values from a direct WASM step, guarding limb routing and direction.
- Visual inspection covered solid and collider/joint views at 1440×900, the mobile layout at 320 px and the fallen rig at tick 166 (head height 0.178 m). Ragdoll-specific checks at widths 320, 768, 1024 and 1440 found no horizontal overflow or page errors. Local ignored captures are `.cache/ragdoll-solid.png`, `ragdoll-overlays.png`, `ragdoll-mobile.png` and `ragdoll-fallen.png`. Screenshot capture emitted GPU `ReadPixels` stall warnings; these captures are not frame-time benchmarks. A capture affected by Vite's build-triggered reload was discarded and repeated after the build, explicitly checking the selected scene.

11 TypeScript tests, strict type checking, the production build and all 16 Chromium browser tests pass. The native/WASM implementation remains the verified 04b1 build (22 native tests); this slice changes presentation and its adapter only. Follow-up review confirmed the cleanup, copy and impulse-routing corrections with no consequential findings remaining. Production output is approximately 512.29 KB gzipped WASM and 199.74 KB gzipped JavaScript; the known raw chunk advisory and wasm-pack notices remain.

The new native/WASM trace evidence remains in [04b1](04b1-ragdoll-wasm-contract.md). No standing, locomotion, ragdoll input recording, eight-player stress, gameplay playability, or full MVP completion is claimed. Those remain explicit later requirements.

## Sources

Three.js **0.180.0** installed sources were used to verify `CapsuleGeometry` and `examples/jsm/geometries/RoundedBoxGeometry.js` dimension conventions, alongside the [official geometry documentation](https://threejs.org/docs/). The physical buffer/frame contracts are in [04b1](04b1-ragdoll-wasm-contract.md), and anatomy/frame conventions are in [ADR 0005](../decisions/0005-passive-character-rig.md).
