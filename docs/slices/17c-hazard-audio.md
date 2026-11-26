# Skyline hazard audio

Local and online Skyline play a short cue when the active round enters each hazard stage. The words already shown on the arena remain visible with sound muted. Both presentations read the same stage they use for the on-screen warning. The local controller gets it from the WASM solver. The online controller gets it from the displayed authoritative snapshot and the verified map schedule.

## Original cues

Each cue uses one Web Audio oscillator with an amplitude envelope. Frequencies sweep exponentially over each pulse. A 16 ms attack and a decay to silence avoid abrupt starts and stops. Peaks are capped at 0.12 linear gain (about −18.4 dBFS). The global warning stays centered at a fixed level as the camera moves.

| Event | Waveform | Frequency sweep | Pulses | Pulse duration |
| --- | --- | --- | ---: | ---: |
| West walkway failing | Triangle | 520 to 370 Hz | 2 | 160 ms |
| East walkway failing | Triangle | 680 to 470 Hz | 3 | 130 ms |
| High winds approaching | Sine | 215 to 620 Hz | 2 | 280 ms |
| Sudden death crosswind | Sine | 430 to 145 Hz | 1 | 480 ms |

The [synthesis source](../../web/src/audio/warningTone.ts) is the original asset. One voice can play at a time. A new cue replaces the prior voice, and the graph disconnects when its envelope finishes.

## Timing and ownership

Within a round epoch, the highest observed stage identifies the warnings the client has already passed. Repeated state, camera movement and older stages cannot reissue a warning. The first observation establishes the current stage. Returning to play after a pause, hidden tab, stalled stream or reconnect also establishes a baseline. A gap over 500 ms between observations suppresses a transition that may have happened while the presentation was interrupted. Subsequent live advances play normally.

The local game and private rooms each own one page-lifetime output. A trusted click or keypress can unlock the browser audio device. The device is opened lazily, and a previously opened context can resume on visibility return or another interaction. Mute immediately silences an active voice. The mute button works before the match and during play, and its versioned preference survives navigation and reload when local storage is writable. The preference stays in memory if storage is unavailable. Hiding the page, leaving a match or closing the output clears the current sound.

## Verification

Unit tests exercise live transitions, duplicates, older stages, epoch changes, skipped time and a late-join baseline. Chromium renders all four cues into PCM samples and measures pulse counts, peak amplitude and a silent tail. The local match test runs the real WASM round on its ordinary clock and checks that crossing the west warning starts one running audio voice. Pausing and resuming during the warning preserves silence.

The two-client scenario runs the native room through its normal hazard deadlines. The host schedules one cue per new stage. A muted guest keeps the visual warnings, enables sound partway through, reloads during the east warning and receives the subsequent wind cues. The browser probe records actual oscillator starts and audio-context state. The packaged game also starts a local round and accepts mute when both Web Audio and persistent storage are unavailable. Listening on physical devices remains a human acceptance gate.

The web suite passed 61 unit tests and the TypeScript production build. Eight targeted Chromium scenarios passed on the development build. Four checks also passed on the packaged preview, covering the two-player hazard schedule, mute persistence, the narrow layout and operation without audio or storage.
