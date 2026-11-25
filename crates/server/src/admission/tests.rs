use super::{
    directory::Directory,
    sessions::IDLE,
    types::{Create, Join},
};
use axum::http::{HeaderMap, StatusCode, header};
use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

fn issue(d: &mut Directory, now: Instant) -> HeaderMap {
    let value = d
        .sessions
        .issue(now)
        .unwrap_or_else(|_| panic!("session allocation"));
    let mut h = HeaderMap::new();
    h.insert(
        header::COOKIE,
        value.split(';').next().unwrap().parse().unwrap(),
    );
    h
}

#[test]
fn idle_session_expires_host_moves_and_empty_lobby_retires() {
    let mut d = Directory::default();
    let t = Instant::now();
    let ip = IpAddr::from([127, 0, 0, 1]);
    let host = issue(&mut d, t);
    let next = issue(&mut d, t);
    let a = d.sessions.authenticate(&host, t).ok().unwrap();
    let b = d.sessions.authenticate(&next, t).ok().unwrap();
    let make = Create {
        request_id: "a".repeat(32),
        name: "Coral".into(),
        color: None,
        capacity: 2,
    };
    let (_, seat) = d.create(a, make).ok().unwrap();
    let join = || Join {
        name: "Blue".into(),
        color: None,
    };
    d.join(b, &seat.code, join()).ok().unwrap();
    d.sessions
        .authenticate(&next, t + Duration::from_secs(1))
        .ok()
        .unwrap();
    let expired = t + IDLE;
    d.allow(ip, expired).ok().unwrap();
    assert!(d.sessions.authenticate(&host, expired).is_err());
    let survivor = d.join(b, &seat.code, join()).ok().unwrap();
    assert!(survivor.host);
    assert_eq!(survivor.slot, 1);
    d.allow(ip, expired + Duration::from_secs(1)).ok().unwrap();
    assert_eq!(d.metadata(&seat.code).ok().unwrap().players, 0);
    d.allow(ip, expired + Duration::from_secs(61)).ok().unwrap();
    assert!(d.metadata(&seat.code).is_err());
}

#[test]
fn admission_budget_is_bounded_per_peer_and_recovers_in_the_next_window() {
    let mut d = Directory::default();
    let t = Instant::now();
    let first = IpAddr::from([127, 0, 0, 1]);
    for _ in 0..120 {
        assert!(d.allow(first, t).is_ok());
    }
    assert_eq!(
        d.allow(first, t).err().unwrap().0,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert!(d.allow(IpAddr::from([127, 0, 0, 2]), t).is_ok());
    assert!(d.allow(first, t + Duration::from_secs(60)).is_ok());
}

#[test]
fn replacing_a_connection_or_reusing_a_seat_fences_late_messages_and_disconnects() {
    let mut d = Directory::default();
    let t = Instant::now();
    let host = issue(&mut d, t);
    let guest = issue(&mut d, t);
    let a = d.sessions.authenticate(&host, t).ok().unwrap();
    let b = d.sessions.authenticate(&guest, t).ok().unwrap();
    let (_, seat) = d
        .create(
            a,
            Create {
                request_id: "a".repeat(32),
                name: "Coral".into(),
                color: None,
                capacity: 2,
            },
        )
        .ok()
        .unwrap();
    let code = &seat.code;
    assert!(d.connect(code, b).is_none());
    let first = d.connect(code, a).unwrap();
    assert_eq!(d.live_state(code, first, Some(true)).unwrap().ready, 1);
    let second = d.connect(code, a).unwrap();
    assert_eq!(d.live_state(code, second, None).unwrap().ready, 0);
    assert!(d.live_state(code, first, Some(true)).is_none());
    d.live_state(code, second, Some(true)).unwrap();
    d.disconnect(code, first);
    assert_eq!(d.live_state(code, second, None).unwrap().ready, 1);
    d.leave(a, code, t).ok().unwrap();
    d.join(
        b,
        code,
        Join {
            name: "Blue".into(),
            color: None,
        },
    )
    .ok()
    .unwrap();
    let successor = d.connect(code, b).unwrap();
    let state = d.live_state(code, successor, Some(true)).unwrap();
    assert_eq!(
        (state.occupied, state.ready, state.host, state.revision),
        (1, 1, 0, 3)
    );
    d.disconnect(code, second);
    assert!(d.live_state(code, second, Some(false)).is_none());
    assert_eq!(d.live_state(code, successor, None).unwrap().ready, 1);
    d.disconnect(code, successor);
    assert!(d.live_state(code, successor, None).is_none());
}

#[tokio::test]
async fn only_a_ready_host_can_allocate_one_worker_for_the_current_roster() {
    use std::sync::Arc;
    let pool = Arc::new(crate::WorkerPool::new(1).unwrap());
    let state = super::Admission {
        maintenance: Arc::new(std::sync::Once::new()),
        pool: pool.clone(),
        directory: Arc::default(),
        origin: "http://localhost".parse().unwrap(),
        sockets: Arc::new(tokio::sync::Semaphore::new(2)),
    };
    let now = Instant::now();
    let (code, host, guest) = {
        let mut directory = state.directory.lock().unwrap();
        let a = issue(&mut directory, now);
        let b = issue(&mut directory, now);
        let a = directory.sessions.authenticate(&a, now).ok().unwrap();
        let b = directory.sessions.authenticate(&b, now).ok().unwrap();
        let (_, seat) = directory
            .create(
                a,
                Create {
                    request_id: "b".repeat(32),
                    name: "Mint".into(),
                    color: None,
                    capacity: 8,
                },
            )
            .ok()
            .unwrap();
        directory
            .join(
                b,
                &seat.code,
                Join {
                    name: "Gold".into(),
                    color: None,
                },
            )
            .ok()
            .unwrap();
        let host = directory.connect(&seat.code, a).unwrap();
        let guest = directory.connect(&seat.code, b).unwrap();
        directory.live_state(&seat.code, host, Some(true)).unwrap();
        (seat.code, host, guest)
    };
    assert_eq!(super::matches::start(&state, &code, host).await, Err(1));
    state
        .directory
        .lock()
        .unwrap()
        .live_state(&code, guest, Some(true))
        .unwrap();
    assert_eq!(super::matches::start(&state, &code, guest).await, Err(1));
    let busy = pool.open().unwrap();
    assert_eq!(super::matches::start(&state, &code, host).await, Err(2));
    drop(busy);
    tokio::time::timeout(Duration::from_secs(2), async {
        while pool.available() == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    super::matches::start(&state, &code, host).await.unwrap();
    assert_eq!(super::matches::start(&state, &code, host).await, Err(1));
    let mut d = state.directory.lock().unwrap();
    assert_eq!(d.metadata(&code).ok().unwrap().phase, "match");
    let game = d.rooms[&code].game.clone().unwrap();
    assert!(!d.request_rematch(&code, host, &game));
    assert!(!d.request_rematch(&code, guest, &game));
    drop(game);
    d.leave(host.session, &code, now).ok().unwrap();
    d.leave(guest.session, &code, now).ok().unwrap();
    assert_eq!(d.metadata(&code).ok().unwrap().phase, "lobby");
}
