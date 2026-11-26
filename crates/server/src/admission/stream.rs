use super::heartbeat::{Heartbeat, Probe};
use super::{
    Admission,
    binding::Binding,
    connection::Connection,
    matches::Match,
    socket::{send, send_message},
};
use axum::extract::ws::{Message, WebSocket};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::{MissedTickBehavior, interval};

pub(super) async fn play(
    socket: &mut WebSocket,
    state: &Admission,
    code: &str,
    owner: Connection,
    game: Arc<Match>,
    capabilities: super::socket::Capabilities,
) {
    let Some(mut binding) = Binding::open(state, code, owner, &game) else {
        return;
    };
    if capabilities.timing && !send(socket, vec![35, 1]).await {
        return;
    }
    let mut frames = game.client.publications();
    // Keep ownership and heartbeat checks alive even when the worker stops publishing.
    let mut timer = interval(Duration::from_millis(250));
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut tick = None;
    let mut published = None;
    let mut acknowledged = None;
    let mut since = Instant::now();
    let mut received = 0;
    let mut rematch_permission = None;
    let mut heartbeat = Heartbeat::new(Instant::now());
    loop {
        tokio::select! {
            changed = frames.changed() => { if changed.is_err() { return; } }
            _ = timer.tick() => (),
            message = socket.recv() => {
                if since.elapsed() >= Duration::from_secs(1) { since = Instant::now(); received = 0; }
                received += 1;
                let user_activity = matches!(&message, Some(Ok(Message::Binary(bytes))) if bytes.first() != Some(&36));
                if received > 120 || binding.host(&mut state.directory.lock().unwrap(), user_activity).is_none() { return; }
                match message {
                    Some(Ok(Message::Pong(bytes))) => heartbeat.pong(&bytes, Instant::now()),
                    Some(Ok(Message::Ping(_))) => (),
                    Some(Ok(Message::Binary(bytes))) => match bytes.as_ref() {
                        [18,1,1] => (),
                        [36, 1, a, b, c, d] if capabilities.timing => {
                            if !send(socket, vec![37, 1, *a, *b, *c, *d]).await { return; }
                        },
                        [30,1] => {
                            if state.directory.lock().unwrap().request_rematch(code, owner, &game) { return; }
                        },
                        [24,1, epoch @ ..] if epoch.len()==8 => {
                            let claimed = u64::from_le_bytes(epoch.try_into().unwrap());
                            if published == Some(claimed) { acknowledged = Some(claimed); }
                        },
                        [1,1,..] if bytes.len()==brawler_protocol::INPUT_BYTES => {
                            if let (Some(_),Some(epoch))=(binding.fighter,acknowledged) {
                                match game.client.input_for_slot(owner.slot, owner.generation, epoch, &bytes) {
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
        match heartbeat.poll(Instant::now()) {
            Probe::Wait => {}
            Probe::Expired => return,
            Probe::Ping(sequence) => {
                let message = Message::Ping(sequence.to_le_bytes().to_vec().into());
                if !send_message(socket, message).await {
                    return;
                }
            }
        }
        let Some(host) = binding.host(&mut state.directory.lock().unwrap(), false) else {
            return;
        };
        if game.client.returning_to_lobby() {
            return;
        }
        let Some(frame) = frames.borrow_and_update().clone() else {
            continue;
        };
        match binding.assignment(state, &frame) {
            Ok(Some(packet)) => {
                if packet[0] == 34 && !capabilities.lineups {
                    // Older clients can re-enter through their existing lobby handshake and get a fresh assignment.
                    send(socket, vec![27, 1]).await;
                    return;
                }
                if !send(socket, packet).await {
                    return;
                }
                acknowledged = None;
            }
            Ok(None) => (),
            Err(()) => return,
        }
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
        let bytes = if capabilities.compact && capabilities.landings {
            if capabilities.footsteps {
                frame.compact_with_footsteps(binding.fighter)
            } else {
                frame.compact_with_landings(binding.fighter)
            }
        } else if capabilities.compact {
            frame.compact_for(binding.fighter)
        } else {
            binding
                .fighter
                .map_or_else(|| frame.for_spectator(), |id| frame.for_recipient(id))
        };
        if !send(socket, bytes).await {
            return;
        }
        tick = Some(frame.tick());
    }
}
