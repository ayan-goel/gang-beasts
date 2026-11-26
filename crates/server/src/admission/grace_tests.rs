use super::{
    directory::Directory,
    types::{Create, Join},
};
use std::time::{Duration, Instant};

#[test]
fn only_the_current_generation_starts_grace_and_exact_expiry_releases_the_host_seat() {
    let mut directory = Directory::default();
    let now = Instant::now();
    let host = [1; 32];
    let guest = [2; 32];
    directory.sessions.0.insert(host, now);
    directory.sessions.0.insert(guest, now);
    let (_, seat) = directory
        .create(
            host,
            Create {
                request_id: "e".repeat(32),
                name: "Host".into(),
                color: None,
                capacity: 2,
                wins: 3,
            },
        )
        .ok()
        .unwrap();
    let code = seat.code;
    directory
        .join(
            guest,
            &code,
            Join {
                name: "Guest".into(),
                color: None,
            },
        )
        .ok()
        .unwrap();
    let first = directory.connect(&code, host).unwrap();
    let replacement = directory.connect(&code, host).unwrap();
    let resume =
        |dir: &mut Directory, session, ticket, time| dir.resume_at(&code, session, ticket, time);
    assert!(resume(&mut directory, host, replacement.generation, now).is_none());
    directory.disconnect_at(&code, first, now);
    directory.prune(now + Duration::from_secs(21));
    assert_eq!(directory.current(host).as_deref(), Some(code.as_str()));
    directory.disconnect_at(&code, replacement, now + Duration::from_secs(22));
    let deadline = now + Duration::from_secs(42);
    let before = deadline - Duration::from_nanos(1);
    assert!(resume(&mut directory, host, first.generation, before).is_none());
    assert!(resume(&mut directory, guest, replacement.generation, before).is_none());
    let back = resume(&mut directory, host, replacement.generation, before).unwrap();
    directory.disconnect_at(&code, back, before);
    let back = resume(&mut directory, host, replacement.generation, before).unwrap();
    assert_ne!(back.generation, replacement.generation);
    directory.prune(deadline);
    assert_eq!(directory.current(host).as_deref(), Some(code.as_str()));
    directory.disconnect_at(&code, back, deadline);
    assert!(
        resume(
            &mut directory,
            host,
            replacement.generation,
            deadline + Duration::from_secs(20)
        )
        .is_none()
    );
    let roster = directory.roster(guest, &code).ok().unwrap();
    assert_eq!(roster.players.len(), 1);
    assert!(roster.players[0].host);
}

#[tokio::test]
async fn maintenance_releases_an_abandoned_worker_and_retires_the_empty_room_without_http_traffic()
{
    use std::sync::Arc;
    let pool = Arc::new(crate::WorkerPool::new(1).unwrap());
    let state = super::Admission {
        pool: pool.clone(),
        directory: Arc::default(),
        maintenance: Arc::new(std::sync::Once::new()),
        origin: "http://localhost".parse().unwrap(),
        sockets: Arc::new(tokio::sync::Semaphore::new(2)),
    };
    let now = Instant::now();
    let (code, host, guest) = {
        let mut d = state.directory.lock().unwrap();
        d.sessions.0.insert([0; 32], now);
        d.sessions.0.insert([1; 32], now);
        let (_, seat) = d
            .create(
                [0; 32],
                Create {
                    request_id: "d".repeat(32),
                    name: "A".into(),
                    color: None,
                    capacity: 2,
                    wins: 3,
                },
            )
            .ok()
            .unwrap();
        d.join(
            [1; 32],
            &seat.code,
            Join {
                name: "B".into(),
                color: None,
            },
        )
        .ok()
        .unwrap();
        let a = d.connect(&seat.code, [0; 32]).unwrap();
        let b = d.connect(&seat.code, [1; 32]).unwrap();
        d.live_state(&seat.code, a, Some(true)).unwrap();
        d.live_state(&seat.code, b, Some(true)).unwrap();
        (seat.code, a, b)
    };
    super::matches::start(&state, &code, host).await.unwrap();
    assert_eq!(pool.available(), 0);
    {
        let mut d = state.directory.lock().unwrap();
        for connection in [host, guest] {
            d.disconnect_at(&code, connection, now - super::maintenance::GRACE);
        }
    }
    super::maintenance::start(&state.directory);
    tokio::time::timeout(Duration::from_secs(2), async {
        while pool.available() == 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    {
        let mut d = state.directory.lock().unwrap();
        assert_eq!(d.metadata(&code).ok().unwrap().players, 0);
        d.rooms.get_mut(&code).unwrap().empty_since = Some(now - Duration::from_secs(60));
    }
    tokio::time::timeout(Duration::from_secs(2), async {
        while state.directory.lock().unwrap().rooms.contains_key(&code) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}
