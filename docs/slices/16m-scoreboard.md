# Reading the scoreboard during a match

Tab from the arena focuses the visible scoreboard. The focused board has an outline and displays the Escape shortcut. Escape returns to the arena. Tab and Shift+Tab continue through the browser’s normal focus order. Local and private rounds both keep running. On a shared keyboard, leaving the arena releases both fighters’ controls. In a private room it releases the local fighter’s input.

The scoreboard is a named region immediately after the arena in keyboard order. It groups the round clock with fighter names, scores and contextual stamina. The private scoreboard also shows the round phase and which fighters are out. It uses the same authoritative game state during play and spectating. Keeping one scoreboard avoids duplicate announcements and preserves focus when scores change. [Scoreboard component](../../web/src/app/Scoreboard.tsx)

## Verification

The local, private-player and private-spectator focus checks failed on the preceding build. On the updated production build, Chromium, Firefox and Playwright WebKit all passed the local and private-player tests. The local test observed a running clock, returned to the canvas with Escape and exercised both directions of normal Tab navigation. The private test observed the outgoing movement axis and hand clear, then saw the same point on both clients when the other fighter ran off the roof. The scoreboard kept focus through the score change. [Browser tests](../../web/tests/scoreboard.spec.ts)

Chromium also passed both match-menu tests and both spectator promotion cases. During spectating, the newcomer read Blue’s score, returned to the arena, and sent zero gameplay inputs until promoted. The lineup change passed with both the current and older room handshakes. The focused local scoreboard, return hint and Pause button fit the 390 × 844 viewport. All 103 web unit tests, TypeScript checks and the production build passed. [Spectator tests](../../web/tests/online-late-join.spec.ts)
