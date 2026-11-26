use super::{
    Admission,
    connection::Connection,
    directory::Directory,
    matches::{Match, welcome},
};
use crate::{RoomFrame, lineup::Participant};
use brawler_engine::FighterId;
use std::{sync::Arc, time::Instant};

pub(super) struct Binding<'a> {
    game: &'a Arc<Match>,
    code: &'a str,
    owner: Connection,
    lineup: Option<Arc<[Participant]>>,
    pub fighter: Option<FighterId>,
}

impl<'a> Binding<'a> {
    pub fn open(
        state: &Admission,
        code: &'a str,
        owner: Connection,
        game: &'a Arc<Match>,
    ) -> Option<Self> {
        let binding = Self {
            game,
            code,
            owner,
            lineup: None,
            fighter: None,
        };
        let mut dir = state.directory.lock().unwrap();
        binding.host(&mut dir, false)?;
        let entrant = dir.rooms[code].participant(owner.slot)?;
        game.lineup.lock().unwrap().queue_for_next_round(entrant);
        Some(binding)
    }

    pub fn host(&self, dir: &mut Directory, touch: bool) -> Option<bool> {
        let membership = dir.live_state(self.code, self.owner, None)?;
        let game = dir.rooms.get(self.code)?.game.as_ref()?;
        if !Arc::ptr_eq(game, self.game) {
            return None;
        }
        if touch {
            *dir.sessions.0.get_mut(&self.owner.session)? = Instant::now();
        }
        Some(usize::from(membership.host) == self.owner.slot)
    }

    pub fn assignment(
        &mut self,
        state: &Admission,
        frame: &RoomFrame,
    ) -> Result<Option<Vec<u8>>, ()> {
        let players = frame
            .lineup
            .as_ref()
            .expect("hosted snapshot carries its lineup");
        if self
            .lineup
            .as_ref()
            .is_some_and(|last| Arc::ptr_eq(last, players))
        {
            return Ok(None);
        }
        self.fighter = players
            .iter()
            .position(|p| p.member == self.owner.member)
            .map(|i| FighterId::ALL[i]);
        if self.fighter.is_some() {
            // Recheck ownership while binding. A replacement tab may have arrived since this frame was selected.
            let mut dir = state.directory.lock().unwrap();
            self.host(&mut dir, false).ok_or(())?;
            self.game
                .client
                .bind_slot(self.owner.slot, self.owner.generation);
        }
        let mut packet = Vec::new();
        if self.lineup.is_some() {
            packet.extend([34, 1]);
            packet.extend(frame.round.to_le_bytes());
        }
        packet.extend(welcome(players, self.fighter));
        self.lineup = Some(players.clone());
        Ok(Some(packet))
    }
}

impl Drop for Binding<'_> {
    fn drop(&mut self) {
        self.game
            .lineup
            .lock()
            .unwrap()
            .cancel_wait(self.owner.generation);
        self.game
            .client
            .unbind_slot(self.owner.slot, self.owner.generation);
    }
}
