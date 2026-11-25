# Running the Skyline server

Start the native service with `cargo run --release --locked -p brawler-server`. The default bind address is `127.0.0.1:8080`; the startup line reports the actual bound port. Set `BRAWLER_LISTEN` to an IP and port to override it, or use `127.0.0.1:0` for an OS-assigned development port. Set `BRAWLER_WORKERS` to configure the fixed pool (default 1). Ctrl-C drains HTTP and joins the native workers.

| Request | Response |
| --- | --- |
| GET /healthz | 200 JSON with status ok while the process serves requests |
| GET /readyz | 200 JSON with ready true when another worker can admit a room. Returns 503 and ready false on capacity or scheduling overload. Readiness is served with Cache-Control no-store. |
| GET /api/maps | 200 JSON with protocolVersion, engineHash and maps. Each entry includes id, displayName, contentVersion and contentHash. Hashes come from the running build and the bundled map bytes. |

The server uses Axum on Tokio for HTTP. Worker threads own the simulation. Map validation and hashing run during service construction. The worker admission tests exercise saturation and recovery independently of HTTP. See [room workers](slices/14c-worker.md) and the [protocol](protocol.md). Public deployment, private-room creation, session credentials and gameplay WebSockets are subsequent gates.

Run `cargo test --locked --release -p brawler-server --test api` for the real router's empty/full admission and catalog cases. A loopback check against the compiled executable exercised all three routes over TCP, checked the bundled map digest and observed a clean Ctrl-C exit. The map hash was 5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5 for content version 1.
