# One hundred eight-fighter round resets

The native Skyline room completed 100 ordinary between-round rebuilds across eight matches. Seven additional first-to-three match endings constructed fresh rooms and were excluded from the reset count. All 107 completed rounds produced exactly one score. No pose, velocity, grip-reference, acknowledgement or spawn-restoration assertion failed.

## Workload

Build and run the [release harness](../../crates/server/examples/reset_endurance/main.rs) from the repository root.

~~~sh
cargo test --locked --release -p brawler-server --example reset_endurance --example room_profile
cargo run --locked --release -p brawler-server --example reset_endurance
~~~

The eight drivers send ordinary 18-byte movement and action records through the server room inboxes at 10 Hz during play. Fighters run toward the center, throw punches, grip and lift. After six seconds, seven fighters run east while one stays, rotating the intended survivor across rounds. Every 25th round continues the central fight into Skyline’s scheduled hazards. Physics, elimination, scoring, the results interval and round construction use the production code. The harness supplies no scores, hits, poses, forced eliminations or artificial game-clock jumps.

Every playing tick checks all 120 limbs for finite position, rotation, linear velocity, angular velocity and mass, and checks opponent grips against elimination state. Each due 30 Hz publication goes through the production encoder’s pose, fighter and grip-reference validation. Submitted sequence numbers must reach all eight corresponding acknowledgements. The score expected from the surviving roster is checked every tick, including during results.

At each round transition, the harness compares every fighter body and all nine map objects against the initial spawn state, including body velocities, orientations and prop visibility. It checks that the arena clock returned to zero, the scoreboard retained the correct points, the countdown restarted and the previous controls, knockout counts, grips and acknowledgements cleared. An input for the preceding round must be rejected when play resumes. A fresh match starts all eight scores at zero.

## Measured result

| Metric | Count |
| --- | ---: |
| Room ticks, including countdowns and results | 180,021 |
| Playing ticks | 128,661 |
| Accepted and applied input records | 171,576 |
| Resolved hits | 4,893 |
| Playing ticks with at least one opponent grip | 69,390 |
| Detached walkways / gusts | 8 / 4 |
| Validated snapshots | 90,009 |
| Between-round resets / additional fresh matches | 100 / 7 |

The timestep corresponds to 50 minutes and 0.35 seconds of game time. An accelerated run on the Apple M3 Pro with macOS 26.6.2 and Rust 1.88.0 reached the hundredth reset in 95.25 seconds of wall time. Sampling only the harness process with macOS `ps -o rss=` gave 5,584 KiB after reset 1, 6,720 KiB after reset 10, 6,928 KiB after reset 50 and 6,992 KiB after reset 90. The process had already exited at the final memory query. These samples cover the native simulation and encoder. Real-time scheduling, sockets, client stalls and the hour-long server memory profile require a live run.

Engine fingerprint `58f614835f343e435536a7dbf678ac3c51b4ea387d9057ab3eccff3926339f4b`. Skyline map SHA-256 `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`.

A second full run reproduced every counter in the table. The short eight-player reset regression, three room-profile regressions, 307 release workspace tests, formatting and strict Clippy checks passed.
