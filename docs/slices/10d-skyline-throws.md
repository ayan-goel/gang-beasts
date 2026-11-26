# Throws and counter-grips on Skyline

A fighter who has caught an opponent can hold lift and let go during the swing. The two private-room players now see controls for their own situation. The lifter sees how to throw. The held player sees how to grab back. When both players hold someone, an incoming grip still belongs to its owner and can persist after the other player lets go. The hints use the grip IDs in the displayed snapshot and appear only for an active fighter during play.

## Verification

| Encounter | Result |
| --- | --- |
| Two-handed lift in a private room | Both players observed the target’s entire rig clear the deck and keep moving along the swing after both hands let go. Four packaged Chromium runs measured 0.09–0.26 m/s along the swing between the first two free snapshots received by the host. |
| Defender grabs back | The defender held the lifter using the normal hand keys. Both players saw that attachment persist after the lifter released both hands. The defender then released it independently. |
| Matched native retreat | Both simulations reached identical limb poses and velocities at release. After the same one-second retreat by the lifter, the counter-grip kept pelvis separation below 1.5 m. Releasing it increased separation by more than 1 m. |
| East-edge throw in browser WASM | Coral ran toward Blue, caught Blue with one hand and walked outward while lifting. At release, Blue’s pelvis was still inside the 10 m edge and moving outward faster than 2 m/s. Every collider cleared the deck by over 10 cm. Blue fell and Coral scored. With lift omitted from the same inputs, Blue stayed in contact with the deck and both players survived. |

The private-room test steers the initial approach using ordinary movement axes. Steering stops once the hands attach. J, K and E go through the browser keyboard controls. Each client’s observer measures the mass-weighted center and lowest collider point across all 15 bodies. Comparing horizontal movement across release allows for gravity acting through the top of the arc. In the rooftop comparison the input script steps the local WASM simulation directly. It also checks that releasing the hand preserves all limb states at that instant. [Online encounters](../../web/tests/online-lift.spec.ts) · [Pose measurements](../../web/tests/support/liftTrace.ts) · [East-edge comparison](../../web/tests/skyline-throw.spec.ts) · [Native counter-grip comparison](../../crates/engine/tests/skyline_hand_actions.rs)

The online check requires at least 0.08 m/s of pre-release swing to distinguish travel from idle motion. The first two free airborne samples must retain over half that speed along the original direction. These tolerances allow different snapshot intervals around release. The native test separately checks every limb’s position, orientation and velocity at the instant the constraints are removed.

These results cover the physical lift, release momentum and one form of counterplay. They establish the technical lifts/throws criterion in §33. Human assessment of timing, effort and readability remains open. The packaged browser runs used Chromium on macOS loopback with Metal. The test build used the hand tuning, grip tuning, character rig and Skyline map from commit `31811e6`.

The release workspace passed 298 Rust tests. The web suite passed 69 unit tests. The production build, formatting and Clippy checks passed. The final nine-scenario Chromium run covered both hand bindings, a reconnect during a grip, the eight-player roster, local crate and ledge interactions, the matched rooftop throw and both private-room encounters.
