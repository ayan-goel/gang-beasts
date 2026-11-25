# Skyline compatibility before ready

Pressing **Ready up** loads the browser build of the shared engine. The WASM module validates and returns its bundled Skyline map and character definition without creating a simulation. The client retains those assets for rendering. Browsing rooms alone does not download the engine.

Each socket starts unverified. After loading, the browser sends its own engine and map fingerprints. The server compares both to the native release and acknowledges success before the client sends SetReady. A mismatch clears readiness. Replacing a socket requires a fresh acknowledgement. A failed download leaves the seat available to retry, and a 12-second client deadline returns control after a stalled load. A hash match establishes build compatibility. The server still owns gameplay.

The test in `web/tests/room-loading.spec.ts` first failed when an interrupted asset request still allowed readiness. Chromium now verifies retrying a failed WASM download and completing readiness. A second case sends premature readiness and different content fingerprints over a real member socket, and checks the server’s rejection for both. Native tests check each digest independently, both header fields, truncation and trailing data. Existing two-browser ready and connection-replacement cases pass.
