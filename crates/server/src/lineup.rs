use crate::Room;
use brawler_engine::{CharacterError, FighterId};
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Participant {
    pub member: u64,
    pub generation: u64,
    pub slot: u8,
    pub name: String,
    pub color: u8,
}

pub(crate) struct Lineup {
    pub active: Arc<[Participant]>,
    pending: Vec<Participant>,
    pub forfeits: u8,
}

impl Lineup {
    pub fn new(players: Vec<Participant>) -> Self {
        Self {
            active: players.into(),
            pending: Vec::new(),
            forfeits: 0,
        }
    }

    pub fn queue_for_next_round(&mut self, entrant: Participant) {
        if self.active.iter().any(|p| p.member == entrant.member) {
            return;
        }
        self.pending.retain(|p| p.slot != entrant.slot);
        self.pending.push(entrant);
    }

    pub fn cancel_wait(&mut self, generation: u64) {
        self.pending.retain(|p| p.generation != generation);
    }

    pub fn depart(&mut self, member: u64) {
        self.pending.retain(|p| p.member != member);
        if let Some(index) = self.active.iter().position(|p| p.member == member) {
            self.forfeits |= 1 << index;
        }
    }

    pub fn try_start_next_round(&mut self, room: &mut Room) -> Result<bool, CharacterError> {
        if self.pending.is_empty() || !room.game().match_state().unwrap().next_round_due() {
            return Ok(false);
        }
        let mut next: Vec<_> = self
            .active
            .iter()
            .enumerate()
            .filter(|(i, _)| self.forfeits & (1 << i) == 0)
            .map(|(i, p)| (p.clone(), Some(FighterId::ALL[i])))
            .chain(self.pending.iter().map(|p| (p.clone(), None)))
            .collect();
        if next.len() < 2 {
            return Ok(false);
        }
        next.sort_by_key(|(p, _)| p.slot);
        let (players, previous): (Vec<_>, Vec<_>) = next.into_iter().unzip();
        room.start_next_round(&previous)?;
        self.active = players.into();
        self.pending.clear();
        self.forfeits = 0;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(slot: u8, member: u64, generation: u64) -> Participant {
        Participant {
            slot,
            member,
            generation,
            name: format!("Player {member}"),
            color: slot,
        }
    }

    #[test]
    fn a_reused_seat_and_a_replaced_waiting_socket_can_only_promote_the_current_occupant() {
        let mut roster = Lineup::new(vec![player(1, 10, 10), player(5, 11, 11)]);
        let former = roster.active.clone();
        let mut room = Room::skyline().unwrap();
        roster.queue_for_next_round(player(5, 11, 20));
        roster.depart(10);
        roster.queue_for_next_round(player(1, 21, 21));
        roster.queue_for_next_round(player(1, 21, 22));
        roster.cancel_wait(21);
        roster.queue_for_next_round(player(7, 23, 23));
        roster.depart(23);
        roster.queue_for_next_round(player(3, 24, 24));
        roster.cancel_wait(24);
        for _ in 0..181 {
            assert!(!roster.try_start_next_round(&mut room).unwrap());
            room.step_with_forfeits(roster.forfeits);
        }
        for _ in 1..300 {
            room.step();
        }
        assert!(roster.try_start_next_round(&mut room).unwrap());
        assert!(!roster.try_start_next_round(&mut room).unwrap());
        assert_eq!(former[0].member, 10);
        assert_eq!(
            roster
                .active
                .iter()
                .map(|p| (p.slot, p.member, p.generation))
                .collect::<Vec<_>>(),
            vec![(1, 21, 22), (5, 11, 11)]
        );
        assert_eq!(room.game().match_state().unwrap().scores(), &[0, 1]);
        assert_eq!(roster.forfeits, 0);
    }
    #[test]
    fn all_waiting_seats_enter_in_slot_order_after_the_winners_result() {
        let mut lineup = Lineup::new(vec![player(0, 10, 10), player(7, 20, 20)]);
        let mut room = Room::skyline().unwrap();
        lineup.depart(10);
        for slot in (0..7).rev() {
            let token = 30 + u64::from(slot);
            lineup.queue_for_next_round(player(slot, token, token));
        }
        for _ in 0..181 {
            room.step_with_forfeits(lineup.forfeits);
        }
        for _ in 1..300 {
            room.step();
        }
        assert!(lineup.try_start_next_round(&mut room).unwrap());
        assert_eq!(
            lineup.active.iter().map(|p| p.slot).collect::<Vec<_>>(),
            (0..8).collect::<Vec<_>>()
        );
        assert_eq!(
            room.game().match_state().unwrap().scores(),
            &[0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(room.game().fighter_ids().len(), 8);
    }
}
