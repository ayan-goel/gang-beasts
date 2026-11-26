# Independent hands in online Skyline

While a grip is attached, private matches show the owner, hand and target below the scoreboard. An opponent grip identifies the fighter and anatomical region. A world grip identifies a crate, vent, roof edge or walkway edge. Local Skyline and the physics playground share the same names. The online list wraps beneath the arena so a cluster of grips cannot cover the fighters.

## Source and lifecycle

The full snapshot carries a separate target ID for each hand. Target zero means the hand is free. Fighter IDs encode match roster index and anatomical part index. Match roster order can differ from the original lobby seat numbers. Arena IDs refer to objects in the verified map manifest. The client resolves targets from the snapshot already used for the displayed body poses. It can display an incoming grip from the first complete snapshot after a reconnect.

A notice begins when the displayed snapshot contains an attachment and clears when the snapshot shows its release. A canvas blur sends neutral input through the ordinary game controls. Results screens hide the grip list, and a new round reads its own snapshot.

## Verification

- In two mirrored rooftop fights, the native solver applied damage from the free fist while the other hand held the opponent. The target fighter, limb and original grip creation tick stayed the same through impact. Releasing the holding hand removed the constraint. [Native fixture](../../crates/engine/tests/skyline_hand_actions.rs)
- In a private room, two browser profiles received the same target ID and the free hand’s strike phase. The test steers the approach using server positions. The owner uses the normal J and K keyboard bindings for the hold, punch and release. Both clients show the limb identified by the packet, and both clear the label after release. Reloading the held fighter’s tab leaves the incoming grip visible. [Two-client scenarios](../../web/tests/online-grips.spec.ts)
- With keyboard movement and hand controls, both clients identified the same crate, observed over 10 cm of crate movement during continuous attachment, and cleared the grips on canvas blur. Lift height and release momentum have separate acceptance fixtures.
- The label tests cover left and right hands independently, both occupied, both free, the eighth fighter, all four map object types and anatomical names with multiple words. [Label tests](../../web/src/online/gripLabels.test.ts)

The release workspace passed 297 native tests. The web suite passed 65 unit tests, and the production build, formatting and Clippy checks passed. All 12 targeted Chromium scenarios passed, including local hand controls, online knockout and an eight-player match. The packaged two-client scenarios also passed with the exact target-limb checks.

These fixtures verify the independent-hands criterion in §33. Human assessment of grip readability and throw counterplay remains open.
