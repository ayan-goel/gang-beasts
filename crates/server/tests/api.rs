use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use brawler_server::{WorkerPool, api};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;

async fn get(app: &axum::Router, path: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::get(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    assert_eq!(response.headers()["content-type"], "application/json");
    let bytes = to_bytes(response.into_body(), 16384).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn a_full_pool_stops_admission_and_keeps_serving_liveness_and_map_metadata() {
    let pool = Arc::new(WorkerPool::new(1).unwrap());
    let app = api(Arc::clone(&pool)).unwrap();
    assert_eq!(
        get(&app, "/readyz").await,
        (StatusCode::OK, json!({"ready":true}))
    );
    let _room = pool.open().unwrap();
    assert_eq!(
        get(&app, "/readyz").await,
        (StatusCode::SERVICE_UNAVAILABLE, json!({"ready":false}))
    );
    assert_eq!(
        get(&app, "/healthz").await,
        (StatusCode::OK, json!({"status":"ok"}))
    );
    let (code, body) = get(&app, "/api/maps").await;
    let maps = brawler_engine::MapDefinition::tour_catalog().unwrap();
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["engineHash"], brawler_engine::ENGINE_HASH);
    assert_eq!(
        body["catalogHash"],
        brawler_engine::MapDefinition::tour_hash()
    );
    let listed = body["maps"].as_array().unwrap();
    assert_eq!(listed.len(), maps.len());
    for (map, entry) in maps.iter().zip(listed) {
        assert_eq!(entry["id"], map.id);
        assert_eq!(
            entry["displayName"],
            map.tour
                .as_ref()
                .map_or("Skyline", |stage| stage.name.as_str())
        );
        assert_eq!(entry["contentVersion"], map.content_version);
        assert_eq!(entry["contentHash"].as_str().unwrap().len(), 64);
    }
}
