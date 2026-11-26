# Skyline hazard warnings

The online arena shows the current walkway or wind warning in the lower-left corner of the play area. Both the text and the panel highlight follow the interpolated server snapshot and the schedule in the verified Skyline map. Reconnecting during a warning restores its current state on the next snapshot. Countdown, round results and match results clear the text.

| Arena tick | Elapsed at 60 Hz | Message |
| --- | --- | --- |
| 0–5,399 | Before 90 s | No warning |
| 5,400 | 90 s | West walkway failing |
| 7,200 | 120 s | East walkway failing |
| 9,000 | 150 s | High winds approaching |
| 10,800 | 180 s | Sudden death · Crosswind |

The local and online views share the warning text. The online warning uses a persistent polite status region and allows pointer input through to the arena. The [hazard audio slice](17c-hazard-audio.md) adds a distinct sound for each transition.

## Verification

Run `npm --prefix web run test:e2e -- online-hazards.spec.ts` to play through the actual timed schedule in two independent Chromium profiles. The test reloads the guest during the east warning and checks that both fighters receive all four warnings and clear the message at round results. The round runs at the server’s normal 60 Hz.

The test first failed on the absent online status region. With the warning present, both profiles observed all four messages, the guest recovered the east warning on reload, and both status regions cleared at results. The real-time run passed in 3.2 minutes on loopback.
