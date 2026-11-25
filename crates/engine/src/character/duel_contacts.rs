use super::{BodyPart, FighterId, duel_fighter::Fighter};
use crate::{
    Hand, HandPhase,
    combat::{BodyMotion, ImpactTuning},
    physics::PhysicsWorld,
};
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikeHit {
    pub tick: u64,
    pub attacker: FighterId,
    pub defender: FighterId,
    pub hand: Hand,
    pub attack_id: u64,
    pub region: BodyPart,
    pub closing_speed: f32,
    pub energy_j: f32,
    pub damage: f32,
}

pub(super) struct ContactFrame(Vec<[BodyMotion; 15]>);

impl ContactFrame {
    pub fn capture(world: &PhysicsWorld, fighters: &[Fighter]) -> Option<Self> {
        let striking = fighters.iter().any(|f| {
            [Hand::Left, Hand::Right]
                .into_iter()
                .any(|h| f.hands.state(h).phase == HandPhase::Strike)
        });
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
            for hand in [Hand::Left, Hand::Right] {
                let fighter = &fighters[attacker as usize];
                let state = fighter.hands.state(hand);
                if state.phase != HandPhase::Strike {
                    continue;
                }
                let fist = fighter.hand_collider(world, hand);
                let punch = self.0[attacker as usize][hand.body().index()];
                let heading = self.0[attacker as usize][BodyPart::Chest.index()].forward();
                let mut strongest: [Option<StrikeHit>; FighterId::ALL.len()] =
                    [None; FighterId::ALL.len()];
                for pair in world
                    .narrow_phase
                    .contact_pairs_with(fist)
                    .filter(|p| p.has_any_active_contact)
                {
                    let first = pair.collider1 == fist;
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
                    if fighter.last_hit[hand as usize][defender as usize] == state.attack_id {
                        continue;
                    }
                    let index = region.index();
                    let struck = self.0[defender as usize][index];
                    let Some(speed) = closing_speed(pair, first, heading, punch, struck) else {
                        continue;
                    };
                    let Some((energy_j, damage)) =
                        tuning.evaluate(speed, [punch.mass, struck.mass], region)
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
                            hand,
                            attack_id: state.attack_id,
                            region,
                            closing_speed: speed,
                            energy_j,
                            damage,
                        });
                    }
                }
                for hit in strongest.into_iter().flatten() {
                    fighters[attacker as usize].last_hit[hand as usize][hit.defender as usize] =
                        state.attack_id;
                    hits.push(hit);
                }
            }
        }
    }
}

fn closing_speed(
    pair: &ContactPair,
    fist_first: bool,
    heading: Vector<Real>,
    fist: BodyMotion,
    target: BodyMotion,
) -> Option<Real> {
    let mut fastest: Option<Real> = None;
    for manifold in &pair.manifolds {
        let normal = if fist_first {
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
            let (a, b) = if fist_first {
                (point.local_p1, point.local_p2)
            } else {
                (point.local_p2, point.local_p1)
            };
            let speed = (fist.point_velocity(a) - target.point_velocity(b)).dot(&normal);
            fastest = Some(fastest.map_or(speed, |old| old.max(speed)));
        }
    }
    fastest
}
