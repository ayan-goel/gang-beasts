# Recovering a stalled game download

A player can recover when the browser’s request for the physics engine or fighter model stops making progress. A private room keeps the same seat and invite. Local play and the physics playground stay in the same document.

## Private rooms

The existing 12-second readiness deadline releases the waiting player back to the lobby controls. Previously, an unfinished fighter request could remain cached as a pending promise. Every later attempt waited on that promise. A retry of an unfinished physics request could also start a second transfer while the first was still active.

The readiness attempt now owns the cancellation signal used by both fetches. Timeout, leaving the room or socket teardown cancels the work. A retry can reuse successfully loaded assets and request missing assets again. The client still waits for the model, engine, prepared graphics and server fingerprint acknowledgement before it shows the seat as ready. [Readiness handshake](../../web/src/lobby/ContentHandshake.ts) · [Private-room regression](../../web/tests/room-asset-retry.spec.ts)

## Local play and practice

After 12 seconds the page offers a retry and keeps the current transfer alive. Choosing retry cancels the old attempt immediately and starts a new one. A completed download removes the prompt without player input. A reported network failure offers the existing retry immediately. The local game applies this behavior to its fighter model and physics binary. The playground uses the same physics loader. [Local regressions](../../web/tests/local-asset-retry.spec.ts)

The browser compiles WASM while streaming when the response has the WebAssembly media type and the browser supports streaming compilation. A generic binary response uses buffered compilation. The client checks cancellation after compilation, before initializing the shared WASM instance. The fighter loader parses the self-contained GLB from a cancellable fetch and caches the successfully parsed model. Ledge practice uses its scene cancellation signal when requesting the model. [Physics loader](../../web/src/game/loadPhysics.ts) · [Fighter loader](../../web/src/rendering/fighter/loadFighterModel.ts)

## Verification

The browser tests hold the first asset request open. In a private room the old request must receive an abort before the player retries. The second request must succeed while the test still holds the first, and the room must show the same invite and named seat. In local play the test reaches a running round with one canvas and one document navigation. In the slow-success case the user waits, the original download completes, and the selected stairs scene advances a physics tick.

~~~sh
npm --prefix web run test:e2e -- local-asset-retry.spec.ts room-asset-retry.spec.ts graphics-startup.spec.ts
~~~

The final packaged build passed 23 Chromium scenarios and 10 each in Firefox and WebKit. Chromium included unavailable WebGL, genuine graphics context loss, a resumed private match, model rendering and GPU preparation. Four development-server checks, all 123 web unit tests and the TypeScript/production build also passed. The native and WASM engine fingerprint was `dc30b858e7d1bb693322b0eca058dfe0eeb99b6958dd77045b2fbe7bc9069df5`.

Three fresh browser profiles opened the static menu and started a playable local round on the Apple M3 Pro. Resource Timing reported the following same-origin encoded body bytes. Firefox and WebKit used HTTPS loopback. Retail Safari and hosted transfer remain open.

| Browser | Menu | Local arena | Total |
| --- | ---: | ---: | ---: |
| Chromium 159 | 61,873 | 2,606,282 | 2,668,155 |
| Firefox 159 | 61,873 | 2,607,197 | 2,669,070 |
| Playwright WebKit 27.2 | 61,924 | 2,606,669 | 2,668,593 |

With JavaScript disabled, the native keyboard shortcut reached the Host a room link in all three browsers. The macOS WebKit configuration used Option-Tab for links. Firefox and Chromium used Tab.
