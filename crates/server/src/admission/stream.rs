use super::{Admission, connection::Connection, matches::Match, socket::send};
use axum::extract::ws::{Message, WebSocket};
use brawler_engine::FighterId;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::{MissedTickBehavior, interval};

// Generation changes bypass the bounded input queue, including when a socket drops mid-send.
struct Binding<'a> {
    game: &'a Match,
    fighter: Option<FighterId>,
    owner: Connection,
}
impl Drop for Binding<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.fighter {
            self.game.client.unbind(id, self.owner.generation);
        }
    }
}

fn current_host(
    state: &Admission,
    code: &str,
    owner: Connection,
    game: &Arc<Match>,
    touch: bool,
) -> Option<bool> {
    let mut dir = state.directory.lock().unwrap();
    let membership = dir.live_state(code, owner, None)?;
    if !dir
        .rooms
        .get(code)?
        .game
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, game))
    {
        return None;
    }
    if touch {
        *dir.sessions.0.get_mut(&owner.session)? = Instant::now();
    }
    Some(usize::from(membership.host) == owner.slot)
}

pub(super) async fn play(
    socket: &mut WebSocket,
    state: &Admission,
    code: &str,
    owner: Connection,
    game: Arc<Match>,
) {
    let fighter = game.fighter(owner.session);
    {
        let mut dir = state.directory.lock().unwrap();
        if dir.live_state(code, owner, None).is_none()
            || !dir
                .rooms
                .get(code)
                .and_then(|r| r.game.as_ref())
                .is_some_and(|g| Arc::ptr_eq(&game, g))
        {
            return;
        }
        if let Some(id) = fighter {
            game.client.bind(id, owner.generation);
        }
    }
    let _binding = Binding {
        game: &game,
        fighter,
        owner,
    };
    if !send(socket, game.welcome(owner.session)).await {
        return;
    }
    let mut timer = interval(Duration::from_nanos(1_000_000_000 / 30));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut tick = None;
    let mut published = None;
    let mut acknowledged = None;
    let mut since = Instant::now();
    let mut received = 0;
    let mut rematch_permission = None;
    loop {
        tokio::select! {
            _ = timer.tick() => (),
            message = socket.recv() => {
                if since.elapsed() >= Duration::from_secs(1) { since = Instant::now(); received = 0; }
                received += 1;
                if received > 120 || current_host(state, code, owner, &game, true).is_none() { return; }
                match message {
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => (),
                    Some(Ok(Message::Binary(bytes))) => match bytes.as_ref() {
                        [18,1,1] => (),
                        [30,1] => {
                            if state.directory.lock().unwrap().request_rematch(code, owner, &game) { return; }
                        },
                        [24,1, epoch @ ..] if epoch.len()==8 => {
                            let claimed = u64::from_le_bytes(epoch.try_into().unwrap());
                            if published == Some(claimed) { acknowledged = Some(claimed); }
                        },
                        [1,1,..] if bytes.len()==brawler_protocol::INPUT_BYTES => {
                            if let (Some(id),Some(epoch))=(fighter,acknowledged) {
                                match game.client.input_for(id, owner.generation, epoch, &bytes) {
                                    Ok(()) | Err(crate::DispatchError::Busy) => (),
                                    _ => return,
                                }
                            }
                        },
                        _ => return,
                    },
                    _ => return,
                }
                continue;
            }
        }
        let Some(host) = current_host(state, code, owner, &game, false) else {
            return;
        };
        if game.client.returning_to_lobby() {
            return;
        }
        let Some(frame) = game.client.latest() else {
            continue;
        };
        if frame.match_finished && rematch_permission != Some(host) {
            if !send(socket, vec![29, 1, u8::from(host)]).await {
                return;
            }
            rematch_permission = Some(host);
        }
        if tick == Some(frame.tick()) {
            continue;
        }
        if published != Some(frame.round) {
            acknowledged = None;
        }
        published = Some(frame.round);
        let bytes = fighter.map_or_else(|| frame.for_spectator(), |id| frame.for_recipient(id));
        if !send(socket, bytes).await {
            return;
        }
        tick = Some(frame.tick());
    }
}
