# Skyline compatibility before ready

Pressing **Ready up** loads the browser build of the shared engine. The WASM module validates and returns its bundled Skyline map and character definition without creating a simulation. The client retains those assets for rendering. Browsing rooms alone does not download the engine.

Each socket starts unverified. After loading, the browser sends its own engine and map fingerprints. The server compares both to the native release and acknowledges success before the client sends SetReady. A mismatch clears readiness. Replacing a socket requires a fresh acknowledgement. A failed download leaves the seat available to retry, and a 12-second client deadline returns control after a stalled load. A hash match establishes build compatibility. The server still owns gameplay.

The test in `web/tests/room-loading.spec.ts` first failed when an interrupted asset request still allowed readiness. Chromium now verifies retrying a failed WASM download and completing readiness. A second case sends premature readiness and different content fingerprints over a real member socket, and checks the server’s rejection for both. Native tests check each digest independently, both header fields, truncation and trailing data. Existing two-browser ready and connection-replacement cases pass.

## Loading the online renderer

The Ready action also imports the online match view, Three.js and its styles. The renderer module and WASM content load in parallel. The browser waits for both before submitting its fingerprints. Viewing the lobby still defers the game downloads. The handshake retains the imported component on the current socket. On assignment the browser mounts that component directly and creates the WebGL scene for its assigned roster. The reconnect check verifies the server reassigned the original fighter before independently checking its visible arena.

A browser test holds the arena module request open while the host presses Ready. The roster must continue to show Waiting until the request completes. Previously the browser could become ready without even requesting the renderer.

In the multiplayer regression run, eight tests passed together across loading, eight-player admission, reconnect, stream recovery and two rematches. The eight-player production test passed three consecutive times in 57.5 seconds. It runs from the eighth spawn toward the clear south edge. Its former eastward path crossed a fixed vent and could leave the fighter on the rooftop. Each test now releases its authenticated seats during teardown, including after a failed assertion.

After mounting the prepared renderer directly, all 74 Chromium tests passed together in 12.4 minutes with the default single worker. This included the server assignment and visible arena checks on reconnect. All 42 web unit tests, type checking and the production build also passed.
