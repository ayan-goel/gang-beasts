# Running the Skyline server

Start the native service with `cargo run --release --locked -p brawler-server`. The default bind address is `127.0.0.1:8080`; the startup line reports the actual bound port. Railway’s injected `PORT` binds the service to all IPv4 interfaces. Set `BRAWLER_LISTEN` to an IP and port to override it, or use `127.0.0.1:0` for an OS-assigned development port. Set `BRAWLER_WORKERS` to configure the fixed pool (default 1). Ctrl-C drains HTTP and joins the native workers.

| Request | Response |
| --- | --- |
| GET /healthz | 200 JSON with status ok while the process serves requests |
| GET /readyz | 200 JSON with ready true when another worker can admit a room. Returns 503 and ready false on capacity or scheduling overload. Readiness is served with Cache-Control no-store. |
| GET /api/maps | 200 JSON with protocolVersion, engineHash and maps. Each entry includes id, displayName, contentVersion and contentHash. Hashes come from the running build and the bundled map bytes. |

The server uses Axum on Tokio for HTTP. Worker threads own the simulation. Map validation and hashing run during service construction. The worker admission tests exercise saturation and recovery independently of HTTP. See [room workers](slices/14c-worker.md) and the [protocol](protocol.md). The [private admission API](slices/16a-admission.md) implements anonymous cookies, create, join and room metadata. Set `BRAWLER_ORIGIN` to the public browser origin (default `http://127.0.0.1:5173`). Production uses HTTPS. The browser uses the same origin for `/rooms/` and `/api/`. Vite’s dev and preview servers proxy `/api/` and WebSocket upgrades on `/ws` to `BRAWLER_API_TARGET` (default `http://127.0.0.1:8080`). For a production-build preview, run the native server with `BRAWLER_ORIGIN=http://127.0.0.1:4173`, run `npm --prefix web run preview` after building, and open `http://127.0.0.1:4173/rooms/`. The [GitHub deployment](deployment.md) uses a first-party HTTP rewrite and direct ticket-authenticated WebSocket upgrades. The [lobby socket](slices/16c-lobby-ready.md) carries authenticated readiness. Two through eight ready players can [start an authoritative match](slices/15f-online-roster.md) and [control their fighters](slices/15b-online-input.md). Public verification requires a deployed two-device match.

Run `cargo test --locked --release -p brawler-server --test api` for the real router's empty/full admission and catalog cases. A loopback check against the compiled executable exercised all three routes over TCP, checked the bundled map digest and observed a clean Ctrl-C exit. The map hash was 5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5 for content version 1.

The [disconnect timer](slices/16e-disconnect-grace.md) reserves a closed socket’s seat for 20 seconds. The service releases abandoned workers and retires empty rooms without an incoming request. A remaining competitor can return to the lobby after the results interval and start a new match with ready friends.

At match results, the current host can [return everyone to the lobby for a rematch](slices/16f-online-rematch.md). The room keeps its invite code and assigns a fresh input epoch to the next match.
