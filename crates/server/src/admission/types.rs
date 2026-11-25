use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub(super) struct Fault(pub StatusCode, pub &'static str);

impl Fault {
    pub const INVALID: Self = Self(StatusCode::UNPROCESSABLE_ENTITY, "invalid_input");
    pub const UNAUTHORIZED: Self = Self(StatusCode::UNAUTHORIZED, "session_required");
    pub const SERVICE: Self = Self(StatusCode::SERVICE_UNAVAILABLE, "unavailable");
    pub const CONFLICT: Self = Self(StatusCode::CONFLICT, "request_conflict");
    pub const MISSING: Self = Self(StatusCode::NOT_FOUND, "room_not_found");
}
#[derive(Serialize)]
struct ErrorBody {
    error: ErrorCode,
}
#[derive(Serialize)]
struct ErrorCode {
    code: &'static str,
}
impl IntoResponse for Fault {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(ErrorBody {
                error: ErrorCode { code: self.1 },
            }),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Empty {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Create {
    pub request_id: String,
    pub name: String,
    pub color: Option<u8>,
    #[serde(default = "default_capacity")]
    pub capacity: u8,
}
fn default_capacity() -> u8 {
    8
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Join {
    pub name: String,
    pub color: Option<u8>,
}

pub(super) fn player(name: &str, color: Option<u8>) -> Result<&str, Fault> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > 24
        || name.chars().any(char::is_control)
        || color.is_some_and(|c| c >= 8)
    {
        return Err(Fault::INVALID);
    }
    Ok(trimmed)
}

#[derive(Serialize)]
pub(super) struct PublicRoom {
    pub code: String,
    pub phase: &'static str,
    pub capacity: u8,
    pub players: usize,
}
#[derive(Serialize)]
pub(super) struct Seat {
    pub code: String,
    pub slot: u8,
    pub host: bool,
    pub name: String,
    pub color: u8,
}

#[derive(Serialize)]
pub(super) struct Roster {
    pub code: String,
    pub capacity: u8,
    pub you: u8,
    pub players: Vec<Seat>,
}
