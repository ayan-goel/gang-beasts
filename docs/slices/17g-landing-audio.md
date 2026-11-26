# Skyline landing feedback

A jump onto the rooftop or a hard stumble against it produces a short, low thump at the fighter’s position in the camera view. Standing, walking in the native fixture and falling through the ring-out sensor stay quiet. The existing animation and match state supply the visual feedback. The effect is available in local and private Skyline matches.

## Contact and delivery

The shared Rust simulation samples pelvis vertical velocity before the solve. For a fighter descending at least 1.5 m/s, it sums the upward component of solved normal impulses across the rig’s contacts with fixed solids whose inward normal has a vertical component of at least 0.7. A total of at least 12 N·s qualifies. This separates a hard landing from the static load of the 40 kg fighter, roughly 6.5 N·s per tick. The event stores the arena tick and nearest whole N·s, capped at 255. A 15-tick cooldown per fighter suppresses follow-up contacts during settling. Each fighter retains one event until another landing or round reset. Movable crates, detached panels, other fighters and sensors do not supply this fixed-surface effect. [Detector](../../crates/engine/src/character/duel_landings.rs)

Local matches read the event with the WASM fighter state. The room server writes the same event into three previously reserved bytes when a client requests the new compact gameplay block. Packet length and publication rate stay constant. Older compact and full clients receive gameplay block one. Each client uses its own view to position the sound. [Wire format](../protocol.md)

The browser accepts events up to 12 simulated ticks old. A new round, join, interrupted stream or return from pause establishes a silent baseline. Duplicate snapshots do not replay a landing. The strongest simultaneous landing gets one voice. Warnings, condition changes, damaging strikes and grip changes take priority in the same observation.

## Original sound

The Web Audio triangle oscillator starts at 85–105 Hz and descends to 39 Hz. The envelope rises for 4 ms, then decays. Greater impulse lengthens the thump and increases volume up to a 90 N·s ceiling. Camera distance scales the entire envelope. The mute control applies immediately. [Synthesis](../../web/src/audio/landingSound.ts)

| Measured signal at 48 kHz | Peak sample magnitude |
| --- | ---: |
| 12 N·s centered at 8 m, each channel | 0.03325 |
| 90 N·s fully left at 8 m, left channel | 0.09920 |
| Same landing at 32 m | 0.02480 |
| Same landing panned right, left channel | Below 0.00001 |

The 255 N·s sample matched the capped 90 N·s sample. At 32 m the rendered signal carried one sixteenth of the energy at 8 m. All measured signals were silent by 169 ms. [Offline audio check](../../web/tests/landing-levels.spec.ts)

## Verification

- Two real jump inputs each produced one contact after descent. The idle opponent and a subsequent walk produced none. In a running ring-out fixture the pelvis dropped to 0.21 m on the roof, producing a 20 N·s stumble at arena tick 193. The subsequent fall below −12 m produced no second contact. [Native cases](../../crates/engine/tests/landing_sound.rs)
- Chromium heard a local landing after an unmuted jump and remained silent through a muted jump. Two independent Chromium clients each started exactly one landing voice for the same server tick. A client requesting the original compact format played in the same room and received valid version-one frames through another player’s landing. The packaged eight-player scoring and two-player knockout/recovery scenarios also passed. [Browser cases](../../web/tests/landing-audio.spec.ts)
- Five release runs on the Apple M3 Pro each exercised eight fighters for 7,200 measured ticks. Median run-level p95 service time was 1.243 ms. One of 36,000 measured ticks exceeded 16.67 ms, reaching 30.107 ms while other verification work was running on the host. Each player received 11,513,803 compact payload bytes per 120 simulated seconds, or 95.95 KB/s. This measures the room and packet encoder. [Five captures](../verification/landing-room-profile.jsonl) · [Benchmark](../../crates/server/examples/room_profile/main.rs)

The native server and packaged WASM shared engine fingerprint `781eca65941f285cbf4e7ae2dfe7fd82b5af1b4f33b1b912f78f6f4da14087cd`. The release workspace passed 315 Rust tests, and all 96 web unit tests passed. TypeScript, Clippy and the production build passed. Listening judgments and effects for ordinary footsteps and loose props remain open.
