# Rematch in the same private room

After a fighter earns the third point, the worker completes the normal results interval and publishes MatchResults. The current host sees **Play again**. Choosing it returns everyone to the same lobby with their names, colors and room code. Participants ready up again, so a rematch starts with the roster that actually intends to play. The browser replaces its socket and disposes the previous scene without reloading the page.

The server checks current membership, socket generation, host authority, match identity and the latest worker phase under the directory lock. A guest or a host in an unfinished match cannot end it. The host permission can change while results are visible. Starting a new match revalidates the roster and match serial after worker allocation. Rejected allocations consume no serial.

## Round identity

The high 32 bits of the network round epoch identify the match within this room. The low 32 bits are the displayed round number, beginning at one. The first match has serial zero. Later matches have fresh input, physics, scores and interpolation history even though the scoreboard starts at round one again. The server rejects commands bound to an old epoch. The original snapshot layout and first-match recordings retain their existing encodings.

## Verification

A native room fixture opens the second match and rejects the first match’s round-one input before accepting input for its own epoch. Admission fixtures check early return requests. Chromium plays full matches by driving a fighter off the roof with the keyboard, checks that a guest cannot return everyone to the lobby, and uses the host action to replay. Two independent Chromium profiles on arm64 macOS 26.6.2 completed two consecutive first-to-three matches and entered a third on loopback. All six ring-outs used ordinary keyboard input. On both restarts the invite and both page time origins stayed the same, both previous WebGL contexts reported context loss, the host had one open game socket, and both browsers showed round one, zero scores and active gameplay. The new match epoch exceeded the last round of the previous match.

Run `npm --prefix web run test:e2e -- online-match.spec.ts --grep "two rematches" --workers=1` against the local service. The complete Chromium suite passed all 71 tests in 8.4 minutes with one test worker. This is the default because local physics playback and eight simultaneously rendered online clients share one machine. An earlier two-worker run timed out in the eight-client and local scenarios, and subsequent room starts encountered the occupied server. The eight-client fixture now releases its test seats through authenticated leave requests during cleanup.

The same two-rematch scenario passed against the production bundle and native server in 1.8 minutes with Chromium 159.0.8154.6. The server advertised engine hash `2cfb39e40d97df8742af9e4eb177bc26055c60fb1a5a9a1fae4198954c256e71` and Skyline content version 1 with hash `5a7b22f8def2fe29714c1616a8cc8bddd8d9555f630bc854acc834ab5fc48da5`.

The 60-minute soak and 100-reset endurance checks are recorded separately in the [MVP acceptance record](../verification/acceptance.md).
