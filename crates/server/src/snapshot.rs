use crate::Room;
use brawler_engine::{Consciousness, Duel, FighterId, Hand, HandPhase, MeleePhase};
use brawler_protocol::snapshot::{
    ARENA_BASE, BodyPose, FighterState, SharedSnapshot, Snapshot, SnapshotError, limb_id,
};

const SLOTS: [FighterId; 2] = [FighterId::First, FighterId::Second];

pub struct RoomFrame {
    world: SharedSnapshot,
    applied: [Option<u32>; 2],
}

impl RoomFrame {
    pub fn for_recipient(&self, id: FighterId) -> Vec<u8> {
        self.world.for_recipient(self.applied[id as usize])
    }
}

impl Room {
    pub fn snapshot_due(&self) -> bool {
        self.tick() > 0 && self.tick().is_multiple_of(2)
    }

    pub fn capture(&self) -> Result<RoomFrame, SnapshotError> {
        let game = self.game();
        let m = game.match_state().expect("Skyline room owns a match");
        let mut bodies = Vec::new();
        for id in SLOTS {
            bodies.extend(game.body_states(id).into_iter().map(|b| BodyPose {
                id: limb_id(id as usize, b.part.index()),
                position: b.state.position,
                rotation: b.state.rotation,
            }));
        }
        bodies.extend(
            game.arena_states()
                .into_iter()
                .filter(|b| b.visible)
                .map(|b| BodyPose {
                    id: ARENA_BASE + u32::from(b.id),
                    position: b.position,
                    rotation: b.rotation,
                }),
        );
        let snapshot = Snapshot {
            tick: self.tick() as u32,
            applied: None,
            round: m.round_id,
            number: m.round_number,
            arena_tick: game.tick() as u32,
            remaining: m.remaining_ticks,
            winner: m.winner,
            phase: match m.phase {
                MeleePhase::Countdown => 0,
                MeleePhase::Playing => 1,
                MeleePhase::RoundResults => 2,
                MeleePhase::MatchResults => 3,
            },
            bodies,
            fighters: SLOTS
                .into_iter()
                .map(|id| fighter(game, id, m.scores()[id as usize]))
                .collect(),
        };
        Ok(RoomFrame {
            world: snapshot.encode_shared()?,
            applied: SLOTS.map(|id| self.applied(id)),
        })
    }
}

fn fighter(game: &Duel, id: FighterId, score: u8) -> FighterState {
    let vitals = game.vitals(id);
    let hands = [Hand::Left, Hand::Right];
    FighterState {
        consciousness: match vitals.phase {
            Consciousness::Active => 0,
            Consciousness::KnockedOut => 1,
            Consciousness::Recovering => 2,
            Consciousness::Eliminated => 3,
        },
        score,
        health: vitals.health,
        stamina: game.stamina(id),
        hands: hands.map(|h| match game.hand_state(id, h).phase {
            HandPhase::Idle => 0,
            HandPhase::WindUp => 1,
            HandPhase::Strike => 2,
            HandPhase::Recovery => 3,
            HandPhase::Reach => 4,
            HandPhase::Gripping => 5,
        }),
        grips: hands.map(|h| {
            if let Some(target) = game.arena_grip(id, h) {
                return ARENA_BASE + u32::from(target);
            }
            game.grip(id, h)
                .map_or(0, |g| limb_id(g.target as usize, g.region.index()))
        }),
    }
}
