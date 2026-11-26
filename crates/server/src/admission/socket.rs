use super::{Admission, connection::Connection, sessions::SessionId, types::Fault};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use brawler_protocol::lobby::Command;
use std::time::{Duration, Instant};
use tokio::time::{MissedTickBehavior, interval, timeout};

#[derive(Clone, Copy)]
struct Capabilities {
    resume: bool,
    lineups: bool,
}

pub(super) async fn upgrade(
    State(state): State<Admission>,
    headers: HeaderMap,
    uri: axum::http::Uri,
    ws: WebSocketUpgrade,
) -> Result<Response, Fault> {
    let (session, code) = {
        let mut dir = state.directory.lock().unwrap();
        let session = dir.sessions.authenticate(&headers, Instant::now())?;
        let code = dir
            .current(session)
            .ok_or(Fault(StatusCode::FORBIDDEN, "not_a_member"))?;
        (session, code)
    };
    let permit = state
        .sockets
        .clone()
        .try_acquire_owned()
        .map_err(|_| Fault::SERVICE)?;
    let has = |field| {
        uri.query()
            .unwrap_or_default()
            .split('&')
            .any(|part| part == field)
    };
    let capabilities = Capabilities {
        resume: has("resume=1"),
        lineups: has("lineups=1"),
    };
    Ok(ws
        .max_frame_size(4096)
        .max_message_size(4096)
        .write_buffer_size(0)
        .max_write_buffer_size(8192)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            serve(socket, state, code, session, capabilities).await;
        })
        .into_response())
}

async fn serve(
    mut socket: WebSocket,
    state: Admission,
    code: String,
    session: SessionId,
    capabilities: Capabilities,
) {
    let Ok(Some(Ok(Message::Binary(packet)))) =
        timeout(Duration::from_secs(3), socket.recv()).await
    else {
        return;
    };
    let (requested, ticket) = match Command::decode(&packet) {
        Some(Command::Connect(code)) => (code, None),
        Some(Command::Resume(code, ticket)) if capabilities.resume => (code, Some(ticket)),
        _ => return,
    };
    if requested.as_slice() != code.as_bytes() {
        return;
    }
    let owner = {
        let mut dir = state.directory.lock().unwrap();
        match ticket {
            Some(id) => dir.resume_at(&code, session, id, Instant::now()),
            None => dir.connect(&code, session),
        }
    };
    let Some(owner) = owner else { return };
    let mut receipt = vec![31, 1];
    receipt.extend(ticket.unwrap_or(owner.generation).to_le_bytes());
    if !capabilities.resume || send(&mut socket, receipt).await {
        exchange(
            &mut socket,
            &state,
            &code,
            owner,
            ticket.is_some(),
            capabilities.lineups,
        )
        .await;
    }
    state.directory.lock().unwrap().disconnect(&code, owner);
}

async fn exchange(
    socket: &mut WebSocket,
    state: &Admission,
    code: &str,
    owner: Connection,
    resuming: bool,
    supports_lineups: bool,
) {
    let mut timer = interval(Duration::from_millis(150));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut previous = None;
    let mut window = Instant::now();
    let mut received = 0;
    let mut loaded = false;
    loop {
        let ready = tokio::select! {
            _ = timer.tick() => None,
            message = socket.recv() => {
                if window.elapsed() >= Duration::from_secs(1) { window = Instant::now(); received = 0; }
                received += 1;
                if received > 32 { return; }
                match message {
                    Some(Ok(Message::Binary(bytes))) => match Command::decode(&bytes) {
                        Some(Command::MapLoaded(hashes)) => {
                            loaded = super::content::compatible(hashes);
                            if !send(socket, vec![21, 1, u8::from(!loaded)]).await { return; }
                            if loaded { None } else { Some(false) }
                        }
                        Some(Command::SetReady(true)) if !loaded => {
                            if !send(socket, vec![21, 1, 2]).await { return; }
                            Some(false)
                        }
                        Some(Command::StartMatch) => {
                            if let Err(reason) = super::matches::start(state, code, owner).await {
                                if !send(socket, vec![25, 1, reason]).await { return; }
                            }
                            None
                        }
                        Some(Command::SetReady(value)) => Some(value),
                        _ => return,
                    },
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => None,
                    _ => return,
                }
            }
        };
        if loaded && previous.is_some() {
            let game = {
                let mut dir = state.directory.lock().unwrap();
                if dir.live_state(code, owner, None).is_none() {
                    return;
                }
                dir.rooms.get(code).and_then(|r| r.game.clone())
            };
            if let Some(game) = game {
                super::stream::play(socket, state, code, owner, game.clone(), supports_lineups)
                    .await;
                let returned = {
                    let mut dir = state.directory.lock().unwrap();
                    dir.prune(Instant::now());
                    dir.live_state(code, owner, None).is_some()
                        && dir.rooms.get(code).is_some_and(|r| {
                            r.game
                                .as_ref()
                                .is_none_or(|g| !std::sync::Arc::ptr_eq(g, &game))
                        })
                };
                if returned {
                    send(socket, vec![27, 1]).await;
                }
                return;
            }
        }
        let (current, bytes, running_match) = {
            let mut directory = state.directory.lock().unwrap();
            let Some(current) = directory.live_state(code, owner, ready) else {
                return;
            };
            if previous == Some(current) {
                continue;
            }
            let mut bytes = current.encode_header(previous.is_none().then_some(owner.slot as u8));
            if previous.is_none_or(|p| p.revision != current.revision) {
                let Ok(roster) = directory.roster(owner.session, code) else {
                    return;
                };
                bytes.push(roster.capacity);
                for seat in roster.players {
                    bytes.extend([seat.color, seat.name.len() as u8]);
                    bytes.extend(seat.name.as_bytes());
                }
            }
            (
                current,
                bytes,
                previous.is_none()
                    && directory
                        .rooms
                        .get(code)
                        .is_some_and(|room| room.game.is_some()),
            )
        };
        // A stalled recipient must release its bounded socket slot and ready flag.
        if !send(socket, bytes).await {
            return;
        }
        if running_match && !send(socket, vec![28, 1]).await {
            return;
        }
        if previous.is_none() && resuming && !running_match && !send(socket, vec![33, 1]).await {
            return;
        }
        previous = Some(current);
    }
}

pub(super) async fn send(socket: &mut WebSocket, bytes: Vec<u8>) -> bool {
    send_message(socket, Message::Binary(bytes.into())).await
}

pub(super) async fn send_message(socket: &mut WebSocket, message: Message) -> bool {
    matches!(
        timeout(Duration::from_secs(2), socket.send(message)).await,
        Ok(Ok(()))
    )
}
