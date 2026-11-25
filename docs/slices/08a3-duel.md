# Two fighters in one solver

## Ownership

The native duel owns one finite 12 m square platform, one Rapier world and one completed-tick counter. Each fighter owns fifteen body handles, fourteen anatomical joints, both hand timers, its gait, get-up and balance state, and its working motor configuration. Both read the same validated character definition and physics configuration. The fighters start facing one another with a 0.30 m lateral offset and 0.56 m longitudinal separation. This puts each left fist in range of the other torso.

The controller borrows the world for the duration of an update. Its frame contains the current fighter’s anatomical handles. The single-character playground and the duel both use the same gait, jump, recovery, arm targets, balance and implicit motors. The duel prepares the first fighter and then the second, steps Rapier once and publishes the completed tick. Reset reconstructs the solver and both characters.

## Verification

The native integration test presses each fighter’s left hand in a separate run. It requires contact with an opponent collider during the seven-tick strike window, over 0.01 N·s of solver impulse, and at least 5 mm of opponent chest displacement versus a no-input control. It verifies that the defender’s hand stays idle and reset restores the original pose. A second test jumps the first fighter while the second walks away, with separate height and travel assertions.

Run both with `cargo test --release --locked -p brawler-engine --test duel`. These fixtures establish simulation ownership, physical collision, and independent controls. Contact-qualified damage, grips, and online match authority have their own acceptance checks.
