# Opponent grips in local sparring

After a held punch completes, the hand can acquire one of the opposing fighter’s physical colliders. The two-fighter fixture orders candidates by signed surface distance, breaking ties by anatomical order. A ray between the nearest surface points rejects intervening solid geometry. The bundled 25 mm margin permits close surfaces to attach. The midpoint becomes a local anchor on both bodies.

The joint has three force-based linear springs and free angular motion. Each axis receives at most 1200/√3 N, bounding the resultant while allowing a loaded grip to stretch. The spring and damping gains live in `config/grip.json` and contribute to the engine configuration hash. The hand motor continues to request its reaching pose. Removing the joint changes neither fighter’s pose nor velocity.

## Verification

Run `cargo test --release --locked -p brawler-engine --test duel_grips` and `cargo test --release --locked -p brawler-engine --lib grip_contact`.

- A tap and a distant held hand produce no attachment. A nearby held hand can grip while the other hand enters its strike phase.
- Two approaching fighters can form a closed loop, stay attached for 60 ticks with finite states, and release their outgoing grips independently. Reset clears attachments.
- In matched 30-tick retreats, the released hand and its former target separate by at least 10 cm more than the held pair. Explicit release preserves all body-state fields at the release instant.
- A 1 mm solid barrier occludes the tested 24 mm surface gap. Turning the barrier into a sensor or disabling it permits the same segment.

The fixture covers two live fighters. Surface grips, target destruction, stamina/load failure, and eight-player stress have separate acceptance requirements.
