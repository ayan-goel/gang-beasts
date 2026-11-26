# Knockouts and physical recovery in local sparring

## Tick ordering

The duel resolves all qualifying strikes from the same physics tick before applying health changes. Each victim takes the sum of those strikes once. This allows simultaneous knockouts. Health regeneration begins 120 ticks after the last accepted damage and adds 8 points per second up to 100. All timings and health values are versioned in `config/consciousness.json`.

A knockout clears the fighter’s actions, walking intent and outgoing grip constraints. The incoming grips, rigid bodies, anatomical joints and velocities persist. Unconscious bodies receive no active motor torque, balancing or movement assistance. The first knockout lasts 180 ticks, subsequent knockouts add 60 ticks, and the duration caps at 720 ticks. During the next 30 ticks, health returns to 60 and the internal torque ceiling increases toward the standing preset. Body orientation selects the same arm and leg tuck as ordinary physical get-up. The completion of the ramp returns control even when the character is still physically on the floor.

Punches against a knocked-out or waking fighter still collide physically. During those two timed phases they award no damage and cannot restart the deadline. The first active tick accepts damage normally. Each transition emits one per-fighter event. Reset restores both health pools, the first-knockout timer and the initial spawn conditions.

## Reproduction

- `cargo test --release --locked -p brawler-engine --test duel_health` checks a real punch, victim identity, regeneration delay/rate and reset.
- `cargo test --release --locked -p brawler-engine --lib vitals::tests` exercises thirteen knockouts under continuous damage, checking every deadline through the twelve-second cap.
- `cargo test --release --locked -p brawler-engine --lib duel::health` checks physical motor disablement, bounded wake-up effort, input suppression, and independent incoming and outgoing grips. The grip cleanup fixture injects an already-resolved impact to isolate ownership.
- `npm --prefix web run test:e2e -- knockout.spec.ts` replays normal movement and punches from full health. Blue goes down at tick 396, starts waking at tick 576 and regains control at tick 606. Its pelvis rises from the floor to over 0.75 m by tick 781. The comparison covers every fighter body, health, phase and transition on all 781 ticks. A separate UI check verifies that a punch changes only the defender’s condition and that reset restores both fighters.

The current encounter covers punch-driven knockouts. The [fighter condition slice](17d-fighter-condition.md) adds presentation audio and two-client knockout replication. Hazards, environmental impact damage and human playtesting have separate gates.
