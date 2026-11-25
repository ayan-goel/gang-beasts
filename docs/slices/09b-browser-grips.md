# Opponent grips in the browser

In Local sparring, hold the hand control through the punch and keep the hand near the other fighter. The inspector shows the gripping hand and the opponent’s actual body region. Reading the four native grip slots at the inspector’s 10 Hz refresh leaves the render loop and simulation rate independent. Releasing a key, moving focus out of the canvas, pausing or resetting clears the corresponding input.

## Verification

`npm --prefix web run test:e2e -- duel-grips.spec.ts duel.spec.ts` exercises the shared keyboard, a held grip with a free-hand punch, release, focus loss and reset. The native/WASM trace compares all 30 rigid bodies and the four grip target regions over 241 frames, including a reciprocal grip followed by separate releases. The checked tolerance for each body-state field is 0.00001. The fixture can be reproduced natively with `cargo run --locked --quiet --release -p brawler-engine --example duel_trace`.
