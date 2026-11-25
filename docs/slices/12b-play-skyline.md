# Playable two-player Skyline match

Open `/play/` or choose **Play Skyline Melee** in the playground. The production build includes a static `play/index.html` entry for direct navigation. Both players share one keyboard. Controls, pause/resume, the round timer, warnings, score, knockout feedback and rematch are available in the game screen.

## World and presentation

Rust owns the round, visible bodies, transforms, scores, epoch, deadlines and hazard state. The renderer consumes the map manifest and fixed-order pose buffers. The façade below the roof has a corresponding native collider. Distant buildings and surface trim are original procedural scenery. A hidden detached panel also disappears from the scene. The HUD uses color, shape and text for player identity and displays a warning before each scheduled failure.

During countdown the camera frames the roof. During play it fits every visible fighter’s body centers with geometric margins in both screen dimensions. It expands immediately for a separating pair and eases toward a closer view. Eliminated bodies leave the group. The fixed view has separate portrait and landscape framing checks. Map collision queries and player-selected camera orbits remain open.

## Verification

- `npm --prefix web run test:e2e -- skyline.spec.ts` drives a real ring-out from the keyboard, checks the score and automatic round transition, and verifies a match-winning round followed by a rematch without reload. The three-round test accelerates the number of ordinary engine steps per render. The first ring-out uses the normal game clock.
- The WASM hazard fixture steps through the warning, both panel releases, falling motion and retirement, then verifies a rematch restores all nine object poses.
- `npm --prefix web test -- --run` includes group-framing checks with separated and falling fighters at three aspect ratios. Chromium captures the menu and live match at desktop size and checks access to the start control at 390 × 844.
- `npm --prefix web run build` produces the playground and the static game entry. `npm --prefix web run preview` serves the built bundle on loopback.

This screen uses the existing two physical ragdolls and local engine clock. Remote joining and server authority have separate acceptance gates.
