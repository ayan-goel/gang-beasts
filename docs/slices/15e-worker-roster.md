# Online match worker rosters

A match reserves one worker and creates a bounded input channel for each participant. The requested roster size is validated before reservation. A room step drains each active slot in native fighter order, advances the shared world once and publishes a full snapshot every second tick. Published snapshots carry all fighter and visible arena bodies. Each recipient receives its own frozen input acknowledgement. Spectators receive the same world with the acknowledgement flag cleared.

The worker integration test creates an eight-player room, advances through countdown and sends eight distinct input sequences. It checks every recipient acknowledgement, all 120 limb IDs and the nine starting arena bodies in the shared snapshot. Invalid sizes leave the worker available. The two-player room and worker fixtures cover queue saturation, expiry, socket replacement, capture stability and worker recycling.
