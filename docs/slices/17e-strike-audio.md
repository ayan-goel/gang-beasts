# Damaging strike feedback

A damaging punch or headbutt now plays a short thud at the defender’s position in the view. The match derives the sound from the same health state that determines a knockout. Input, hand motion and contact with an undamaged fighter cannot start it. The local game publishes its presentation state on a resolved hit as well as its ordinary HUD cadence. Online play uses the state selected by the interpolation timeline.

## Mixing

A decrease of at least one health point between successive live observations qualifies while the defender stays active. Damage from multiple strikes between two observations contributes to one sound. For simultaneous defenders, the largest qualifying decrease supplies the sound. Each defender has a 120 ms cooldown. A new round, interrupted stream, hidden tab or pause establishes a fresh baseline. The existing mute setting controls impacts as well as other cues. A hazard, knockout or recovery announcement suppresses a coincident impact and stops a lingering thud.

The waveform combines a falling 165-to-65 Hz sinusoidal component with a brief seeded noise transient. It is generated once per audio context. A two-millisecond attack and damped tail limit sharp edges. Damage up to 35 points determines pitch and gain. The nominal maximum linear gain is 0.16, about −15.9 dBFS before the source envelope. The sound uses the defender’s rendered pelvis projected into the current camera view for stereo pan. Distance from that camera scales gain by 8 m / distance beyond eight metres. Global warnings and condition cues keep their centered mix.

## Verification

The cue tests cover simultaneous damage, independent cooldowns, duplicate observations, regeneration, negligible damage, a long gap, a fresh epoch and warning priority. Chromium’s stereo audio renderer checks left/right placement, capped loudness, lower energy for a lighter strike, inverse-distance attenuation and a silent tail after 150 ms. Local missed punches and headbutts at the initial spawn positions leave the impact voice silent.

Two packaged Chromium clients each started the impact voice during the same native-server fight. Both then observed the knockout, heard the condition cues and saw Blue recover. The local shared-keyboard fight passed twice with audio enabled and twice muted. Muting suppressed both the impact and the condition sounds while the visible knockout still appeared.

The initial keyboard fixture sometimes timed out because it began issuing controls during the countdown-to-play render transition and steered according to render cadence. The repeatable fixture starts 24 physics ticks into play and steers once per fixed tick using actual keyboard actions. The web suite passed 76 unit tests, TypeScript checking and the production build. Browser verification included the real audio graph, silent misses, unavailable audio/storage, persistent mute, local and online combat and private-room grips. Human listening and mix preferences remain open.
