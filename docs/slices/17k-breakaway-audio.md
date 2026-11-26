# Skyline walkway breakaway sound

The west and east walkways make a short metallic crack when the roof connection fails. Each sound comes from the attached edge of the panel. A listener can hear the side giving way while the centered warning announces the next hazard. The score and effects use the existing independent mixer channels.

## Trigger and position

Local play reads the completed WASM arena tick. Private play reads the sampled server arena tick. Both compare that tick with the west and east deadlines in the same verified Skyline manifest. The sound requires the corresponding advance from west warning to east warning, or from east warning to gust warning, within twelve physics ticks of detachment. A duplicate or older stage, a skipped stage, a new round or a gap over 500 ms cannot replay an old break. Joining a match or returning from pause establishes the current stage silently.

The attachment point is the center of the panel's roof-facing upper edge. In the authored map the west and east points are (−10, 0, −2) m and (10, 0, 2) m. Both game modes project that point through their current camera, using the same stereo pan and distance attenuation as fighter impacts. A panel that subsequently falls or retires has already used its cue for the round. [Source and synthesis](../../web/src/audio/breakawaySound.ts) · [Transition checks](../../web/src/audio/breakawayCues.test.ts)

## Original sound

A deterministic noise transient supplies the initial crack. Three independently decaying modes at 218, 577 and 1,310 Hz supply the metallic ring. The mono source has a 2 ms attack and ends at 320 ms. Each audio context owns one cached buffer. One breakaway voice can play alongside the hazard warning. Breakaways use the effects bus, including its mute and level controls.

Chromium, Firefox and WebKit rendered the source at 44.1 and 48 kHz. At 8 m the measured peak sample amplitudes were 0.07743 and 0.07554. At 24 m the peaks were one third as large. Full left and right pan kept the opposite channel below 1e−7, and the buffers were silent after 321 ms. These are digital measurements before master gain. Listening on speakers and headphones remains a human acceptance check. [Offline stereo verification](../../web/tests/breakaway-sound.spec.ts)

## Reproduce

~~~sh
npm --prefix web test -- --run src/audio/breakawayCues.test.ts
npm --prefix web run test:e2e -- breakaway-sound.spec.ts online-hazards.spec.ts skyline-audio.spec.ts
~~~

The local run starts a match, pauses and resumes during the first warning, and observes both panel breaks. The private run follows two browsers through the full hazard schedule and a round reset. The guest reloads after the west break so the recovered session must stay quiet until the east break.

On the M3 Pro, both production Chromium scenarios passed on their ordinary clocks. Both breaks produced distinct opposite-side stereo positions. The guest heard the west break, reloaded without replaying it, then heard the east break. The menu, local game, practice route and friend joining passed through the separate playable preview. The web suite passed 123 unit tests, type checking and the production build. The engine fingerprint was `e484e2afe166c75f98e8c203bf495da5120661d1da6f55d0886360b710a3ef79`.
