# Round and match scoring

Skyline settles each round after evaluating every fighter’s pelvis for that tick. A sole survivor earns one point. A simultaneous fall by the last two fighters ends the round in a draw. The local and private-room result banners identify the round winner, identify the match winner on the third point, and explain that a draw awards no point. [Settlement](../../crates/engine/src/character/duel_match.rs) · [Melee state](../../crates/engine/src/melee.rs) · [Result text](../../web/src/game/meleeResult.ts)

## Simultaneous falls

The native regression starts the second fighter running east at playing tick 38 and the first fighter running west at tick 221. It sends ordinary run inputs at 10 Hz. The different starting times account for the asymmetric paths from the spawns. Both pelvises cross below the fall plane in the same solve. With two fighters, the test checks a draw and two zero scores. With a third fighter still on the deck, it checks a single point for that fighter. Each outcome must survive the full 300-tick results interval and carry the correct scores into round two. [Native room regression](../../crates/server/tests/room_scoring.rs)

## Repeated results

The private-room scenario gives the host a second copy of a genuine round-result snapshot. It sends the identical bytes again immediately after the first snapshot of the next round. The guest receives the ordinary stream. Each player wins one keyboard-driven round. Both scoreboards show the winner at results and retain the correct total after the transition. A separate match checks both players’ result banners and scores when one reaches three wins, then repeats the match in the same room. [Two-player scenario](../../web/tests/online-match.spec.ts) · [Repeated delivery](../../web/tests/support/repeatResults.ts) · [Match results](../../web/tests/support/rematchScenario.ts)

## Browser results

The packaged local WASM game reproduced the two-fighter draw with both scores at zero. A test hook scheduled the run commands at the same playing ticks as the native room. The ordinary game loop performed every solve, presented the result and advanced to round two. [Local draw](../../web/tests/skyline-draw.spec.ts)

In the eight-player private match, seven users ran off the roof using keyboard controls. All eight browsers showed “Fighter 1 wins the round” and scores of 1–0–0–0–0–0–0–0. All eight retained those totals across the reset and cleared the prior elimination markers. Seven clients used compact snapshots and one used the original full format. The separate two-client test passed with two immediate duplicate deliveries and two delayed previous-round deliveries to the host. Both clients also agreed on two 3–0 match results. [Eight-player scenario](../../web/tests/online-roster.spec.ts)

These checks establish the §33 scoring criterion. Five packaged Chromium scenarios passed on Apple M3 Pro, macOS 26.6.2 and Metal over loopback. The release workspace passed 308 Rust tests. The web suite passed 79 unit tests. The release web build, TypeScript, formatting and strict Clippy checks passed.

Engine fingerprint `58f614835f343e435536a7dbf678ac3c51b4ea387d9057ab3eccff3926339f4b`. Skyline map SHA-256 `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`.

To repeat the native cases, run `cargo test --release --locked -p brawler-server --test room_scoring`. To repeat the browser cases, run `npm --prefix web run test:e2e -- online-match.spec.ts online-roster.spec.ts skyline-draw.spec.ts skyline.spec.ts`.
