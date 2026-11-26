# Regional punch responsiveness

A focused private-room fighter sends the first hand press immediately on a fresh snapshot. Both hands and jump share the existing allowance of one expedited send per publication. It expires after 100 ms and clears on focus loss or a round change. Pressing a second control before the next publication preserves the ordered input edges. Keyboard and mouse feed the same per-hand input state. The server still controls punch acceptance, windup, contact, damage, cooldown and the transition from a held hand into a grab. [Controls](../../web/src/online/RemoteControls.ts) · [Simultaneous-input checks](../../web/src/online/RemoteControls.test.ts)

## Measurement

The test creates 20 independent two-player rooms per run. In each room, the host taps both hands separately from a standing pose. Hand order alternates, and each hand gets ten keyboard taps and ten mouse clicks. The other hand must return to idle before the next trial. Both drawn hands must settle within 6 mm of their resting reference for 250 ms. The host match socket uses 60 ms of configured round-trip delay with up to 5 ms variation in each direction. The guest stays on direct loopback.

For each tap the observer records three times from the trusted browser input event.

- The outgoing input packet carrying the chosen hand's press.
- The first server snapshot acknowledging that sequence and reporting windup or strike on that hand.
- The first body draw with that hand at least 3 cm from its pre-input position in chest-local coordinates.

The draw measurement captures the onset of the physical punching motion. Hit confirmation and display scanout are separate measurements. The release check computes nearest-rank p95 independently for each hand. An unanswered tap fails the check. [Browser scenario](../../web/tests/punch-latency.spec.ts) · [Draw observer](../../web/tests/support/punchDraw.ts)

From the repository root, run `cd web && BRAWLER_CHECK_REGIONAL_PUNCH=1 npm run test:e2e -- punch-latency.spec.ts` on an idle host to test the production build against the 150 ms threshold. The ordinary browser check validates the packet, server state and drawn motion at every tap.

## Production results

Two Chromium 159.0.8154.6 clients used ANGLE Metal on an Apple M3 Pro running macOS 26.6.2. The test observed a real server-accepted punch and subsequent physical motion of the named hand for every tap. Each column contains 20 left-hand and 20 right-hand trials. The baseline failed the 150 ms target. Both hands passed in each of the two final runs. [Per-tap measurements, RTT samples and build fingerprints](../verification/regional-punch-feedback.jsonl)

| Input to observed event, p95 | Baseline | Final run 1 | Final run 2 |
| --- | ---: | ---: | ---: |
| Left press sent | 38.4 ms | 0.7 ms | 0.8 ms |
| Left punch accepted in a received snapshot | 133.4 ms | 101.7 ms | 98.2 ms |
| Left hand first drawn 3 cm from rest | 176.9 ms | 146.8 ms | 145.4 ms |
| Right press sent | 32.7 ms | 0.6 ms | 0.5 ms |
| Right punch accepted in a received snapshot | 131.3 ms | 90.9 ms | 98.8 ms |
| Right hand first drawn 3 cm from rest | 165.3 ms | 147.3 ms | 146.4 ms |

The timing gate applies to standing punches on the measured regional profile. Human feel, other moves and connections experiencing packet loss remain separate acceptance items.

## Close combat

At the same regional profile, the host walked toward the other fighter using ordinary keyboard controls, caught a limb, and punched with the free hand. Both clients observed the free hand enter its strike phase while the original limb remained attached. The roles of the left and right hand were tested separately. The left-hand hold also persisted through a guest reload. Both views cleared the grip on release. The tests check RTT on the host's fighting connection. [Two-client grip checks](../../web/tests/online-grips.spec.ts)

The standing-jump release check on the punch build measured 148.3 ms at p95 across 20 fresh rooms.
