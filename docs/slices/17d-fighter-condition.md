# Knockout and recovery feedback

Skyline names each fighter who is knocked out or getting up. Local play reads the condition published by the WASM solver. Online play reads the consciousness field for each roster slot in the same authoritative snapshot that supplies the displayed poses. Spectators use the same display. The notice clears when the fighter regains control, is eliminated or the round ends. Current notices can be read immediately after reconnecting.

## Sound

A fresh transition from active to knocked out plays a falling triangle tone, sweeping from 175 to 58 Hz over 310 ms. Moving from knocked out into recovery plays two rising sine tones, each sweeping from 310 to 545 Hz over 110 ms. The source caps the peak of each condition cue at 0.08 linear gain, about −21.9 dBFS. Both are original, centered match announcements. The visible name identifies the affected fighter.

The controller compares consecutive observed conditions within the same round. A new epoch, pause, hidden tab or interrupted stream establishes the current state silently. Further live changes can play. Returning to active clears the notice silently. Simultaneous knockouts share one cue. If one fighter goes down while another starts to recover in the same observation, the knockout cue takes priority and both names remain visible. A separate warning voice keeps the hazard envelope audible when the two events coincide. Muting or leaving play stops both voices.

## Verification

The local Chromium fixture reads fighter positions to steer two players using their normal canvas keyboard bindings. The WASM simulation determines contacts and damage. The test follows Blue through the knockout, the visible get-up and the return to play, and records both oscillator starts.

The online fixture joins two browser profiles to a real native server, feeds movement and attack intentions over their respective sockets and delivers the original server snapshots to both pages. It stops the clinch at the first knockout. Both clients must observe the same four consciousness states and name the downed and recovering fighter. [Online fixture](../../web/tests/online-knockout.spec.ts)

Across three consecutive packaged matches on loopback, all fifteen limbs retained their body IDs through each transition. In adjacent server publications spanning at most six ticks, no limb moved half a metre at a transition. Neither player scored, the match stayed in round one, and a fresh headbutt from the recovered fighter spent stamina on both clients.

The 781-tick sparring fixture compares every body across native and browser physics. It checks each limb on the transition tick and on the first solver tick under the new motor policy. The largest native step across these six samples was 4.73 cm and 0.295 radians. The regression limits are one palm radius (7 cm) and π/8 radians for this specific encounter. Separate native tests confirm that the health transition preserves every body state at knockout, outgoing grips release, incoming grips persist, and motor effort ramps before fresh inputs resume. Browser audio tests render the two new envelopes to PCM and check their pulse counts, peak levels and silent tails. Unit tests cover simultaneous transitions, duplicate frames, stale streams, first observation and a new epoch. Human assessment of how easily the two sounds can be distinguished during a fight remains open.

Five focused native tests and all 63 web unit tests passed. All nine targeted Chromium scenarios passed across the browser runs. The software-rendered local hazard check ran out of wall time three simulated seconds before the first warning. All four audio scenarios passed with the same deadlines using the Mac GPU. Software-rendered real-time performance remains open.
