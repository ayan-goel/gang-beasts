# Pulling back onto Skyline

While holding a rooftop ledge, hold the jump control to pull up. Coral uses Space and Blue uses Enter. Releasing jump interrupts the effort. The hand controls still govern voluntary grip release. A successful pull returns the fighter to ordinary walking and get-up control.

## Physics and eligibility

The engine derives the lip and inward direction from the stored grip anchor and the actual supporting box. Only an attached, enabled, fixed ledge can brace a pull. A walkway due to detach during the next solve has already lost eligibility for that solve. A fighter must start at or below the rim with stamina remaining. A prop or opponent grip supplies no climbing assistance.

While the pelvis is below the rim, the arm and knee targets tuck and the controller raises the pelvis on the outside of the wall. Once it clears the top by 20 cm, the controller directs the shins and torso inward. The solver retains the physical grip throughout. Automatic release requires pelvis clearance, at least 22 cm inward depth and upward support from the selected roof on the torso or a leg. Only hands attached to that support are released. All character motion goes through bounded impulses, motors and contacts.

Every assisted tick checks two volumes against other live solids: the space above the intended landing and the outside path from hip height up to the lip. Occupied space suspends assistance, retains the grip and incurs only normal grip costs. A waiting input can resume once the obstacle clears. The solver still determines whether a grip can sustain the hanging load.

## Calibration

The versioned grip configuration sets a 760 N ceiling for one hand, 960 N for two hands on the same supporting surface, and a 2.2 m/s desired approach speed limit. The force budget uses the mass of the climber. An attached passenger adds physical resistance. Pulling costs 6 stamina points per assisted second in addition to the ordinary grip cost. An attempt can drive at most 240 ticks before the player must release and press again. Supported get-up and locomotion retain their existing calibration.

## Verification

| Scenario | Evidence |
| --- | --- |
| One-hand side catch | Starts hanging below the north edge, pulls up, reaches foot support and walks over one metre further onto the roof. Pelvis displacement stays below 24 cm each tick. |
| Two-hand catch | Both hands initially hold the north lip. The fighter returns to standing height with foot support. |
| West walkway in a running round | Ordinary input produces the scheduled collapse and catch. Pulling on tick 7320 recovers onto the main deck. The Chromium fixture advances the approach, waits for an actual Space keydown, then observes the live match and scores. |
| Blocked landing | A low ceiling and a second fighter each block the pull. For 45 ticks, Coral's full physical state and stamina match the hanging control. Removing the obstruction lets the held input resume. |
| Interrupted effort | Clearing the jump input after 15 assisted ticks leaves the fighter hanging from the same hand. |
| Panel failure | The engine denies assistance in the detachment tick and on the detached panel. |
| Passenger load | A second physical fighter hangs from the climber. Both grips persist through the comparison window. After 18 ticks the solo climber is at least 6 cm higher. |

Native reproduction: `cargo test --release --locked -p brawler-engine --lib mantle_`. Browser reproduction: `npm --prefix web run test:e2e -- skyline-mantle.spec.ts`. Network latency cases and human rescue playtests remain open.
