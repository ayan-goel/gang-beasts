# Skyline spectator camera

A private-room spectator starts in group view. Previous fighter and Next fighter cycle through surviving fighters in roster order, wrapping at either end. Group view restores the shared framing. The scoreboard remains visible. Eliminated players get the same controls while they wait for the next round.

A selection belongs to one round. The browser resolves it against the same interpolated snapshot used to draw the fighters. If the selected fighter falls, the next surviving fighter becomes the target. A knocked-out or hanging fighter remains eligible. With no survivors, the camera frames the roof. A lineup change or respawn clears the selection. [Selection rules](../../web/src/online/spectatorCamera.ts)

Following changes which bodies contribute to the camera bounds. Every surviving fighter still renders, and the selected fighter uses the existing full-limb bounds, smoothing and orbit. Z and X work with focus on the canvas or the camera buttons. Movement input remains scoped to the fighter’s canvas. [Renderer](../../web/src/rendering/FixtureScene.ts) · [Controls](../../web/src/online/SpectatorControls.tsx)

## Verification

- Two private-room scenarios observed the actual Three.js camera during draws. A waiting spectator followed Mint, returned to the group, followed Blue, orbited with a camera button focused, and automatically followed Mint when Blue fell. Both supported lineup handshakes promoted the spectator in round two. The spectator sent zero gameplay inputs while waiting and subsequently moved their own fighter. A 390 × 800 viewport repeat showed the selected fighter, scoreboard and all three controls. [Late join](../../web/tests/online-late-join.spec.ts) · [Camera measurement](../../web/tests/support/cameraAim.ts)
- In an eight-player match, the eliminated eighth player followed fighter two. When fighter two fell, the camera selection moved to fighter three. That client’s gameplay input count stayed fixed through the handoff. Its canvas regained keyboard focus at round two. An active player’s private orbit continued to affect only their own movement. [Eight-player match](../../web/tests/online-roster.spec.ts) · [Orbit isolation](../../web/tests/skyline-orbit.spec.ts)
- Unit tests cover wraparound in both directions, no survivors, active-player exclusion, eliminated-player access, stale round selections and match results. The release build, TypeScript check and 81 web unit tests passed. [Selection tests](../../web/src/online/spectatorCamera.test.ts)

Tested on macOS 26.6.2 with Apple M3 Pro, Chromium, Metal and a release server over loopback. The packaged online match chunk was `OnlineMatch-C5WcyWTo.js`. The Vite preview proxy logged a connection reset during the portrait run. The current-handshake clients each observed one game socket through promotion. Camera collision and human readability assessment are still open under §33.

Run `npm --prefix web run test:e2e -- online-late-join.spec.ts online-roster.spec.ts skyline-orbit.spec.ts` to repeat the browser checks.
