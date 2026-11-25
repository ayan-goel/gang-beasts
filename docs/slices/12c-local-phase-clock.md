# Local Skyline phase timing

Local matches count down for 180 ticks and show round results for 300 ticks. These phases have no physics solve. Previously the browser applied the four-step catch-up limit to them. At roughly four visible frames per second, the native countdown still had 84 of 180 ticks left after the 6.5-second browser test deadline. An ordinary local ring-out test also intermittently exceeded its 18-second deadline to see round two.

The browser now allows elapsed fixed ticks up to the larger of four and the current waiting phase’s remaining ticks. The phase boundary limits the extra work. A visible frame can advance the inexpensive native clock without running a backlog of fighter physics. The native engine still owns the exact phase transition, score and round rebuild. A hidden local Skyline page does not advance its match. Visibility changes and resuming from pause reset elapsed time and fractional debt.

## Verification

The clock tests advance a full 180-tick countdown in twelve 250 ms frames. They cover a ten-second delay immediately before a phase change, a clean next frame and the ordinary four-step limit. The Chromium regression uses a real keyboard ring-out while limiting its animation callbacks to four per second. It delivers callbacks while the test page reports itself hidden for longer than the entire countdown. After visibility returns, it checks that the match starts and proceeds to round two with the survivor’s point.

Run `npm --prefix web test -- --run src/game/FixedClock.test.ts` and `npm --prefix web run test:e2e -- skyline-clock.spec.ts skyline.spec.ts`.

Both clock regressions, the slow-render browser scenario and all four local Skyline browser tests passed. The full workspace browser run also passed these five local scenarios. It reported separate online startup failures under investigation.
