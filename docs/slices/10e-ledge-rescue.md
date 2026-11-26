# Recovering from a ledge catch

Skyline shows the climber’s pull-up key while a hand holds the main roof or a walkway that is still attached. The hint identifies Space for the first local player or a private-room player and Enter for the second local player. A detached walkway can still carry a physical grip. Its prompt clears on detachment because a falling panel cannot support a pull-up.

## Fall-arrest timing

When the west walkway falls, the fighter’s body can swing below the wrist before the arm supports its weight. In the original 24-tick, 740 N preset, five of eight neighboring approaches caught the main roof and then lost the grip. They differed only in when the player stopped moving inward. In one case the hand broke free with more than 94 stamina remaining.

The bundled lip preset now tolerates 72 accumulated overloaded ticks at 740 N. This gives the initial swing time to settle. In a diagnostic replay one near-edge approach accumulated 66 overloaded ticks on one hand. A tick below the force threshold dissipates a tick of strain. The contact test, palm travel, force sampling, stamina drain and pull forces use the same configuration as the original catch. Fighter and prop grips retain their eight-tick window.

## Verification

- Across eight consecutive stop times, ticks 7076–7083, Coral made palm contact when the walkway fell. Forty ticks after contact the pelvis was outside and below the roof and a hand still held it. Holding jump then returned Coral to foot support on the deck in every case. [Native rescue cases](../../crates/engine/src/character/mantle_tests.rs)
- The same constrained spring supports a steady 40 kg load on one hand. A sustained 80 kg load still breaks within 90 ticks. [Load calibration](../../crates/engine/src/character/grip_joint.rs)
- The prompt tests cover the main roof, each walkway before and after its own detachment, and grips on fighters or props. The local browser case exercises the visible prompt, keyboard Space and supported return. [Prompt tests](../../web/src/game/ledgeHint.test.ts) · [Local browser case](../../web/tests/skyline-mantle.spec.ts)
- In a private room the test steers the initial approach with ordinary movement axes. The west walkway detaches on its scheduled tick. The player holds J and K to reach and presses Space after catching the main deck. Both Chromium clients see a hand attached while the pelvis is outside and below the roof, followed by the fighter standing inside the roof with hands free, round one still playing and both scores at zero. [Private-room rescue](../../web/tests/online-ledge.spec.ts)
- In the production preview, the climber’s connection had 60 ms of added delivery delay in each direction from before the collapse through recovery. At arena tick 7232 the pelvis was at x = −10.29 m, y = −0.52 m with the right hand on the roof. At tick 7506 the pelvis was at x = −9.37 m, y = 0.64 m with both hands free. The climber then walked another 0.8 m inward in both views. Only the climber saw the Space prompt.

The packaged native server and browser reported engine fingerprint `57dae7ad2c811965c2e652662dba8c9946aadded5e448d095dec3bcd25fa95e4`. All 299 release-mode Rust tests and 71 web unit tests passed, along with Clippy, TypeScript and the production build. Eleven focused Chromium regressions passed, including a second run of the delayed rescue, independent grips, airborne throws, grip stamina, an eight-player room, local catch and voluntary release, and a local pull-up. The network case covers fixed application-message delay on the climber’s connection. Jitter, TCP retransmission and human playability remain open. In the default camera view the roof hides most of the hanging fighter. The grip label and pull-up prompt stayed visible.
