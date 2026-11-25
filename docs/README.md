# Engineering documentation

[Technical specification](../TECHNICAL_SPEC.md) is the canonical product and architecture contract. These documents record implementation choices and evidence; they do not redefine acceptance.

- [Implementation plan](implementation-plan.md): ordered slices and their gates.
- [Acceptance record](verification/acceptance.md): evidence against all MVP release requirements.
- [Playground controls](slices/03-playground-controls.md): contracts, review corrections and verification.
- [Foundation evidence](verification/foundation.md): native/WASM tests, build identity and browser smoke.
- [Decisions](decisions/): numbered records of context, alternatives, and consequences.
- [Slices](slices/): behavior, contracts, tests, and remaining work for each completed increment.
- [Verification](verification/): reproducible commands and measured results.

Every implementation commit updates the relevant slice record. Significant choices receive an ADR. Changes to normative behavior update the root specification in the same commit. Historical evidence remains dated and is never promoted to a current guarantee.
