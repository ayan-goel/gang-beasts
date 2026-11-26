use super::*;
use serde_json::json;
use std::{collections::HashSet, time::Duration};

pub(super) struct Measurements {
    props: HashSet<ColliderHandle>,
    limbs: HashSet<ColliderHandle>,
    times: Vec<Duration>,
    min_awake: usize,
    total_awake: usize,
    max_contacts: usize,
    max_crate_contacts: usize,
    limb_contact_ticks: usize,
    grip_ticks: usize,
    max_speed: f32,
    max_joints: usize,
}

impl Measurements {
    pub fn new(game: &Duel, crates: &[RigidBodyHandle; 100]) -> Self {
        let collider = |body: RigidBodyHandle| game.world.bodies[body].colliders()[0];
        Self {
            props: crates.iter().copied().map(collider).collect(),
            limbs: game
                .fighters
                .iter()
                .flat_map(|f| f.bodies)
                .map(collider)
                .collect(),
            times: Vec::new(),
            min_awake: 100,
            total_awake: 0,
            max_contacts: 0,
            max_crate_contacts: 0,
            limb_contact_ticks: 0,
            grip_ticks: 0,
            max_speed: 0.,
            max_joints: 0,
        }
    }

    pub fn observe(
        &mut self,
        game: &Duel,
        crates: &[RigidBodyHandle; 100],
        duration: Option<Duration>,
    ) {
        let world = &game.world;
        for body in crates
            .iter()
            .chain(game.fighters.iter().flat_map(|f| &f.bodies))
        {
            let body = &world.bodies[*body];
            assert!(body.is_enabled() && body.is_dynamic());
            assert!(
                body.translation()
                    .iter()
                    .chain(body.rotation().coords.iter())
                    .chain(body.linvel())
                    .chain(body.angvel())
                    .all(|v| v.is_finite())
            );
            self.max_speed = self.max_speed.max(body.linvel().norm());
        }
        let Some(duration) = duration else {
            return;
        };
        self.times.push(duration);
        let awake = crates
            .iter()
            .filter(|&&h| !world.bodies[h].is_sleeping())
            .count();
        self.total_awake += awake;
        self.min_awake = self.min_awake.min(awake);
        let mut pairs = 0;
        let mut crate_pairs = 0;
        let mut limb_contact = false;
        for pair in world
            .narrow_phase
            .contact_pairs()
            .filter(|pair| pair.has_any_active_contact)
        {
            pairs += 1;
            let (a, b) = (pair.collider1, pair.collider2);
            let (a_prop, b_prop) = (self.props.contains(&a), self.props.contains(&b));
            crate_pairs += usize::from(a_prop && b_prop);
            limb_contact |= a_prop && self.limbs.contains(&b) || b_prop && self.limbs.contains(&a);
        }
        self.max_contacts = self.max_contacts.max(pairs);
        self.max_crate_contacts = self.max_crate_contacts.max(crate_pairs);
        self.limb_contact_ticks += usize::from(limb_contact);
        let held = FighterId::ALL.into_iter().any(|id| {
            [Hand::Left, Hand::Right]
                .into_iter()
                .any(|h| game.arena_grip(id, h).is_some_and(|obj| obj >= 10))
        });
        self.grip_ticks += usize::from(held);
        self.max_joints = self.max_joints.max(world.impulse_joints.len());
    }

    pub fn finish(mut self, measured: usize) {
        self.times.sort_unstable();
        assert_eq!(self.times.len(), measured);
        let percentile = |p: usize| self.times[(measured * p).div_ceil(100) - 1].as_micros();
        let over = self
            .times
            .iter()
            .filter(|t| t.as_nanos() * 60 > 1_000_000_000)
            .count();
        eprintln!(
            "prop_profile {}",
            json!({
                "samples":measured, "p50_us":percentile(50),"p95_us":percentile(95),"p99_us":percentile(99),
                "worst_us":percentile(100),"missed":over, "min_awake_crates":self.min_awake,
                "mean_awake_crates":self.total_awake as f64/measured as f64, "peak_contacts":self.max_contacts,
                "peak_extra_crate_pairs":self.max_crate_contacts,"fighter_crate_ticks":self.limb_contact_ticks,
                "extra_crate_grip_ticks":self.grip_ticks,"peak_joints":self.max_joints,"peak_speed_m_s":self.max_speed
            })
        );
        assert!(self.total_awake >= 75 * measured && self.max_crate_contacts >= 50);
        assert!(self.limb_contact_ticks > 0 && self.grip_ticks > 0);
        assert!(self.max_speed < 30.);
    }
}
