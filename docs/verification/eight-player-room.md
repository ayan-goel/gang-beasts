# Eight-player Skyline room baseline

On an Apple M3 Pro, the median run-level p95 time to accept due inputs, advance an eight-fighter room and prepare any due snapshots was **0.955 ms**. Each player's full snapshots used **130.72 KB/s**. Full snapshots alone exceed the §28.1 optimized whole-stream target of 100 KB/s.

## Reference run

Measured 2026-11-26 from gameplay revision `8efed6e`. Apple M3 Pro, 12 logical CPUs, 18 GiB RAM, macOS 26.6.2 arm64, AC power. The benchmark ran one unpinned thread with the ordinary desktop background load. A preceding sandboxed series recorded a one-minute system load of 2.58 near completion. Thermal data was unavailable. Rust 1.88.0, Cargo release defaults, locked Rapier 0.26.1 with enhanced determinism, default game configuration. Engine fingerprint `c7f509a7225ff049e341584554951ab9e1914e03057a24d0902f31e8d9a7fd6c`, Skyline SHA-256 `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`.

~~~sh
cargo test --locked --release -p brawler-server --example room_profile
cargo build --locked --release -p brawler-server --example room_profile
/usr/bin/time -l target/release/examples/room_profile
~~~

The example outputs one JSON object per run. Each run starts in a fresh room, advances 30 simulated seconds including countdown, then measures 120 simulated seconds at the fixed 60 Hz timestep. The loop advances as fast as the host permits. Eight drivers steer toward the center from the default spawns and stagger holds, lifts and taps at 10 Hz. The script uses no random inputs. The measured interval spans the scheduled west walkway failure. Each run produced 9,600 accepted input records, 402 resolved hits and 7,098 ticks with an opponent grip. All eight fighters survived throughout all 7,200 measured ticks.

## Service time

All times are microseconds. Total includes server input submission, room stepping and, every second tick, shared snapshot capture and eight recipient copies. Step includes controllers, Rapier and gameplay. Publication percentiles use only the 3,600 ticks that produced snapshots. Bot decision-making, activity counters and percentile calculations run outside the measured interval. Timers measure wall time on the calling thread. The budget column counts total samples over 1/60 second using unrounded durations.

| Run | Total p50 | Total p95 | Total p99 | Total worst | Step p95 | Publication p95 | Over budget |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 840 | 927 | 968 | 1,141 | 926 | 3 | 0 |
| 2 | 874 | 961 | 1,004 | 1,308 | 960 | 3 | 0 |
| 3 | 870 | 953 | 1,000 | 1,843 | 952 | 3 | 0 |
| 4 | 883 | 959 | 995 | 1,091 | 958 | 3 | 0 |
| 5 | 874 | 955 | 997 | 1,066 | 953 | 3 | 0 |

Across the five runs including warmup and instrumentation, macOS reported 38.42 s elapsed, 38.38 s user CPU, 0.04 s system CPU and 6,029,312 bytes maximum resident set size. The workload includes 120 fighter bodies with 112 anatomical joints, physical grip constraints and Skyline props. Awake-body counts, live constraint counts, contact pairs, allocated bytes and isolated solver time remain unmeasured. Production worker scheduling and socket delivery require wall-clock tests.

## Snapshot traffic

The encoder produced 15,686,752 bytes per recipient across 3,600 snapshots in each run. Dividing by 120 simulated seconds gives 130,722.93 B/s per player and 1,045,783.47 B/s for eight recipients. These are decimal units. Lobby and control messages, WebSocket framing, TCP/TLS and retransmits require a separate wire measurement. The native timing and traffic results support retaining Rapier for this workload. The browser frame-time matrix, worst-case scenarios, optimized traffic and endurance gates remain open.
