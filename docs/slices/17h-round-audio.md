# Round audio in Skyline

Local and private matches sound the countdown changes and a higher start tone when movement becomes available. A scored round has two rising pulses. A draw has two falling pulses. The final match result has three rising pulses. The visual countdown, score and winner text remain visible when sound is muted.

## Timing and playback

The browser compares consecutive displayed match states. It reads the same 60 Hz countdown and round epoch as the HUD. The initial observation is silent. For example, a player who enters on “3” hears “2”, “1” and the start tone. A player who sees the previous round end can also hear the next “3”. A skipped number is never queued for later playback. The simulation or private server determines which result to announce.

Returning from pause, reconnecting or returning to a visible tab establishes a fresh baseline. A gap over 500 ms between observations also suppresses the transition across that gap. The match menu can stay open in private play while results continue. Each fresh round cue stops the previous voices and uses the existing mute and audio unlock behavior. [State transitions](../../web/src/audio/RoundCues.ts) · [Playback](../../web/src/audio/SkylineSound.tsx)

## Original cues

All five cues use a centered sine oscillator, a 16 ms attack and a peak envelope gain of 0.07. Repeated pulses are separated by 70 ms between the envelope decay endpoint and the next attack. [Tone definitions](../../web/src/audio/matchTone.ts)

| Game event | Frequency sweep | Pulses | Decay endpoint after pulse start |
| --- | --- | ---: | ---: |
| Countdown change | 440 Hz | 1 | 75 ms |
| Play begins | 660–880 Hz | 1 | 160 ms |
| Round won | 554–740 Hz | 2 | 120 ms |
| Draw | 294–247 Hz | 2 | 130 ms |
| Match won | 784–1047 Hz | 3 | 160 ms |

## Verification

Both new match tests failed on the preceding production bundle because no round voice started. On the updated build, Chromium and Playwright WebKit recorded the countdown, three physical ring-outs and one final match cue in a local first-to-three. In a private match both clients started one point cue and a client that reconnected during results continued into the next countdown without replaying the point. Chromium also reproduced a same-tick draw with zero points and one descending cue. The existing landing and online knockout scenarios passed. [Local and private matches](../../web/tests/round-audio.spec.ts) · [Draw fixture](../../web/tests/skyline-draw.spec.ts)

Chromium and Firefox rendered all five cues at 48 kHz in an offline audio context. Each had the specified pulse count, a peak sample magnitude above 0.05 and at most 0.071, and silence throughout the last 100 ms of the one-second buffer. Firefox’s live audio context on this macOS host remained suspended at time zero after the start click. The two live Firefox checks also failed with its test-output preference enabled. Playback through a working Firefox output device and human listening remain open. [PCM and fallback checks](../../web/tests/skyline-audio.spec.ts)

All 105 web unit tests, TypeScript checks and the production build passed. The [round transition tests](../../web/src/audio/RoundCues.test.ts) cover repeated snapshots, draw versus score, epoch changes, stalls and reset.
