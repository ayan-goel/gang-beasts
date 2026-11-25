# Local sparring

Select **Local sparring** in the playground and resume. Coral and Blue share one native/WASM physics world. Each fighter receives separate movement, jump and hand inputs. Keyboard input belongs to the focused canvas. Leaving the canvas, hiding the tab, pausing or leaving the scene clears held inputs and buffered actions. Accepted punches retain their normal cooldown.

| Action | Coral | Blue |
| --- | --- | --- |
| Movement | WASD | Arrow keys |
| Run | Left Shift | Right Shift |
| Jump | Space | Enter |
| Left / right hand | J / K or left / right mouse | Comma / period |

## Presentation and runtime

Both meshes read their 15 completed body poses from the same simulation tick. The oblique camera centers the two pelvis positions and adjusts its distance for their separation and the viewport aspect ratio. The side panel counts hits for each player and identifies the latest attacker, defender, body region and computed damage points. The count describes sparring contacts. Match scores, health and knockouts have separate lifecycle rules.

The runtime consumes hit records after every simulation step, including catch-up steps between frames. The WASM boundary returns an owned 210-float body array per fighter. Hit records carry tick and attack ID as decimal strings to retain full 64-bit precision. Invalid fighter indices, hand indices and input types are rejected. Reset constructs both fighters anew and clears displayed hit history.

## Verification

The Chromium UI fixture punches as Coral, resets, punches as Blue and switches back to a single-world fixture. Another fixture traces simultaneous left punches, then a Coral jump and Blue walking. It compares all 420 body-state values at every tick from 0 to 130, together with both native hit events. The native script registers both hits at tick 68, each to the opposing chest for about 12.6 points. A keyboard fixture observes both fighters’ actual WASM input writes through simultaneous directions, right Shift and canvas blur.

Run the native reference with `cargo run --release --locked -p brawler-engine --example duel_trace`. Build WASM with `npm --prefix web run build` and run the browser checks with `npm --prefix web run test:e2e -- duel.spec.ts`.

On November 25, 2026 the release workspace passed 194 native checks. The two previously recorded stair tests failed. The production build, formatting, Clippy and 19 web unit tests passed. The full Chromium run passed 43 cases. After adding the separate keyboard-routing regression, all three sparring cases passed together. The two-rig reference comparison remained within 0.00001 for all 55,020 body-state values.
