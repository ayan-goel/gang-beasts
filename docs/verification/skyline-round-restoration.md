# Skyline round restoration

The private-room scenario follows the host and guest through a complete hazard cycle. It reloads the guest after the west walkway detaches. During the crosswind, both clients inspect the nine named meshes in their active Three.js scene. The two exterior walkways should be hidden. In the next countdown, all nine objects must be visible at their authored centers with identity rotation and no warning glow. The comparison rounds position in metres and quaternion components to two decimal places. Both players must carry their scores into the new round and return to play.

The test listens to Three.js scene creation in the browser. It reads the same objects updated by the normal online renderer, including in the guest’s replacement document. The server runs its ordinary round clock. [Browser scenario](../../web/tests/online-hazards.spec.ts) · [Scene observer](../../web/tests/support/arenaMeshes.ts) · [Authored map](../../assets/maps/skyline/map.json)

Three complementary checks cover the physical behavior.

- The [native arena fixtures](../slices/11a-skyline-physics.md) exercise crate impulses, fixed vents, detachment and the pelvis crossing the fall plane. They compare rebuilt prop state with the initial state.
- The [online crate test](../../web/tests/online-grips.spec.ts) moves a gripped crate using normal player controls and checks its release. The [eight-player match](../slices/15f-online-roster.md#verification) exercises player falls, replicated elimination and scoring.
- The [eight-fighter endurance run](reset-endurance.md) verifies body and prop restoration across 100 ordinary round transitions, including rounds with both walkways detached.

## Result

Two timed runs passed on an Apple M3 Pro with macOS 26.6.2, Chromium and Metal. In each run, the host and reconnected guest saw both walkways disappear, all nine meshes return to the authored state and both fighters enter round two. In the second run both clients also observed the west walkway alone emitting its warning color before detachment. Each run took 3.3 minutes over loopback. This completes the §33 arena mechanics criterion.

The production bundle was byte-identical to a fresh release build. The native and WASM engine fingerprint was `58f614835f343e435536a7dbf678ac3c51b4ea387d9057ab3eccff3926339f4b`. The map SHA-256 was `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`. The separate online crate test, six native arena checks, 76 web unit tests and TypeScript checks passed.

To repeat the browser checks against a local server, run `npm --prefix web run test:e2e -- online-hazards.spec.ts online-grips.spec.ts`.
