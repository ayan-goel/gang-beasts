# Main menu and browser entry

The root opens Skyline Melee’s main menu. Hosting takes the player to the create form. Joining opens the code field directly. The local match and the physics playground each have a separate entry. A bookmark or invite with a query string retains that query when the development or preview server adds a trailing slash.

| Destination | Public path |
| --- | --- |
| Main menu | `/` |
| Host a private room | `/rooms/` |
| Join by code | `/rooms/?action=join` |
| Open an invite | `/rooms/?code=<room code>` |
| Two-player local match | `/play/` |
| Physics playground | `/playground/` |

The menu uses HTML links, a stylesheet and one WebP image. It loads without JavaScript, WebGL, WASM or a room API session. Choosing a game loads its runtime. The local game and playground share WASM startup and error handling through separate application entry points. A local match works independently of the room service. [Menu](../../web/index.html) · [Simulation startup](../../web/src/app/mountSimulation.tsx)

The menu image comes from the original Skyline throw fixture. Coral ran to the east roof edge, gripped Blue and released the lift. The capture uses both skinned poses ten simulation ticks after release, seen from outside the walkway. The image is 1152 × 880 and occupies 58,680 bytes. [Throw fixture](../../web/tests/skyline-throw.spec.ts) · [Asset inventory](../asset-provenance.md)

## Verification

On an Apple M3 Pro running macOS 26.6.2, fresh Chromium 159.0.8154.6 and Google Chrome 158.0.8127.37 contexts produced the same cold-load measurements over loopback. Values sum same-origin navigation and resource `encodedBodySize` through the first local round reaching Playing.

| Download | Encoded body bytes |
| --- | ---: |
| Main menu HTML, CSS and still | 60,495 |
| Local match after leaving the menu | 2,581,183 |
| Complete path from cold entry to playable local round | 2,641,678 |

The menu rendered at widths of 1440, 390 and 320 pixels with JavaScript disabled. The first Tab focused Host a room, Local match stayed in view and the page fit the viewport. Both browser builds reached a local match, opened the diagnostic scene and joined two independent seats by code. The Chromium live-preview test also readied both seats and observed Playing on both clients. [Browser regressions](../../web/tests/main-menu.spec.ts)

All 81 web unit tests, TypeScript checks and the production build passed. The development run covering the migrated playground tests passed 57 of 58 selected scenarios. The additional online knockout scenario observed zero impact sounds once. It passed on an isolated repeat in the same configuration and three consecutive packaged runs with Metal. The cause of the missed cue remains unverified.

The measured local path is within the §28 download target. Hosted transfer and the Firefox and Safari release checks remain open under §33.
