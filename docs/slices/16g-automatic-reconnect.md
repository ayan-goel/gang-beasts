# Automatic match reconnect

When an active match socket closes, the browser removes the old arena and clears its controls. A recovering player sees a reconnect status and can leave. Retries start after 250 ms, then wait 500 ms, 1 s and at most 2 s between attempts. One 20-second deadline covers the entire outage, including waiting for the replacement socket and map handshake. A failed recovery returns to the existing room recovery screen. The authoritative seat deadline still runs on the server from the time it observes the disconnect.

An automatic retry carries the ticket issued to that tab when it last explicitly connected. A manual connection rotates the ticket. The server accepts an automatic retry only for a disconnected, unexpired seat with the same session and ticket. Each accepted socket receives a new connection generation, so input and cleanup from an older socket cannot affect the current fighter. Consecutive retries retain the ticket in case the replacement transport closes before Welcome arrives.

After a successful resume the renderer loads the current map, receives the current server snapshot and sends the existing round acknowledgement. The new input sampler starts neutral. If the server has already returned the room to the lobby, it confirms that state explicitly. Ordinary pre-match lobby disconnects keep their existing manual reconnect control. Navigation cancels local retry timers. See the [wire records](../protocol.md) and [server grace rules](16e-disconnect-grace.md).

## Verification

The native admission test covers an obsolete ticket after takeover, a still-connected seat, a different session, repeated retries with the same ticket and refusal at exact expiry. Browser unit tests advance a simulated clock through backoff, an unfinished handshake, cancellation and protocol errors.

The Chromium scenario starts two fighters, holds a movement key, drops the host connection and refuses its first retry. It checks the host’s fighter assignment, round acknowledgement, score and neutral first movement after restoring the connection. A new keypress then produces movement. Opening the room in another tab transfers control to that tab. During the full retry window the former tab gets no fighter assignment and ultimately shows its manual reconnect button. [Silent-outage detection](16h-silent-match-outages.md) also starts recovery when the socket stays open.
