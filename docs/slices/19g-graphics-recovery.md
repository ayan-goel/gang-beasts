# Graphics recovery during play

A WebGL interruption replaces the blank arena with a message and a reload action. When the browser restores the context, Three.js rebuilds its GPU state. The same canvas draws the current scene with the selected graphics preset and viewport size. A player who had canvas focus regains it unless they moved focus to another control during the interruption.

| Where graphics stopped | What happens during the interruption | If the player reloads |
| --- | --- | --- |
| Local Skyline, sparring or ledge practice | The fixed clock stops and held input clears. Restoration resumes the prior playback state and discards elapsed wall time. | The local session starts over. |
| Physics playground | The simulation and fixture actions are suspended. A paused fixture keeps its exact tick. Switching scenes releases the interruption notice. | The playground starts a new fixture. |
| Private Skyline | Loss of canvas focus immediately releases held input. The server continues running. The scoreboard stays live while drawing is interrupted. | The existing room reconnect flow attempts to recover the seat. |

If WebGL 2 cannot start, local and private Skyline explain the graphics requirement and offer **Retry graphics**. A private guest can watch the scoreboard and retry within the same connection. A failed local physics download can be retried on the current page. Browsers without WebAssembly get a compatibility message before loading the simulation or claiming readiness in a room. [Graphics handling](../../web/src/rendering/GraphicsRecovery.ts) · [Local startup](../../web/src/app/mountSimulation.tsx)

## Browser checks

The tests use the browser’s `WEBGL_lose_context` extension. Local recovery checks the suspended round clock, resumed clock and rendered pixels. Private recovery checks that a held movement axis and hand return to neutral, both clocks keep moving, the original socket stays connected and fresh input reaches the server. A separate case uses the reload fallback to rejoin the ongoing round. The playground case restores a paused simulation, advances one tick and switches scenes during a second interruption. [Context-loss tests](../../web/tests/graphics-recovery.spec.ts)

Startup checks withhold WebGL 2 until retry, interrupt the WASM download once, and remove WebAssembly before page initialization. The private-room cases verify the graphics retry on the same connection and eligibility messaging before ready-up. [Startup tests](../../web/tests/graphics-startup.spec.ts)

## macOS verification

All nine context-loss and startup scenarios passed against the production bundle on an Apple M3 Pro running macOS 26.6.2. Chromium 159.0.8154.6 used HTTP loopback. Firefox 159.0 and Playwright WebKit 27.2 used HTTPS loopback. Firefox and WebKit also passed the three entry checks covering direct URLs, menu navigation to local play and practice, and private host/join by code.

| Browser | Menu encoded body bytes | Additional bytes to first playable arena | Total encoded body bytes |
| --- | ---: | ---: | ---: |
| Firefox 159.0 | 60,495 | 2,591,924 | 2,652,419 |
| Playwright WebKit 27.2 | 60,529 | 2,592,276 | 2,652,805 |

Both browser traces include the physics WASM and fighter model. These measurements cover local delivery of the production bundle. Hosted distribution still needs its own measurement. Retail Safari declined the WebDriver session because remote automation was disabled, so its §33 entry remains open. [Per-resource measurements](../verification/browser-entry-graphics.json)

## Local HTTPS reproduction

WebKit on macOS did not send the secure host session cookie when the preview used plain HTTP on 127.0.0.1. The private-room checks pass on HTTPS loopback with the existing cookie attributes. To exercise HTTPS, set `BRAWLER_TLS_CERT` and `BRAWLER_TLS_KEY` to absolute paths to a local certificate and its private key. Vite accepts the pair for development and preview. The Playwright config uses HTTPS for the browser and the matching server origin when the pair is present.

From the repository root, the following example creates a one-day test certificate outside the workspace. Playwright accepts it for this test context. The native room worker listens on the usual loopback HTTP endpoint behind Vite.

~~~sh
test_cert_dir=$(mktemp -d)
openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost,IP:127.0.0.1' \
  -keyout "$test_cert_dir/key.pem" -out "$test_cert_dir/cert.pem"
export BRAWLER_TLS_CERT="$test_cert_dir/cert.pem"
export BRAWLER_TLS_KEY="$test_cert_dir/key.pem"
cd web
npx playwright install chromium firefox webkit
npm run test:e2e -- --browser=webkit graphics-recovery.spec.ts graphics-startup.spec.ts
~~~

Use `--browser=chromium` or `--browser=firefox` for the other engines. Run the suites sequentially when sharing a single room worker. For interactive private play over local HTTPS, configure the native server’s `BRAWLER_ORIGIN` to the browser’s HTTPS origin and use a certificate trusted by that browser.
