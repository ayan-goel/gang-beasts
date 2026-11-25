mod connection;
mod content;
mod directory;
mod lobby;
mod matches;
mod sessions;
mod socket;
mod stream;
mod types;

use axum::{
    Json, Router,
    extract::{ConnectInfo, DefaultBodyLimit, Path, Request, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use brawler_engine::CharacterError;
use directory::Directory;
use serde::Serialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};
use types::{Create, Empty, Fault, Join, PublicRoom, Seat};

#[derive(Clone)]
struct Admission {
    pool: Arc<crate::WorkerPool>,
    directory: Arc<Mutex<Directory>>,
    origin: HeaderValue,
    sockets: Arc<tokio::sync::Semaphore>,
}

pub(crate) fn routes(origin: &str, pool: Arc<crate::WorkerPool>) -> Result<Router, CharacterError> {
    let uri: axum::http::Uri = origin
        .parse()
        .map_err(|_| CharacterError::Invalid("invalid public origin"))?;
    let local = matches!(uri.host(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if uri.authority().is_none()
        || uri.authority().is_some_and(|a| a.as_str().contains('@'))
        || !matches!(uri.path(), "" | "/")
        || uri.query().is_some()
        || !(uri.scheme_str() == Some("https") || (local && uri.scheme_str() == Some("http")))
    {
        return Err(CharacterError::Invalid(
            "expected HTTPS or loopback public origin",
        ));
    }
    let state = Admission {
        pool,
        directory: Arc::default(),
        sockets: Arc::new(tokio::sync::Semaphore::new(512)),
        origin: HeaderValue::from_str(origin.trim_end_matches('/'))
            .map_err(|_| CharacterError::Invalid("invalid origin header"))?,
    };
    Ok(Router::new()
        .route("/ws", get(socket::upgrade))
        .route("/api/session", post(session))
        .route("/api/rooms", post(create))
        .route("/api/rooms/{code}", get(metadata))
        .route("/api/rooms/{code}/lobby", get(roster))
        .route("/api/rooms/{code}/leave", post(leave))
        .route("/api/rooms/{code}/join", post(join))
        .layer(DefaultBodyLimit::max(4096))
        .route_layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state))
}

async fn guard(
    State(state): State<Admission>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    // Browser credentials are ambient. Reject foreign or missing Origins before state changes.
    let mut response = if (req.method() == axum::http::Method::POST || req.uri().path() == "/ws")
        && req.headers().get(header::ORIGIN) != Some(&state.origin)
    {
        Fault(StatusCode::FORBIDDEN, "origin_denied").into_response()
    } else {
        let gate = state
            .directory
            .lock()
            .unwrap()
            .allow(peer.ip(), Instant::now());
        match gate {
            Ok(()) => next.run(req).await,
            Err(err) => err.into_response(),
        }
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn body<T>(result: Result<Json<T>, JsonRejection>) -> Result<T, Fault> {
    result
        .map(|Json(v)| v)
        .map_err(|e| Fault(e.status(), "invalid_input"))
}

#[derive(Serialize)]
struct SessionInfo {
    active: bool,
    room: Option<String>,
}

async fn session(
    State(s): State<Admission>,
    headers: HeaderMap,
    data: Result<Json<Empty>, JsonRejection>,
) -> Result<Response, Fault> {
    body(data)?;
    let mut d = s.directory.lock().unwrap();
    let now = Instant::now();
    let current = d.sessions.authenticate(&headers, now).ok();
    let room = current.and_then(|id| d.current(id));
    let mut res = Json(SessionInfo { active: true, room }).into_response();
    if current.is_none() {
        let cookie = d.sessions.issue(now)?;
        res.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie).expect("server-generated hex cookie"),
        );
    }
    Ok(res)
}

async fn create(
    State(s): State<Admission>,
    headers: HeaderMap,
    data: Result<Json<Create>, JsonRejection>,
) -> Result<(StatusCode, Json<Seat>), Fault> {
    let req = body(data)?;
    let mut d = s.directory.lock().unwrap();
    let id = d.sessions.authenticate(&headers, Instant::now())?;
    let (status, seat) = d.create(id, req)?;
    Ok((status, Json(seat)))
}

fn code(raw: &str) -> Result<String, Fault> {
    if raw.len() != 6 {
        return Err(Fault::MISSING);
    }
    let upper = raw.to_ascii_uppercase();
    if !upper.bytes().all(|b| sessions::ALPHABET.contains(&b)) {
        return Err(Fault::MISSING);
    }
    Ok(upper)
}
async fn join(
    State(s): State<Admission>,
    Path(raw): Path<String>,
    headers: HeaderMap,
    data: Result<Json<Join>, JsonRejection>,
) -> Result<Json<Seat>, Fault> {
    let key = code(&raw)?;
    let req = body(data)?;
    let mut d = s.directory.lock().unwrap();
    let id = d.sessions.authenticate(&headers, Instant::now())?;
    Ok(Json(d.join(id, &key, req)?))
}
async fn metadata(
    State(s): State<Admission>,
    Path(raw): Path<String>,
) -> Result<Json<PublicRoom>, Fault> {
    Ok(Json(s.directory.lock().unwrap().metadata(&code(&raw)?)?))
}

#[cfg(test)]
mod tests;

async fn roster(
    State(s): State<Admission>,
    Path(raw): Path<String>,
    headers: HeaderMap,
) -> Result<Json<types::Roster>, Fault> {
    let mut d = s.directory.lock().unwrap();
    let id = d.sessions.authenticate(&headers, Instant::now())?;
    Ok(Json(d.roster(id, &code(&raw)?)?))
}
async fn leave(
    State(s): State<Admission>,
    Path(raw): Path<String>,
    headers: HeaderMap,
    data: Result<Json<Empty>, JsonRejection>,
) -> Result<StatusCode, Fault> {
    body(data)?;
    let mut d = s.directory.lock().unwrap();
    let now = Instant::now();
    let id = d.sessions.authenticate(&headers, now)?;
    d.leave(id, &code(&raw)?, now)?;
    Ok(StatusCode::NO_CONTENT)
}
