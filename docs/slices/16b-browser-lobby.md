# Private room lobby

Players enter from **Private rooms** on the Skyline match screen or follow `/rooms/?code=ABCDEF`. Creating a room chooses its capacity, player name and optional color. The invitation contains only the room code. The roster displays names, colors, the host and the viewer's seat. Copying uses the browser clipboard, with a selectable link as a fallback.

## Session and roster behavior

The server returns the current seat’s room code during session bootstrap. A refresh or a second tab in the same profile returns to that room. If an invitation points elsewhere, the page preserves its URL and shows the current roster until the player explicitly leaves. It then opens the prefilled join form. The server derives the acting player from the session cookie.

The [live room connection](16c-lobby-ready.md) delivers the current roster. A disconnected page falls back to a member-only HTTP fetch every six seconds after the previous request finishes. The page cancels an outstanding fetch and polling on unmount. Public room metadata omits names. Leaving releases the seat immediately and transfers the host role to the lowest occupied slot. The browser retrieves the new role on its next room-state update. No simulation worker is allocated by these actions.

## Verification

Run `npm --prefix web run test:e2e -- private-lobby.spec.ts`. Playwright starts the native service on 5181 and Vite on 5180. The tests cover independent host and guest sessions, an invite, roster updates, reload, voluntary host transfer, switching rooms with a pending invitation and missing-room feedback at a 320-pixel viewport. Rust router tests independently check that possession of a code alone cannot disclose player names or restore a departed seat through a create retry.

The §33 friend-joining check passed on loopback in Playwright Chromium 159.0.8154.6 with the native release server and Skyline map version 1. Run `npm --prefix web run test:e2e -- lobby-capacity.spec.ts`. Nine independent cookie jars followed the public UI. Each of the first eight players saw their own seat, the ninth saw “This room is full,” and the host retained the original eight-player roster. This verifies admission on one machine. Public deployment and eight simultaneous fighters have separate acceptance checks.

The room page has its own entry bundle. Its Skyline image comes from the repository’s rendered map. A lobby supports two to eight seats. Ready controls use the live connection. Host start and remote fighter controls remain open.
