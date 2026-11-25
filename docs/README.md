# Engineering documentation

[Technical specification](../TECHNICAL_SPEC.md) is the canonical product and architecture contract. These documents record implementation choices and evidence; they do not redefine acceptance.

- [Implementation plan](implementation-plan.md): ordered slices and their gates.
- [Acceptance record](verification/acceptance.md): evidence against all MVP release requirements.
- [Playground controls](slices/03-playground-controls.md): contracts, review corrections and verification.
- [Recording and tuning](slices/03b-recording-tuning.md): bounded native/WASM reproduction and transactional configuration changes.
- [Native passive ragdoll](slices/04a-native-passive-ragdoll.md): anatomy, constraints, collision behavior and drop fixtures.
- [Ragdoll WASM contract](slices/04b1-ragdoll-wasm-contract.md): batched body/joint buffers, ownership and native/browser parity.
- [Ragdoll browser inspector](slices/04b2-ragdoll-browser-inspector.md): scene selection, solid geometry, joint inspection and resource cleanup.
- [Bounded pose motors](slices/05a-bounded-pose-motors.md): native relative control, physical effort budgets and conservation fixtures.
- [Native standing](slices/05b1-native-standing.md): support detection, solver-based pose control, bounded balance and physical standing/push fixtures.
- [Browser standing](slices/05b2-standing-browser.md): driven WASM ownership, motor targets, support/effort inspection and 60-second browser verification.
- [Native locomotion](slices/06a-native-locomotion.md): bounded movement assistance, procedural gait, support velocity, braking and motor relaxation.
- [Browser locomotion](slices/06b-browser-locomotion.md): movement input, native/WASM parity, live gait targets and input/resource lifecycle.
- [Moving-support transitions](slices/06c1-moving-supports.md): bounded native deck response, impact compliance and support removal.
- [Native slope gait](slices/06c2a-native-slopes.md): supported leg/sole targets and bounded braking on inclined planes.
- [Reversal braking](slices/06c2b1-reversals.md): tick-sampled direction changes, bounded braking, neutral key gaps and native/WASM evidence.
- [Stopping phases](slices/06c2b2-stopping.md): explicit moment-limited stopping, support-frame corrections, verification and remaining traversal failures.
- [Open terrain scenarios](verification/terrain-gaps.md): remaining slope, stair and stop-phase reproductions.
- [Foundation evidence](verification/foundation.md): native/WASM tests, build identity and browser smoke.
- [Decisions](decisions/): numbered records of context, alternatives, and consequences.
- [Slices](slices/): behavior, contracts, tests, and remaining work for each completed increment.
- [Verification](verification/): reproducible commands and measured results.

Every implementation commit updates the relevant slice record. Significant choices receive an ADR. Changes to normative behavior update the root specification in the same commit. Historical evidence remains dated and is never promoted to a current guarantee.
