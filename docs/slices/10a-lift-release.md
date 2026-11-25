# Local lifting and release

Coral holds E and Blue holds slash while gripping. Each attached arm raises its anatomical shoulder and elbow targets. A hand holding nothing contributes zero lift. Releasing the lift key lowers the arms while leaving the grip intact. Releasing a hand removes that joint immediately. The other hand and the opponent’s controls remain independent.

## Grounded assistance

A foot must have a current supporting contact on fixed or kinematic geometry to receive upward assistance. This prevents two fighters from using each other as a floor in midair. In flight, internal arm motors can still move. The local map currently has a fixed floor.

Each attached hand has a 400 N maximum. The vertical controller aims for the hand to reach 0.45 m above the chest, reduces effort over the final 0.10 m and accounts for relative upward speed with 0.06 s of lookahead. Of that capped effort, 75% acts on the hand and 25% on the lifter’s chest. This is grounded character assistance. The opponent moves through physical constraints and collisions. Its mass, posture, resistance and attachment locations all affect the outcome. Release applies no bonus impulse.

An early internal force-pair prototype pushed the lifter over during the approach. The bounded, support-dependent torso contribution lets the test lifter brace and raise the other fighter through the existing grips. Human tuning during play remains open.

## Evidence

- `cargo test --release --locked -p brawler-engine --lib duel::lift_tests` covers unchanged motion with empty hands, one- and two-hand force caps, arm target isolation and removal of grounded assistance when support disappears.
- In the two-hand fixture an active 40 kg opponent clears the deck with **every collider** at least 15 mm above it. Its pelvis rises more than 8 cm above its initial height and more than 10 cm above the matched non-lifting case. Releasing both hands while the opponent is rising leaves all 15 body states identical at the release instant.
- The 376-frame native/WASM trace includes a lift and airborne release. The sampled pelvis rises from approximately 0.750 m before the lift to 0.973 m before release at 0.82 m/s upward, then continues to 1.002 m three ticks later. See `cargo run --release --quiet --locked -p brawler-engine --example duel_trace` and `npm --prefix web run test:e2e -- duel.spec.ts duel-grips.spec.ts`.

This slice establishes lift-and-release in a local flat-floor encounter. Ledge rescue, varied prop masses, online input and human playability have separate gates.

At this checkpoint the release workspace suite passed 208 tests. The two documented stair-ascent and top-tread stopping tests still failed. The production web build, formatting, Clippy, 20 web unit tests and 46 Chromium tests passed.
