# Independent hand strikes and reach

## Player behavior

In the locomotion playground, each mouse button punches with its corresponding arm. J, K and the on-screen hand buttons provide equivalent inputs. Holding an input after the punch keeps that hand extended. Releasing it returns the arm to its gait pose. Sources combine independently, so a player holding J can release the left mouse button and continue reaching.

## Simulation contract

The first completed tick after an accepted press starts a five-tick wind-up. Ticks 6–12 drive the shoulder and elbow forward. Ticks 13–24 retract a tapped arm toward rest or blend a held arm directly into reach. On tick 25 it can reach, return to idle or start a fresh punch from a new press. The other arm runs its own clock. A tap between simulation steps survives until the next step. Focus loss clears unsampled input and held reach while preserving the cooldown of an accepted punch.

The rig assigns a fresh ID on acceptance. Reset starts a new sequence. The local hand inspector exposes an owned four-element unsigned 64-bit array containing phase and attack ID for the left hand and then the right. Idle carries ID zero. A network consumer will also need attacker and round identity.

The controller applies the authored pitch targets through the existing solver motors, capped at 45 N·m at each active shoulder and elbow. The arms’ gravity moment about the chest biases chest and supported ankle pitch by up to 0.20 radians. Each supported ankle carries its share. The base motor gains and ceilings still apply to those joints, and airborne feet supply no counterbalance. Automatic get-up takes control of the arms during recovery. Configuration changes are included in the engine build fingerprint.

## Verification

- The native timing fixture staggers the left and right presses by three ticks and verifies every phase. Input cancellation during a strike preserves its cooldown. Reset and disable exercise the two different lifecycle paths.
- The physical contact fixture places a gravity-neutral 2 kg dynamic target within reach of one hand. Each side must produce a solver contact impulse above 0.01 N·s during its strike window and leave the target moving forward faster than 0.2 m/s. Joint anchor separation stays below 5 cm throughout.
- The motor fixture narrows one authored shoulder limit and checks the target, the opposite arm’s motor ceiling and restoration of the gait ceilings.
- A two-hand hold originally tipped the chest at tick 137. The balance regression covers one or both hands for 20 seconds and staggered presses and releases from four settling times.
- The Chromium fixtures exercise simultaneous mouse buttons, keyboard and mouse overlap, focus loss and the on-screen hold button. The WASM fixture checks all 25 ticks of a tap, over 38 cm of simulated forward hand travel, rejected invalid inputs and the lifetime of a returned hand record. Another browser fixture holds both arms out for 1,200 ticks, checks chest height every tick and releases just the left hand.

Run the native cases with `cargo test --locked --release -p brawler-engine --lib strike_tests`. Rebuild with `npm --prefix web run build` and run `npm --prefix web run test:e2e -- strikes.spec.ts` for the browser cases. Damage, target selection, grips and the two-player fight remain open.

On November 25, 2026, the release workspace run passed 187 native tests. The two open stair-ascent tests failed. The production build, Clippy, formatting and 19 web unit tests passed. The full Chromium run passed 38 cases. After adding the separate 20-second WASM hold, all three strike cases passed in five consecutive repetitions each.
