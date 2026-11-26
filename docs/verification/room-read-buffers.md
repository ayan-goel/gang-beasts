# Live room WebSocket read buffers

Each authenticated game socket accepts at most 4,096 bytes per inbound frame or message. The server previously used the networking library’s default initial read capacity of 128 KiB per connection. Eight players therefore reserved 1 MiB for reading. The server now initializes each read buffer at 4 KiB, reducing that reservation to 32 KiB for a full room, a difference of 992 KiB. The existing immediate write flush, 8 KiB write limit and input/message checks still apply. A frame may arrive across multiple reads. [Socket setup](../../crates/server/src/admission/socket.rs)

## Heap comparison

Two separate 90-second release servers on the reference M3 Pro ran the same eight-client combat, reconnect and rematch workload. At 18, 46 and 74 seconds from launching each pilot, `ps -o rss=` sampled the dedicated server and macOS `heap --noContent -s` sampled its allocation-size classes. The probes ran on separate pilots to keep heap inspection out of the hour-long measurements.

| Pilot time | Previous RSS | Revised RSS | Previous / revised live 128 KiB blocks |
| --- | ---: | ---: | ---: |
| 18 s | 8,512 KiB | 7,728 KiB | 8 / 0 |
| 46 s | 9,152 KiB | 8,352 KiB | 8 / 0 |
| 74 s | 10,176 KiB | 8,688 KiB | 8 / 0 |

All three previous samples contained eight live 128 KiB blocks. All three revised samples contained zero. Both pilots passed the eight-seat input, snapshot, combat, resume and rematch checks. All 41 server tests, workspace Clippy, workspace formatting, the workload tests, memory calculation tests and web type checking passed. The revised pilot used server binary SHA-256 `92e98123295184cea7faa553ae3139edc1d33976420058e5d29e97b932a7035a`. Future endurance captures record that digest alongside the engine and map identities, and record the host’s phase and arena tick with each minute of RSS.

The sample comparison verifies removal of the per-seat reservation. A smaller allocation can still leave resident pages under the allocator or OS. The cause of the original hour’s median spread requires separate evidence. [Live endurance capture and acceptance rule](live-endurance.md)
