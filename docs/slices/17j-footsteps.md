# Rooftop footsteps

In local and private Skyline matches, a walking or running fighter produces a quiet boot tap when a foot takes weight on the roof. A player can hear an opponent approach from the side of the screen. The existing effects, master and mute controls apply.

## When a footstep sounds

After each Rapier solve, the fighter must be active, receiving movement input, clear of get-up assistance, chest above pelvis and moving horizontally at least 0.45 m/s. A sole must cross from below 0.25 N·s of upward contact impulse to at least that threshold on a fixed solid. The contact normal must have an upward component of at least 0.7. A moving prop or a wall does not qualify. Separation of nine simulation ticks between taps limits chatter. A hard landing reserves the next fifteen ticks for its own cue. The detector only reads the completed solve. [Contact detector](../../crates/engine/src/character/duel_landings.rs)

WASM exposes the last round-local contact tick to the local client. Private rooms transmit recent contacts in negotiated compact snapshots, retaining the existing packet size. Each listener chooses the closest newly observed step and allows at least 90 ms between taps. A simultaneous warning, condition change, damaging strike, grip cue or hard landing takes priority. Resync and resume establish a silent baseline. [Protocol](../protocol.md#full-skyline-snapshots) · [Cue selection](../../web/src/audio/MatchCues.ts)

## Sound and verification

An original 80 ms mono sample combines a damped 125 Hz tone with filtered noise. Playback rate varies from 0.96 to 1.04 with the contact tick. Camera position controls stereo pan and distance attenuation. Chromium rendered a peak amplitude of 0.03815 fully left at 8 m and 0.00954 at 32 m. The opposite channel stayed below 0.00001. The slowest sample was silent by 87.5 ms. [Sound source](../../web/src/audio/footstepSound.ts) · [Rendered waveform test](../../web/tests/footstep-levels.spec.ts)

- Native walk and run fixtures each produced at least three spaced contacts over 100 ticks. Idle, standing jump, airborne movement, input release and reset have dedicated assertions. [Native regression](../../crates/engine/tests/footsteps.rs)
- Chromium and WebKit exercised local travel, stop and mute; two live clients received the same sequence of contact ticks and each started at least two voices. The stationary fighter emitted none. A client requesting only landing records played alongside a footstep-capable host. Chromium also exercised the original compact client. [Footstep scenarios](../../web/tests/footstep-audio.spec.ts) · [Landing scenarios](../../web/tests/landing-audio.spec.ts)
- Five M3 Pro release runs each measured 7,200 ticks of eight-fighter combat after warm-up. Median run-level p95 service time was 0.962 ms, worst observed tick was 4.004 ms, and zero of 36,000 measured ticks exceeded 16.67 ms. Every recipient received 11,513,803 payload bytes per 120 simulated seconds, matching the earlier landing-only profile. These timings cover simulation and snapshot preparation. [Raw runs](../verification/footstep-room-profile.jsonl) · [Workload](../../crates/server/examples/room_profile/main.rs)

Engine fingerprint for the native server and packaged WASM was `2fed9f7d990be5b20360402ec1b29e7557da0e91fb942a9a9a39a7526e335255`. Human listening and movement-feel assessment remain open.
