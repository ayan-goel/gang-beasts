# Eight fighters in a closed grip loop

The release engine held a single cycle connecting all eight Skyline fighters while they walked outward against each other. The fixture runs twice, once with every right hand holding the next fighter and once with every left hand holding the previous fighter. Both runs use the standard fifteen-body rigs, hand action timing, contact selection, grip springs, collisions, stamina and get-up controllers.

## Reproduce

```sh
cargo test --locked --release -p brawler-engine --lib eight_fighters_form_one_closed_loop -- --nocapture
```

The test starts an eight-player Skyline match and runs the countdown. Before the first playing solve it places the eight complete rigs on a one-meter-radius circle at the roof center, each rotated 0.8 radians from facing inward. It then presses the chosen hand once per fighter. Starting at the usual scattered spawns produced zero ticks with a closed loop. The fixture supplies only the initial placement. The game creates and releases every grip joint. [Native fixture](../../crates/engine/src/character/grip_loop_tests.rs)

| Playing ticks at 60 Hz | Input and checks |
| --- | --- |
| 1–119 | Hold one hand. All eight acquired the next neighbor on tick 25. From tick 60 onward the test checks each target, original acquisition tick and the total of 120 joints. |
| 120–164 | Every fighter walks radially outward while holding. The test requires the measured grip load to exceed 100 N to prove the loop was stressed. |
| 165–539 | Release movement and continue holding. The same eight grips must survive with all fighters present. |
| 540–2340 | Release the hands. All 112 anatomical joints remain and no grip returns. Observe another 30 seconds, including recovery motion, then compare the final second of kinetic energy with the loaded interval. |

## Measurements

| Peak across all fighters and ticks | Right-hand cycle | Left-hand cycle |
| --- | ---: | ---: |
| Anatomical joint anchor error | 12.73 mm | 11.31 mm |
| Compliant grip anchor separation | 0.241 m | 0.191 m |
| Reported grip load | 442.36 N | 411.17 N |
| Limb speed | 6.00 m/s | 5.90 m/s |
| Limb angular speed | 59.70 rad/s | 28.14 rad/s |
| Total fighter kinetic energy | 495.16 J | 249.87 J |
| Mean energy during outward movement | 136.92 J | 83.84 J |
| Mean energy over the final second | 0.045 J | 0.006 J |

Each solve checks all 120 enabled bodies for finite pose and velocity. The 5 cm anatomical tolerance comes from §29.1. The separate 0.31 m ceiling accommodates a compliant grip at its configured force limit (1,200 N divided by 4,200 N/m plus 0.025 m reach tolerance). The fixture uses tripwires of 12 m/s, 100 rad/s and 1,000 J for a runaway solve. It also requires final mean kinetic energy below both 1 J and one percent of the loaded mean. Three seconds after release the right-hand group was still moving and getting up. A 30-second observation captures its return to rest.

The runs used Apple M3 Pro arm64, Rust 1.88.0, release mode, eight solver iterations, the bundled Skyline map and engine fingerprint `9fc04f93698e513140a17094914fb8ba7acc997003fabe7e93ee81be52cf33c1`. This verifies the native closed-loop fixture. Server latency, other worst-case workloads and the full stability gate have separate records.
