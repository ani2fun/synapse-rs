//! Integration: the security stamp on EVERY route class (oracle: `SecurityHeadersSpec` + the
//! step-36/38 prod incidents as fixtures). The exact header values are the contract — an
//! over-tight CSP silently broke prod fonts/Monaco once and d2's ELK worker twice; these
//! assertions are the regression net.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use std::fs;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use tower::ServiceExt;

const ISSUER: &str = "https://keycloak.kakde.eu/realms/synapse";

async fn headers_of(app: axum::Router, uri: &str) -> (StatusCode, HeaderMap) {
    let res = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    (res.status(), res.headers().clone())
}

fn value<'a>(headers: &'a HeaderMap, name: &str) -> &'a str {
    headers
        .get(name)
        .unwrap_or_else(|| panic!("{name} missing"))
        .to_str()
        .unwrap()
}

fn app(root: &std::path::Path) -> axum::Router {
    let mut deps = common::deps(root);
    ISSUER.clone_into(&mut deps.ident.issuer);
    synapse_server::app(deps)
}

#[tokio::test]
async fn stamps_all_five_headers_on_a_200() {
    let tmp = tempfile::tempdir().unwrap();
    let (status, headers) = headers_of(app(tmp.path()), "/api/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value(&headers, "x-content-type-options"), "nosniff");
    assert_eq!(value(&headers, "x-frame-options"), "SAMEORIGIN");
    assert_eq!(
        value(&headers, "referrer-policy"),
        "strict-origin-when-cross-origin"
    );
    assert!(value(&headers, "content-security-policy").contains("frame-ancestors 'self'"));
    assert!(value(&headers, "strict-transport-security").contains("max-age=31536000"));
}

#[tokio::test]
async fn the_csp_allows_the_keycloak_origin_or_sign_in_breaks() {
    let tmp = tempfile::tempdir().unwrap();
    let (_, headers) = headers_of(app(tmp.path()), "/api/health").await;
    let csp = value(&headers, "content-security-policy");
    assert!(csp.contains("connect-src 'self' https://keycloak.kakde.eu"));
    assert!(csp.contains("frame-src 'self' https://keycloak.kakde.eu"));
}

#[tokio::test]
async fn the_csp_permits_the_apps_real_resources() {
    // The step-36 incident (fonts/Monaco/inline) + step-38 (d2's ELK eval) + the RS wasm app.
    let tmp = tempfile::tempdir().unwrap();
    let (_, headers) = headers_of(app(tmp.path()), "/api/health").await;
    let csp = value(&headers, "content-security-policy");
    assert!(csp.contains("'unsafe-inline'"));
    assert!(csp.contains("'unsafe-eval'"));
    assert!(csp.contains("'wasm-unsafe-eval'"));
    assert!(csp.contains("worker-src 'self' blob:"));
    assert!(csp.contains("https://fonts.googleapis.com"));
    assert!(csp.contains("font-src 'self' data: https://fonts.gstatic.com"));
}

#[tokio::test]
async fn stamps_errors_and_every_route_class_too() {
    let tmp = tempfile::tempdir().unwrap();

    // An error response (404 from a matched route).
    let (status, headers) = headers_of(app(tmp.path()), "/api/blog/no-such-post").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(value(&headers, "x-content-type-options"), "nosniff");

    // The /c4 proxy's degrade path (upstream down → 502) is stamped.
    let (status, headers) = headers_of(app(tmp.path()), "/c4/view").await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    assert_eq!(value(&headers, "x-content-type-options"), "nosniff");

    // The static SPA index is stamped.
    let dist = tmp.path().join("dist");
    fs::create_dir_all(&dist).unwrap();
    fs::write(dist.join("index.html"), "SPA").unwrap();
    let mut deps = common::deps(tmp.path());
    ISSUER.clone_into(&mut deps.ident.issuer);
    deps.static_root = dist.to_string_lossy().into_owned();
    let (status, headers) = headers_of(synapse_server::app(deps), "/synapse/deep/link").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(value(&headers, "x-content-type-options"), "nosniff");
}
