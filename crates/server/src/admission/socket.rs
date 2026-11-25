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

pub(super) async fn upgrade(
    State(state): State<Admission>,
    headers: HeaderMap,
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
    Ok(ws
        .max_frame_size(4096)
        .max_message_size(4096)
        .write_buffer_size(0)
        .max_write_buffer_size(8192)
        .on_upgrade(move |socket| async move {
            let _permit = permit;
            serve(socket, state, code, session).await;
        })
        .into_response())
}

async fn serve(mut socket: WebSocket, state: Admission, code: String, session: SessionId) {
    let Ok(Some(Ok(Message::Binary(packet)))) =
        timeout(Duration::from_secs(3), socket.recv()).await
    else {
        return;
    };
    if !matches!(Command::decode(&packet), Some(Command::Connect(requested)) if requested.as_slice() == code.as_bytes())
    {
        return;
    }
    let Some(owner) = state.directory.lock().unwrap().connect(&code, session) else {
        return;
    };
    exchange(&mut socket, &state, &code, owner).await;
    state.directory.lock().unwrap().disconnect(&code, owner);
}

async fn exchange(socket: &mut WebSocket, state: &Admission, code: &str, owner: Connection) {
    let mut timer = interval(Duration::from_millis(150));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut previous = None;
    let mut window = Instant::now();
    let mut received = 0;
    loop {
        let ready = tokio::select! {
            _ = timer.tick() => None,
            message = socket.recv() => {
                if window.elapsed() >= Duration::from_secs(1) { window = Instant::now(); received = 0; }
                received += 1;
                if received > 32 { return; }
                match message {
                    Some(Ok(Message::Binary(bytes))) => match Command::decode(&bytes) {
                        Some(Command::SetReady(value)) => Some(value),
                        _ => return,
                    },
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => None,
                    _ => return,
                }
            }
        };
        let (current, bytes) = {
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
            (current, bytes)
        };
        // A stalled recipient must release its bounded socket slot and ready flag.
        if !matches!(
            timeout(
                Duration::from_secs(2),
                socket.send(Message::Binary(bytes.into()))
            )
            .await,
            Ok(Ok(()))
        ) {
            return;
        }
        previous = Some(current);
    }
}
