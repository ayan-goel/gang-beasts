# Grip stamina and load failure

## Timing and ownership

Each fighter starts with 100 stamina. At 60 Hz, charge 4 points/s for one attached hand or 7 points/s total for two. Charge the acquisition tick even when the solver has not yet loaded the new joint. An owner with both hands free regenerates 18 points/s up to the starting maximum. A conscious fighter retains an attached hand through physical get-up. The arm motor may change its pose while the hand remains constrained.

Each hand samples the norm of its three constraint motor impulses after the solve. Rapier 0.26 exposes the last solver substep’s impulses in N·s. Dividing by that substep’s duration gives an end-of-tick sample in N. The filter uses a time constant of 0.08 s and a fixed-tick weight of dt/(0.08 + dt). The sample excludes force transmitted through ordinary collisions or anatomical joints.

For each occupied hand, add 4 × max(0, filtered force / 392.4 N − 1) points/s. Cap the sum of the extra drains at 12 points/s. An overloaded hand accumulates one tick of strain per tick at or above 650 N and loses one accumulated tick on each unloaded or lower-force tick. Eight accumulated ticks break that joint. This preserves strain across short release gaps. At zero stamina all outgoing grips fail. A failed hand needs another accepted press to acquire again. The other fighter retains independent stamina and grip ownership.

## Reproduction

- `cargo test --locked --release -p brawler-engine --lib grip_budget` checks per-owner drain, recovery, the extra-drain ceiling, a single transient spike, unilateral overload and retained strain.
- `cargo test --locked --release -p brawler-engine --lib grip_joint` suspends a 40 kg rigid load on one or two production grip springs. After ten seconds at 2, 4 and 8 solver iterations the final per-hand forces are within 4 N of 392.4 N or 196.2 N respectively. A 140 kg one-spring load crosses the sustained-break threshold. These are weight calibrations. Articulated ledge rescues require their own fixtures.
- `cargo test --locked --release -p brawler-engine --test duel_grips` includes a two-hand hold on distinct opponent limbs with individual release, holds an opponent until exhaustion, checks joint removal and continued recovery while the original input stays down.
- `npm --prefix web run test:e2e -- duel-grips.spec.ts duel.spec.ts` checks visible owner-specific stamina and regeneration. The 241-frame recorded comparison also checks both stamina values after each step across native and browser builds.

The load and break thresholds are initial gameplay tuning. Multiplayer stress tests and playtesting during lifts and ledge rescues remain open.
