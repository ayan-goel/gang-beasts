use super::{BodyPart, FighterId, duel_fighter::Fighter};
use crate::{
    Hand, HandPhase,
    combat::{BodyMotion, ImpactTuning},
    physics::PhysicsWorld,
};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrikeSource {
    Punch(Hand),
    Headbutt,
}

impl StrikeSource {
    const ALL: [Self; 3] = [
        Self::Punch(Hand::Left),
        Self::Punch(Hand::Right),
        Self::Headbutt,
    ];

    fn active(self, fighter: &Fighter) -> Option<(BodyPart, u64, f32)> {
        match self {
            Self::Punch(side) => {
                let state = fighter.hands.state(side);
                (state.phase == HandPhase::Strike).then_some((side.body(), state.attack_id, 1.))
            }
            Self::Headbutt => fighter
                .headbutt
                .strike()
                .map(|(id, scale)| (BodyPart::Head, id, scale)),
        }
    }
    pub fn hand(self) -> Option<Hand> {
        match self {
            Self::Punch(side) => Some(side),
            Self::Headbutt => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikeHit {
    pub tick: u64,
    pub attacker: FighterId,
    pub defender: FighterId,
    pub source: StrikeSource,
    pub attack_id: u64,
    pub region: BodyPart,
    pub closing_speed: f32,
    pub energy_j: f32,
    pub damage: f32,
}

pub(super) struct ContactFrame(Vec<[BodyMotion; 15]>);

impl ContactFrame {
    pub fn capture(world: &PhysicsWorld, fighters: &[Fighter]) -> Option<Self> {
        let striking = fighters
            .iter()
            .any(|f| StrikeSource::ALL.iter().any(|s| s.active(f).is_some()));
        striking.then(|| {
            Self(
                fighters
                    .iter()
                    .map(|fighter| fighter.bodies.map(|h| BodyMotion::sample(&world.bodies[h])))
                    .collect(),
            )
        })
    }

    pub fn resolve(
        &self,
        world: &PhysicsWorld,
        fighters: &mut [Fighter],
        tuning: &ImpactTuning,
        tick: u64,
        hits: &mut Vec<StrikeHit>,
    ) {
        for &attacker in &FighterId::ALL[..fighters.len()] {
            for (slot, source) in StrikeSource::ALL.into_iter().enumerate() {
                let fighter = &fighters[attacker as usize];
                let Some((limb, attack_id, scale)) = source.active(fighter) else {
                    continue;
                };
                let collider = world.bodies[fighter.bodies[limb.index()]].colliders()[0];
                let striker = self.0[attacker as usize][limb.index()];
                let heading = self.0[attacker as usize][BodyPart::Chest.index()].forward();
                let mut strongest: [Option<StrikeHit>; FighterId::ALL.len()] =
                    [None; FighterId::ALL.len()];
                for pair in world
                    .narrow_phase
                    .contact_pairs_with(collider)
                    .filter(|p| p.has_any_active_contact)
                {
                    let first = pair.collider1 == collider;
                    let other = if first {
                        pair.collider2
                    } else {
                        pair.collider1
                    };
                    let Some(body) = world.colliders[other].parent() else {
                        continue;
                    };
                    let Some((defender, region)) = Fighter::other_limb(fighters, attacker, body)
                    else {
                        continue;
                    };
                    if fighter.last_hit[slot][defender as usize] == attack_id {
                        continue;
                    }
                    let index = region.index();
                    let struck = self.0[defender as usize][index];
                    let Some(speed) = closing_speed(pair, first, heading, striker, struck) else {
                        continue;
                    };
                    let Some((energy_j, damage)) =
                        tuning.evaluate(speed, [striker.mass, struck.mass], region, scale)
                    else {
                        continue;
                    };
                    let best = &mut strongest[defender as usize];
                    if best.is_none_or(|hit| {
                        damage > hit.damage || (damage == hit.damage && index < hit.region.index())
                    }) {
                        *best = Some(StrikeHit {
                            tick,
                            attacker,
                            defender,
                            source,
                            attack_id,
                            region,
                            closing_speed: speed,
                            energy_j,
                            damage,
                        });
                    }
                }
                for hit in strongest.into_iter().flatten() {
                    fighters[attacker as usize].last_hit[slot][hit.defender as usize] = attack_id;
                    hits.push(hit);
                }
            }
        }
    }
}

fn closing_speed(
    pair: &ContactPair,
    attacker_first: bool,
    heading: Vector<Real>,
    attacker: BodyMotion,
    target: BodyMotion,
) -> Option<Real> {
    let mut fastest: Option<Real> = None;
    for manifold in &pair.manifolds {
        let normal = if attacker_first {
            manifold.data.normal
        } else {
            -manifold.data.normal
        };
        if heading.dot(&normal) <= 0.0 {
            continue;
        }
        for point in manifold.points.iter().filter(|p| p.data.impulse > 0.0) {
            // Each limb collider shares its body's local frame. Reconstruct both
            // point velocities before solve so the collision has not removed closing speed.
            let (a, b) = if attacker_first {
                (point.local_p1, point.local_p2)
            } else {
                (point.local_p2, point.local_p1)
            };
            let speed = (attacker.point_velocity(a) - target.point_velocity(b)).dot(&normal);
            fastest = Some(fastest.map_or(speed, |old| old.max(speed)));
        }
    }
    fastest
}
