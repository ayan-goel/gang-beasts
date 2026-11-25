# Private room admission

Skyline issues anonymous same-origin sessions. The browser bootstraps once before creating or joining a room. A cryptographically random six-character code identifies a lobby. Each browser receives its own 256-bit OS-generated credential in a Secure, HttpOnly, SameSite=Strict cookie with the __Host- prefix and path /. The registry stores its SHA-256 digest. The credential never appears in a response body, share URL or application log. Multiple tabs sharing one cookie represent one player.

## HTTP contract

All mutations require the configured public Origin and a JSON body of at most 4096 bytes. Routes return JSON with Cache-Control no-store. Errors use the shape `{"error":{"code":"..."}}`.

| Route | Input and result |
| --- | --- |
| POST /api/session | Send an empty JSON object. Issues a session cookie or refreshes the existing session's server-side expiry. The response is identical for both paths. |
| POST /api/rooms | Send name, optional color (0–7), optional capacity (2–8, default 8) and requestId (32 hex characters). Returns 201 with code, slot, host, name and color. Repeating the same request under the same session returns 200 and the same seat. A changed request or a second active room returns 409. |
| POST /api/rooms/{code}/join | Send name and optional color. Returns the player's seat, including their assigned color. A retry returns the same seat. Full rooms and occupied requested colors return 409. Codes accept either case. |
| GET /api/rooms/{code} | Returns code, phase, capacity and player count. Unknown or malformed codes return 404. |

Names are trimmed, limited to 24 Unicode scalar values and reject control characters. A session occupies one stable slot in one lobby. A collision when generating a code retries without touching the existing lobby. Idempotency keys are scoped to the authenticated creator. Authentication and mutation happen under one directory lock, with no physics work or I/O waits inside it.

Inactive sessions expire after 30 minutes. Cleanup transfers the host role to the lowest occupied slot, leaving other assignments intact. On observing an empty lobby, the directory starts a 60-second expiry timer. Cleanup runs on admission requests, at most once per second. The directory caps sessions at 512, lobbies at 64 and tracked peer IPs at 1024. Each direct TCP peer has a 120-request, one-minute budget across these routes. The service ignores forwarded IP headers. A production proxy needs a deployment-specific rate-limit arrangement before exposure. These initial caps are policy choices pending load tests.

## Verified scope

Run `cargo test --release --locked -p brawler-server`. Router tests cover safe create retries, rejected origin and invalid fields, two- and eight-slot admission, and a ninth player denied entry. Internal clock tests cover expiry, host transfer, empty cleanup and rate-window reset. Two independent Chromium contexts also created and joined one room against the compiled process on loopback. Both browsers received a secure HttpOnly cookie accessible to HTTP alone.

Admission reserves lobby seats. Worker startup on ready/start, live socket authentication and reconnecting an existing fighter still need integration. The native physics room currently supports two fighters.
