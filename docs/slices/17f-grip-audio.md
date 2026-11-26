# Grip feedback

A catch makes a short downward scrape, a release makes a lighter upward sound, and a tiring grip creaks. Each cue follows the holder's rendered pelvis in the current camera view. The existing grip labels and low-stamina meter show the same actions with sound muted.

## Gameplay decisions

- Local play reads the two physical hand targets from WASM. Private matches use the corresponding target identities from authoritative snapshots. An empty hand produces no catch. Moving directly from one target to another produces a new catch.
- The creak fires on crossing from above 25 stamina to 25 or below while the same hand still holds the same target. This threshold is shared with the visual meter. Exhaustion or a voluntary let-go can then trigger a release. The creak represents low stamina, independent of the instantaneous force on the joint.
- One observed update can start one grip voice. Strain takes priority over a catch, and a catch over a release. The nearer holder wins a tie. Each cue kind has a 150 ms cooldown across the match. A warning, knockout, recovery or damaging impact takes priority on the same update. A new sound replaces any unfinished grip voice.
- Joining, changing rounds, resuming after interruption or receiving an observation more than 500 ms after the last one establishes a silent baseline. Repeated snapshots of a continuing hold produce no further sound.

[Grip selection](../../web/src/audio/gripCues.ts) · [Match cue lifecycle](../../web/src/audio/MatchCues.ts)

## Sound provenance and levels

All three sounds use an original filtered sawtooth sweep and gain envelope synthesized with Web Audio. At 48 kHz, Chromium's offline renderer produced the following single-channel peak amplitudes at a camera distance of 8 m and full left or right pan. The duration is the last sample whose absolute amplitude exceeded 0.0001. [Sound source](../../web/src/audio/gripSound.ts) · [PCM regression](../../web/tests/grip-audio.spec.ts)

| Cue | Frequency sweep | Measured peak | Duration above threshold |
| --- | --- | ---: | ---: |
| Catch | 320 to 115 Hz | 0.0631 | 51.7 ms |
| Strain | 145 to 72 Hz | 0.0738 | 186.5 ms |
| Release | 180 to 360 Hz | 0.0439 | 68.7 ms |

At 32 m the catch peak was 0.0158, one quarter of its value at 8 m. The opposite channel's absolute peak stayed below 0.00001 for each fully panned cue.

## Gameplay verification

In the local release build, holding both hands in empty space produced no grip sound. Walking up to a rooftop crate produced the visual grip and catch sound. Letting go removed the notice and produced a release. In a private match, both players independently observed the same catch, strain and release sequence as the server drained the holder's stamina. Keeping the key down during recovery produced no extra catch. [Local gameplay](../../web/tests/grip-audio.spec.ts) · [Private match stamina and audio](../../web/tests/online-stamina.spec.ts)

The release build and all 88 web unit tests passed. The refreshed playtest preview passed the audible and muted local crate cases and the two-client fatigue case. The existing damaging-impact, knockout, mute-storage and timed walkway audio regressions also passed.

Measured on an Apple M3 Pro, macOS 26.6.2, Chromium 159.0.8154.6. Private matches used loopback. The measured amplitudes describe rendered PCM. Listening assessment on physical speakers and headphones remains open.
