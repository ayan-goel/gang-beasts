use super::{Admission, connection::Connection, matches::Match, socket::send};
use axum::extract::ws::{Message, WebSocket};
use std::{sync::Arc, time::Duration};
use tokio::time::{MissedTickBehavior, interval};

pub(super) async fn play(
    socket: &mut WebSocket,
    state: &Admission,
    code: &str,
    owner: Connection,
    game: Arc<Match>,
) {
    if !send(socket, game.welcome(owner.session)).await {
        return;
    }
    let fighter = game.fighter(owner.session);
    let mut timer = interval(Duration::from_nanos(1_000_000_000 / 30));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut tick = None;
    loop {
        tokio::select! {
            _ = timer.tick() => (),
            message = socket.recv() => match message {
                Some(Ok(Message::Binary(bytes))) if bytes.as_ref() == [18, 1, 1] => continue,
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                _ => return,
            }
        }
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
        }
        let Some(frame) = game.client.latest() else {
            continue;
        };
        if tick == Some(frame.tick()) {
            continue;
        }
        let bytes = fighter.map_or_else(|| frame.for_spectator(), |id| frame.for_recipient(id));
        if !send(socket, bytes).await {
            return;
        }
        tick = Some(frame.tick());
    }
}
