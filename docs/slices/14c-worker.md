# Dedicated room workers

The native server starts a configured pool of 1–64 long-lived OS threads. The current allocation is one active room per worker. Opening a room reserves a free worker, constructs Skyline on that thread, and returns its handle after the initial snapshot is available. Closing the last handle ends that room. The next occupant gets a fresh world, publication slot and command channels. Pool shutdown wakes and joins the workers. The thread limit bounds allocation; capacity on a particular host still needs measurement.

## Timing and admission

The monotonic clock derives elapsed ticks from nanoseconds since room startup. The first step is due at 16,666,667 ns and the sixtieth at one second. A pass consumes at most four outstanding steps and yields if more remain. Every consumed step advances the ordinary engine once. The worker counts delayed passes and suspends new room admission while any worker reports a debt of at least one second. On-time publication happens every two completed ticks. Readers obtain an immutable reference to the latest complete frame and can skip superseded frames. The physics worker never takes a reader lock while stepping or copying a network packet.

## Commands

Each fighter has a separate eight-command channel. A sender receives a busy result when its channel fills. The worker drains at most eight commands per fighter per tick, then applies the room's sequence and round rules. Enqueue success means the command reached the channel. The later per-recipient acknowledgement identifies application. Lifecycle commands are ordered with inputs; a caller must handle a busy result before treating detach or attach as complete.

Commands carry their monotonic receipt instant. The worker rejects a record that waited at least 250 ms and clears prior held input when its accepted receipt ages out, including after a stalled scheduling pass. The room also enforces its 15-simulation-tick expiry. Duplicate, invalid-epoch and expired records cannot renew either deadline.

## Verification

Run `cargo test --locked --release -p brawler-server`. A synthetic one-second stall checks exact deadlines and catch-up without skipping any step. Channel fixtures fill Coral's queue while Blue submits, reject expired and replayed commands, and disconnect with pending input. Live thread tests advance two rooms independently, receive an applied action through a real-time publication, recycle a worker with a fresh scoreboard and acknowledgement, and reject commands after pool shutdown. These results establish scheduling behavior. Host capacity, external network delivery, session fencing and endurance require additional tests.
