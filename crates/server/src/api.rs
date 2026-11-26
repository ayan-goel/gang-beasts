use crate::WorkerPool;
use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    routing::get,
};
use brawler_engine::{CharacterError, ENGINE_HASH, MapDefinition};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MapInfo {
    id: String,
    display_name: &'static str,
    content_version: u32,
    content_hash: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    protocol_version: u8,
    engine_hash: &'static str,
    catalog_hash: String,
    maps: Vec<MapInfo>,
}

#[derive(Clone)]
struct ApiState {
    pool: Arc<WorkerPool>,
    catalog: Catalog,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}
#[derive(Serialize)]
struct Readiness {
    ready: bool,
}

pub fn api(pool: Arc<WorkerPool>) -> Result<Router, CharacterError> {
    api_for_origin(pool, "http://127.0.0.1:5173")
}

pub fn api_for_origin(pool: Arc<WorkerPool>, origin: &str) -> Result<Router, CharacterError> {
    let maps = MapDefinition::tour_catalog()?
        .iter()
        .map(|map| {
            let content_hash = if map.tour.is_none() {
                MapDefinition::skyline_hash()
            } else {
                format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(map).expect("validated map serializes"))
                )
            };
            MapInfo {
                id: map.id.clone(),
                display_name: map
                    .tour
                    .as_ref()
                    .map_or("Skyline", |stage| stage.name.as_str()),
                content_version: map.content_version,
                content_hash,
            }
        })
        .collect();
    let state = ApiState {
        pool: Arc::clone(&pool),
        catalog: Catalog {
            protocol_version: brawler_protocol::VERSION,
            engine_hash: ENGINE_HASH,
            catalog_hash: MapDefinition::tour_hash(),
            maps,
        },
    };
    Ok(Router::new()
        .route("/healthz", get(|| async { Json(Health { status: "ok" }) }))
        .route("/readyz", get(readiness))
        .route(
            "/api/maps",
            get(|State(s): State<ApiState>| async move { Json(s.catalog) }),
        )
        .with_state(state)
        .merge(crate::admission::routes(origin, pool)?))
}

async fn readiness(State(s): State<ApiState>) -> impl axum::response::IntoResponse {
    let ready = s.pool.available() != 0;
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        [(header::CACHE_CONTROL, "no-store")],
        Json(Readiness { ready }),
    )
}
