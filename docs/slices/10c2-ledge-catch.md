# Catching and hanging from an exposed roof edge

Holding a hand control through its reach phase lets the fighter aim at a nearby exposed upper lip. Each hand uses the shoulder anchor, bind-pose arm span and palm radius to select a reachable point on the moving geometry. Collision and the existing 25 mm contact test still decide acquisition. Turning, walking and falling continue during the attempt.

The reaching hand uses the existing arm torque budget to apply a bounded impulse toward the edge. An equal opposite impulse on the chest at the same point balances both linear and angular momentum. The two ordinary arm motors yield during this action. A free hand can keep reaching while the other is attached.

## Catch loads and slip

A lip adds 25 mm of linear travel in each constraint axis to the compliant grip as its maximum permitted slip. The hand can rotate freely. Load sampling sums the spring and travel-limit impulses on each axis before computing the resultant. The limit prevents the visual hand from swinging metres away from a lip that the simulation still considers held.

The 40 kg fixture supports its full weight on one hand. The initial roof and walkway preset used 24 accumulated overloaded ticks at 740 N. The [subsequent fall-arrest calibration](10e-ledge-rescue.md) extends that window to 72 ticks. A sustained 80 kg single-hand load still breaks within 90 ticks. Opponent, crate and vent holds use 650 N for eight ticks. Both policies use the same smoothing, stamina and release history.

## Evidence

- A native rig placed in an unsupported side-on fall starts outside the contact gap, catches, hangs for three seconds, then falls more than two metres after release. An out-of-range rig and a rig with both controls released fall past the lip.
- A second native fixture runs the regular Skyline schedule from spawn. Coral starts walking west on tick 6800, turns toward the deck on 7050, stops on 7078, and holds both hands from 7160. The west walkway detaches on tick 7200. A hand holds the main roof continuously from 7229 through 7320.
- The Chromium test runs the same input ticks in the managed match, checks hanging height, continuous attachment and a palm-center distance below 18 cm from the roof lip on every attached tick, then releases both hands and checks Blue's ring-out point.
- The load calibration suspends known masses from the same constrained spring and measures the actual solver reaction.

These checks cover a deliberate catch and voluntary release. One- and two-hand pull-up onto the roof, overhead clearance and obstructed rescues remain open.
