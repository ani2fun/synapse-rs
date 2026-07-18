//! Integration test: the walking skeleton's endpoint, driven through the REAL assembled router
//! (`synapse_server::app()`) — middleware and all. What this suite exercises is what the binary
//! serves.

// Test code asserts hard — the banned-in-production panics are the point here.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

fn app() -> axum::Router {
    common::app_over(std::path::Path::new("/nonexistent-synapse-content"))
}

#[tokio::test]
async fn get_health_returns_the_typed_ok() {
    let app = app();

    let res = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/json")
    );

    let bytes = axum::body::to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json, serde_json::json!({ "status": "ok" }));
}

/// Liveness must NOT consult the store. These ITs run against a lazy pool pointed at nothing,
/// so if health ever grew a dependency check this would start failing — which is the point:
/// a liveness probe that dies with Postgres turns an outage into a restart loop.
#[tokio::test]
async fn liveness_stays_up_even_though_the_store_is_unreachable() {
    let res = app()
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}

/// Readiness DOES consult the store, so the same unreachable pool must produce a 503 — that is
/// what takes an instance out of the load balancer instead of letting it serve errors.
#[tokio::test]
async fn readiness_is_503_when_the_store_cannot_answer() {
    let res = app()
        .oneshot(Request::builder().uri("/api/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);

    let bytes = axum::body::to_bytes(res.into_body(), 64 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    // Generic on purpose: the store's own error names hosts and usernames and stays in the log.
    assert_eq!(json, serde_json::json!({ "status": "not ready" }));
    let body = String::from_utf8_lossy(&bytes);
    assert!(!body.contains("nobody"), "the connection user leaked: {body}");
    assert!(!body.contains("127.0.0.1"), "the host leaked: {body}");
}

#[tokio::test]
async fn unknown_route_is_a_404() {
    let app = app();

    let res = app
        .oneshot(Request::builder().uri("/api/nope").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
