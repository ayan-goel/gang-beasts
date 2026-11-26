# Skyline compatibility before ready

Pressing **Ready up** loads the browser build of the shared engine. The WASM module validates and returns its bundled Skyline map and character definition without creating a simulation. The client retains those assets for rendering. Browsing rooms alone does not download the engine.

Each socket starts unverified. After loading, the browser sends its own engine and map fingerprints. The server compares both to the native release and acknowledges success before the client sends SetReady. A mismatch clears readiness. Replacing a socket requires a fresh acknowledgement. A failed download leaves the seat available to retry, and a 12-second client deadline returns control after a stalled load. A hash match establishes build compatibility. The server still owns gameplay.

The test in `web/tests/room-loading.spec.ts` first failed when an interrupted asset request still allowed readiness. Chromium now verifies retrying a failed WASM download and completing readiness. A second case sends premature readiness and different content fingerprints over a real member socket, and checks the server’s rejection for both. Native tests check each digest independently, both header fields, truncation and trailing data. Existing two-browser ready and connection-replacement cases pass.

## Loading the online renderer

The Ready action also imports the online match view, Three.js and its styles. The renderer module and WASM content load in parallel. The browser waits for both before submitting its fingerprints. Viewing the lobby still defers the game downloads. The handshake retains the imported component on the current socket. On assignment the browser mounts that component directly and creates the WebGL scene for its assigned roster. The reconnect check verifies the server reassigned the original fighter before independently checking its visible arena.

A browser test holds the arena module request open while the host presses Ready. The roster must continue to show Waiting until the request completes. Previously the browser could become ready without even requesting the renderer.

In the multiplayer regression run, eight tests passed together across loading, eight-player admission, reconnect, stream recovery and two rematches. The eight-player production test passed three consecutive times in 57.5 seconds. It runs from the eighth spawn toward the clear south edge. Its former eastward path crossed a fixed vent and could leave the fighter on the rooftop. Each test now releases its authenticated seats during teardown, including after a failed assertion.

After mounting the prepared renderer directly, all 74 Chromium tests passed together in 12.4 minutes with the default single worker. This included the server assignment and visible arena checks on reconnect. All 42 web unit tests, type checking and the production build also passed.

## Preparing the first draw

With two default headless Chromium clients on the M3 Pro, the first arena frames could stall for more than 800 ms. Both clients received the server snapshots, but a browser could reach its first live audio observation after the countdown had already changed from three to two. The sound cursor deliberately discards transitions after a long observation gap. The [presentation clock correction](15i-snapshot-clock.md#recovering-from-queued-publications) fixes backlog recovery. Preparing the GPU work before the host starts the match addresses the initial drawing stall.

On an explicit Ready, the lobby creates one detached Skyline scene with the map, lights, shadows and an original skinned fighter. It draws to a small canvas and polls a WebGL 2 fence until that draw completes. The room's existing loading deadline covers the preparation. At assignment the first fighter takes its actual kit, the canvas moves into the match viewport, and the remaining fighters join the scene. The kit update repaints that fighter's existing suit texture and hidden silhouette while retaining the compiled materials. [Preparation](../../web/src/online/preparedArena.ts) · [Content handshake](../../web/src/lobby/ContentHandshake.ts)

A canceled ready releases the staged scene and its graphics context. Leaving during the pending draw aborts the wait and releases the context. Once the canvas has moved into the match, the match owns its disposal. Reconnecting to a running round bypasses lobby drawing. A device that cannot initialize WebGL can still enter, follow the live scoreboard and use Retry graphics.

The [browser regression](../../web/tests/room-render-ready.spec.ts) captures the actual drawing context in the lobby and verifies that the same canvas draws the match. It also covers cancel, re-ready and leaving with an unfinished GPU fence. The [fighter skin check](../../web/src/rendering/fighter/FighterSkin.test.ts) verifies suit markings, silhouette color and texture reuse for a different assigned kit.

On the M3 Pro, the default Chromium renderer passed the landing, countdown, cancel/re-ready and pending-draw cleanup scenarios in two consecutive production runs. Both clients emitted a landing voice for the same server jump and both emitted the countdown and scored round cues. WebKit passed those four cases over HTTPS. Separate production Chromium runs covered two complete rematches, all eight fighters starting and agreeing on a point, and retrying an unavailable GPU during live play. The development build passed the content-loading and graphics-loss checks. All 110 web unit tests, TypeScript checks and the production build passed.
