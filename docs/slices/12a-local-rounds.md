# Local Skyline melee rounds

The local match starts with both keyboard slots loaded. A 180-tick countdown precedes each round. Only the playing phase advances the physical world and accepts gameplay input. The round clock starts at zero each time.

After each completed solve, the engine checks every fighter’s pelvis against the fall plane. Elimination is permanent within the round. It disables that fighter’s bodies and removes incoming and outgoing grips before publishing the final consciousness transition. Simultaneous falls result in a draw. One surviving fighter earns one point. Multiple survivors at the end of sudden death draw. Leaving the playing phase consumes the outcome exactly once.

Results remain visible for 300 ticks. A match-winning result (three wins) enters match results. Otherwise a new world is built from the map and character definitions before the next countdown. Scores persist across rounds. Rematch resets scores, starts at round one, and advances the round identity so old state cannot address the new round.

## Verification

- `cargo test --release --locked -p brawler-engine --test skyline_melee` drives a real fighter off the edge three times. It checks the score, results delay, full prop reset, match win and rematch.
- `cargo test --release --locked -p brawler-engine --lib rounds::tests` exercises simultaneous physical falls and a round reaching the final deadline.
- `cargo test --release --locked -p brawler-engine --lib melee::tests` covers an eight-slot scoring mask and duplicate results. The current physical match has two local fighters. Private online sessions and an eight-fighter physical world remain open.
