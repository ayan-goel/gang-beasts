# Skyline fighter rosters

Skyline accepts two through eight fighters in one native Rapier world. Each fighter owns a fifteen-body rig, two hand controllers, vitals, grip stamina and movement state. Slot order selects the authored spawn and remains stable through round reconstruction. All active controllers submit their forces before the shared physics step. The two-player constructor still selects the first two spawns.

## Contact and round rules

A hand can grip any other fighter or an eligible arena surface. The nearest unobstructed surface wins. Equal distances resolve in fighter slot and body-part order, followed by map ID. Elimination releases both outgoing and incoming grips before disabling the eliminated rig.

A strike can register one hit per victim. Multiple contacts on the same victim select the strongest qualifying hit, with body-part order breaking damage ties. Damage resolves after all hands have been evaluated. Scoring considers the whole roster after everyone crossing the kill plane this tick has been eliminated.

## Verification

The roster tests cover every supported size, reject invalid counts, check authored spawn coordinates, and exercise seven eliminations in one tick. Slot eight keeps its point through the next round, and a match reset clears all eight scores. A three-fighter physics fixture checks a real punch, grip and elimination release between the first and third slots. The existing two-fighter control and contact tests also run against this simulation.

This slice covers the native game. The public lobby still starts two-player matches. End-to-end eight-player play, server tick budgets and human movement feel have separate acceptance criteria.
