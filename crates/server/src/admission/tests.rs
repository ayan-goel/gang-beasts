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
