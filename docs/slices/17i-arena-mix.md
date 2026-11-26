# Skyline arena music and volume

Open the local pause menu or the private match menu to adjust Master, Music and Effects. Each slider accepts keyboard input and shows its percentage. The persistent mute button also remains available outside the menu. Master and Effects start at 100% to preserve existing cue levels. Music starts at 60%. Mute preserves all three selections.

## Playback

During a visible, active round the listener hears a sixteen-second original instrumental. Eight bars at 120 BPM combine a syncopated bass with a higher plucked melody. Each note uses a fundamental, a second harmonic and a short attack with a squared decay. The last note decays before the loop boundary. The score uses no external samples or music downloads. [Score and synthesis](../../web/src/audio/arenaScore.ts)

The Web Audio graph routes both the music and effects gains through the master gain. Faders use linear amplitude and an eight-millisecond time constant for live gain changes. For example, master at 50% and effects at 30% give a warning 15% of its full-level amplitude. The existing camera panning and distance attenuation happen before the effects gain. [Mixer](../../web/src/audio/MatchMix.ts) · [Cue routing](../../web/src/audio/SoundOutput.ts)

The first audible round generates a 256,000-sample mono buffer at 16 kHz. One loop source plays per listener. Local pause, round results, a stalled private stream, a hidden tab and an interrupted audio context stop playback. Returning to live play starts the phrase again. The private match menu keeps playing the current round while the listener adjusts the mix. The simulation, event clock and other players' output are independent of these controls.

The browser stores validated integer percentages and mute together in `skyline.audio.v2`. A saved version-one mute setting is honored on first use. Invalid fields fall back independently. A blocked store leaves the settings available in memory. [Preference handling](../../web/src/audio/volumeSettings.ts)

## Measurements

| Check | Result |
| --- | --- |
| Original PCM, peak and RMS amplitude | 0.05726 peak, 0.01220 RMS at unity gain across all sixteen seconds |
| Full-level loop buffer | 1,024,000 bytes per audio context, reused across rounds |
| Score generation, Chromium on the M3 Pro | 4.2 ms on the first call and 3.1–3.2 ms on three subsequent calls. This measures synthesis alone |
| Channel gain on a mono output | A 50% master with 40% music rendered 20% of the baseline music peak. A 50% master with 30% effects rendered 15% of the baseline warning peak. Muted or zeroed outputs rendered exact silence |

The [offline graph check](../../web/tests/mix-render.spec.ts) rendered actual PCM in Chromium and Firefox. The [playable cases](../../web/tests/volume-mix.spec.ts) cover local pause, keyboard adjustment, reload, independent two-client playback and mute restoration. The original peak and RMS describe the source signal. Human listening and device-dependent output volume remain open.

## Playable verification

On the M3 Pro, Chromium and Playwright WebKit passed the local and two-client mixer scenarios. The production Chromium run also passed all five existing landing and round scenarios, including a first-to-three match and reconnect during results. The walkway warning passed in a separate uninterrupted run. At a 390-pixel viewport width the pause dialog used 350 pixels without horizontal overflow, and keyboard input set music to zero. All 108 web unit tests, TypeScript checks and the production build passed.

Two existing private cue checks failed in Chromium with its default headless renderer on this host. The second client missed the first landing, and the countdown check observed an empty cue sequence. Repeating the same tests with the same browser settings against the preceding production release reproduced both failures. The updated release passed both checks with Metal acceleration. Two-client cue delivery on the default headless renderer remains open. The mixer checks passed there.
