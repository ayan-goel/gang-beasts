# Sudden-death crosswind

Players hear a continuous rush while the Skyline crosswind pushes their fighters. Local play follows the current WASM hazard and phase. Private play uses the hazard and phase from the server’s sampled arena state. The effects mixer rebuilds the loop when play resumes during a gust, including after a fresh connection and audio unlock. The existing transition cue announces the onset once. The visual warning and the physical force continue when effects are muted.

## Playback

Only an active, audible match in the gust stage requests wind. The same voice continues across ordinary snapshots. Leaving play, pausing locally, losing live online presentation, hiding the tab, suspending audio or turning off effects stops the loop. A fresh round starts quiet. The listener hears the crosswind across the arena, independently of camera distance. Music and effects keep their independent gain controls. [Match state and hook](../../web/src/audio/SkylineSound.tsx) · [Output lifecycle](../../web/src/audio/SoundOutput.ts) · [Effects bus](../../web/src/audio/MatchMix.ts)

## Original sound

The four-second mono source uses deterministic filtered noise, with a low rumble and two slow variations in level. It is generated at 16 kHz on first use and cached once per audio context. A quarter-second crossfade between continuing filtered noise and the beginning of the source preserves energy at the loop seam. Entry and exit have gain envelopes to smooth transitions. The generated buffer uses 256,000 bytes of float samples. [Synthesis](../../web/src/audio/crosswindSound.ts)

## Verification

Run `npm --prefix web run test:e2e -- crosswind-sound.spec.ts mix-render.spec.ts` against the development server for offline audio rendering. The fixture checks the onset, the first loop seam, both output rates, separate effects gain and silence at zero gain or mute. The local and private [timed gameplay](../../web/tests/online-hazards.spec.ts) checks use the authored Skyline schedule. [Local pause and resume](../../web/tests/skyline-audio.spec.ts)

The offline graph rendered five seconds in Chromium 159, Playwright Firefox and Playwright WebKit on the Apple M3 Pro. Each browser crossed the four-second seam at both 44.1 and 48 kHz. Peak sample magnitude ranged from 0.05948 to 0.05963 at full master/effects gain. RMS over 400 samples centered on the seam ranged from 0.01194 to 0.01251. Halving master volume and setting effects to 30% produced 15% of the full peak with music at 100%. Mute and zero effects produced digital silence. Perceived balance on speakers and headphones remains a listening check.

Two packaged Chromium scenarios used the authored three-minute lead-in. Both local players shared one wind voice, pausing stopped it, and resuming restored it. In a private match both browsers started one voice at the gust. The guest reloaded mid-gust and recovered the loop after a keyboard interaction without replaying the onset warning. The loop stopped at round results on both clients and the next round began normally. All 123 web unit tests, TypeScript checks and the production build passed. The simulation fingerprint for these matches was `dc30b858e7d1bb693322b0eca058dfe0eeb99b6958dd77045b2fbe7bc9069df5`.
