use axum::{
    Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use brawler_server::{WorkerPool, api};
use serde_json::{Value, json};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceExt;

const ORIGIN: &str = "http://127.0.0.1:5173";
async fn request(
    app: &Router,
    path: &str,
    body: Option<Value>,
    cookie: &str,
    origin: &str,
) -> (StatusCode, Value, String) {
    let mut req = Request::builder()
        .uri(path)
        .extension(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 3333))))
        .header("cookie", cookie)
        .header("origin", origin);
    if body.is_some() {
        req = req
            .method("POST")
            .header("content-type", "application/json");
    }
    let res = app
        .clone()
        .oneshot(
            req.body(Body::from(body.map_or_else(String::new, |v| v.to_string())))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let cookie = res
        .headers()
        .get("set-cookie")
        .map_or("", |h| h.to_str().unwrap())
        .to_owned();
    let raw = to_bytes(res.into_body(), 4096).await.unwrap();
    (
        status,
        if status == StatusCode::NO_CONTENT {
            Value::Null
        } else {
            serde_json::from_slice(&raw).expect("JSON API response")
        },
        cookie,
    )
}
async fn session(app: &Router) -> String {
    let (s, _, header) = request(app, "/api/session", Some(json!({})), "", ORIGIN).await;
    assert_eq!(s, StatusCode::OK);
    for flag in ["HttpOnly", "Secure", "SameSite=Strict", "Path=/"] {
        assert!(header.contains(flag));
    }
    header.split(';').next().unwrap().to_owned()
}
fn create() -> Value {
    json!({"requestId":"a1b2c3d4e5f601234567890123456789","name":"Coral","capacity":2})
}

#[tokio::test]
async fn creates_retry_safely_and_admits_only_the_requested_number_of_friends() {
    let app = api(Arc::new(WorkerPool::new(1).unwrap())).unwrap();
    let host = session(&app).await;
    let guest = session(&app).await;
    let outsider = session(&app).await;
    assert_ne!(host, guest);
    let (s, room, _) = request(&app, "/api/rooms", Some(create()), &host, ORIGIN).await;
    assert_eq!(s, StatusCode::CREATED);
    let code = room["code"].as_str().unwrap();
    assert_eq!(code.len(), 6);
    assert_eq!(room["slot"], 0);
    let (s, duplicate, _) = request(&app, "/api/rooms", Some(create()), &host, ORIGIN).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(duplicate, room);
    let mut different = create();
    different["capacity"] = json!(5);
    assert_eq!(
        request(&app, "/api/rooms", Some(different), &host, ORIGIN)
            .await
            .0,
        StatusCode::CONFLICT
    );
    let path = format!("/api/rooms/{code}");
    let join = format!("{path}/join");
    let (s, member, _) = request(&app, &join, Some(json!({"name":"Blue"})), &guest, ORIGIN).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(member["slot"], 1);
    assert_eq!(
        request(&app, &join, Some(json!({"name":"Blue"})), &guest, ORIGIN)
            .await
            .1,
        member
    );
    assert_eq!(
        request(&app, &join, Some(json!({"name":"Late"})), &outsider, ORIGIN)
            .await
            .1["error"]["code"],
        "room_full"
    );
    let (_, metadata, _) = request(&app, &path, None, "", ORIGIN).await;
    assert_eq!(
        metadata,
        json!({"code":code,"capacity":2,"players":2,"phase":"lobby"})
    );
}

#[tokio::test]
async fn rejects_foreign_origins_unissued_credentials_and_invalid_request_fields() {
    let app = api(Arc::new(WorkerPool::new(1).unwrap())).unwrap();
    assert_eq!(
        request(
            &app,
            "/api/session",
            Some(json!({})),
            "",
            "https://other.invalid"
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&app, "/api/rooms", Some(create()), "", ORIGIN)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let member = session(&app).await;
    assert_eq!(
        request(
            &app,
            "/api/rooms",
            Some(create()),
            &member,
            "https://other.invalid"
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut invalid = create();
    invalid["capacity"] = json!(9);
    assert_eq!(
        request(&app, "/api/rooms", Some(invalid), &member, ORIGIN)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut extra = create();
    extra["score"] = json!(42);
    assert_eq!(
        request(&app, "/api/rooms", Some(extra), &member, ORIGIN)
            .await
            .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn eight_anonymous_sessions_can_join_and_a_ninth_receives_room_full() {
    let app = api(Arc::new(WorkerPool::new(1).unwrap())).unwrap();
    let mut eight = create();
    eight["capacity"] = json!(8);
    let host = session(&app).await;
    let (_, result, _) = request(&app, "/api/rooms", Some(eight), &host, ORIGIN).await;
    let join = format!("/api/rooms/{}/join", result["code"].as_str().unwrap());
    for i in 1..=8 {
        let cookie = session(&app).await;
        let (s, body, _) = request(
            &app,
            &join,
            Some(json!({"name":format!("Friend {i}")})),
            &cookie,
            ORIGIN,
        )
        .await;
        if i < 8 {
            assert_eq!(body["slot"], i);
        } else {
            assert_eq!(s, StatusCode::CONFLICT);
        }
    }
}

#[tokio::test]
async fn a_room_code_cannot_read_names_and_leaving_cannot_replay_the_create_request() {
    let app = api(Arc::new(WorkerPool::new(1).unwrap())).unwrap();
    let creator = session(&app).await;
    let outsider = session(&app).await;
    let (_, made, _) = request(&app, "/api/rooms", Some(create()), &creator, ORIGIN).await;
    let path = format!("/api/rooms/{}", made["code"].as_str().unwrap());
    let lobby = format!("{path}/lobby");
    assert_eq!(
        request(&app, &lobby, None, "", ORIGIN).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, &lobby, None, &outsider, ORIGIN).await.0,
        StatusCode::FORBIDDEN
    );
    let (_, roster, _) = request(&app, &lobby, None, &creator, ORIGIN).await;
    assert_eq!(roster["players"][0]["name"], "Coral");
    assert_eq!(
        request(
            &app,
            &format!("{path}/leave"),
            Some(json!({})),
            &creator,
            ORIGIN
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, &lobby, None, &creator, ORIGIN).await.0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&app, "/api/rooms", Some(create()), &creator, ORIGIN)
            .await
            .0,
        StatusCode::CONFLICT
    );
}
