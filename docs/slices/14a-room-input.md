# Headless Skyline room

The server crate owns one native Skyline match. Its room clock counts every completed step from creation, including countdown and results. The arena clock counts playing ticks and starts again on round reconstruction. The worker receives immutable world access for observation. Only the room can mutate the match.

## Input lifecycle

| Decision | Reason and effect |
| --- | --- |
| Bind a submission to a fighter and round outside the input record | The eventual authenticated session determines the slot. An ordered transport must establish the new round before forwarding input and fence replaced sockets. The in-process API expects a trusted caller. |
| Admit eight pending records per fighter | Allows two full browser sampling bursts. An overflowing enqueue returns an error without reserving its sequence or refreshing its timeout. |
| Accept a modular forward gap of 1–120 | A two-second sequence window at the ordinary sample rate tolerates short gaps while rejecting replay and distant jumps. A new round starts a new sequence namespace. The first sequence may be zero. |
| Drain in stable fighter and sequence order | Short taps and repeated edges reach the engine before one solver tick. The engine owns combat cooldowns. The client tick cannot accelerate or rewind the world. |
| Publish the most recently consumed sequence after stepping | Each recipient gets its own optional acknowledgement. Enqueue, rejection, and cancellation do not advance it. |
| Expire a hold after 15 ticks without acceptance | The next solver tick starts neutral. Duplicate frames cannot keep a physical grip alive. Fresh movement and lift levels resume on receipt. Jump and hand activations require a new press edge. |
| Cancel on detach or round end | Detach releases outgoing grips and clears buffered input. Reattachment in the same epoch preserves the sequence watermark. Reconstruction clears acknowledgement and sequence history and rejects the retired epoch. |

Run `cargo test --locked --release -p brawler-server`. The fixtures compare every body of both fighters with direct engine controls tick by tick, including a short tap, a wrapped sequence, running and timeout. Further fixtures send Coral off the roof, verify Blue scores once, reject input from the old round, and keep sending duplicate heartbeats while a real hand-to-crate joint expires.

This slice accepts in-process binary submissions for two fighters. HTTP and WebSocket transport, identity and session ownership, disconnect grace expiry, eight-player physics and worker scheduling need their own integration evidence. Jump, run, hands and lift use the engine controls available today. Reserved future actions and headbutt have no gameplay binding in this slice.

## State publication

The room makes a full capture available after every second completed step. Explicit captures can also serve a join or resync on an odd tick. Every pose, vital, grip, score, map object and input acknowledgement comes from the same completed state. The server encodes the shared world once, freezes both recipients' applied sequence numbers, and copies the record with only the recipient acknowledgement fields changed when preparing delivery. The browser reconstructs absent arena bodies as retired and selects the warning from the agreed map schedule and completed arena tick.

The native capture test compares all 30 fighter transforms against the room, byte for byte. The Chromium fixture reads five frames from the native server example: initial countdown, a southern crate grasp, physical ring-out, next-round reconstruction and west-panel retirement. The transport for this fixture is the example's stdout. The live socket and interpolation have separate acceptance gates.

Run `cargo test --locked --release -p brawler-server --test snapshot` and `npm --prefix web run test:e2e -- room-snapshot.spec.ts`.
