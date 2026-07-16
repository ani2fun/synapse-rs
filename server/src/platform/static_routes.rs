//! The SPA + static assets (oracle: `StaticRoutes`, step 19). The fallback ENUMERATES the
//! SPA's reserved first segments (kept in step with the client's `Page`) instead of a
//! catch-all — a trailing wildcard is greedy enough to shadow `/api` and `/media` (the
//! Cortex-inherited lesson). An absent dist mounts nothing: dev keeps Vite + the API-only
//! root. Hashed assets are immutable for a year; the index is `no-cache` so deploys show.

use std::path::{Path, PathBuf};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;

/// The client's reserved first segments — `Page::from_segments`'s app-map, mirrored.
const SPA_SEGMENTS: [&str; 4] = ["synapse", "blog", "account", "admin"];

const INDEX_CACHE: &str = "no-cache";
const ASSET_CACHE: &str = "public, max-age=31536000, immutable";

pub struct StaticRoutes {
    root: PathBuf,
}

impl StaticRoutes {
    pub fn new(static_root: impl AsRef<Path>) -> Self {
        Self {
            root: static_root.as_ref().to_path_buf(),
        }
    }

    /// A production dist is present — dev (no dist) mounts nothing.
    pub fn enabled(&self) -> bool {
        self.root.is_dir()
    }

    pub fn routes(&self) -> Router {
        if !self.enabled() {
            return Router::new();
        }
        let mut router = Router::new()
            .route("/", get(index))
            .route("/index.html", get(index))
            .route("/silent-check-sso.html", get(silent_sso))
            .route("/assets/{*rest}", get(asset));
        for segment in SPA_SEGMENTS {
            router = router
                .route(&format!("/{segment}"), get(index))
                .route(&format!("/{segment}/{{*rest}}"), get(index));
        }
        router.with_state(self.root.clone())
    }
}

async fn index(state: axum::extract::State<PathBuf>) -> Response {
    serve(&state, "index.html", "text/html; charset=utf-8", INDEX_CACHE).await
}

async fn silent_sso(state: axum::extract::State<PathBuf>) -> Response {
    serve(
        &state,
        "silent-check-sso.html",
        "text/html; charset=utf-8",
        INDEX_CACHE,
    )
    .await
}

async fn asset(
    state: axum::extract::State<PathBuf>,
    axum::extract::Path(rest): axum::extract::Path<String>,
) -> Response {
    let rel = format!("assets/{rest}");
    serve(&state, &rel, content_type_of(&rel), ASSET_CACHE).await
}

/// Traversal-guarded file response: the REAL path must stay under the real root.
async fn serve(root: &Path, rel: &str, content_type: &'static str, cache: &'static str) -> Response {
    let root = root.to_path_buf();
    let rel = rel.to_owned();
    let bytes = tokio::task::spawn_blocking(move || {
        let root_real = root.canonicalize().ok()?;
        let target = root.join(&rel).canonicalize().ok()?;
        if target.starts_with(&root_real) && target.is_file() {
            std::fs::read(target).ok()
        } else {
            None
        }
    })
    .await
    .ok()
    .flatten();
    match bytes {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, HeaderValue::from_static(content_type))
            .header(header::CACHE_CONTROL, HeaderValue::from_static(cache))
            .body(Body::from(bytes))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn content_type_of(rel: &str) -> &'static str {
    match Path::new(rel).extension().and_then(|ext| ext.to_str()) {
        Some("js") => "text/javascript",
        Some("css") => "text/css",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
