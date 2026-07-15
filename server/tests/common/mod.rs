//! Shared IT plumbing: the real assembled router over a filesystem repo.

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use synapse_server::catalog::application::CatalogService;
use synapse_server::catalog::infrastructure::FileSystemContentRepository;
use synapse_server::execution::application::RunCodeService;
use synapse_server::execution::infrastructure::GoJudgeRunner;

/// The full app over a content root (integration tests drive the REAL stack, middleware and
/// all). A nonexistent root is valid — the catalog is simply empty.
pub fn app_over(content_root: &Path) -> Router {
    // Port 9 (discard) refuses connections — run tests that need a live sandbox point the
    // executor elsewhere via `app_with_executor`.
    app_with_executor(content_root, "http://127.0.0.1:9")
}

/// The full app with an explicit go-judge base URL (route ITs stub go-judge with a local
/// axum server and point this at it).
pub fn app_with_executor(content_root: &Path, executor_url: &str) -> Router {
    let repo = FileSystemContentRepository::new(content_root, true);
    let run = Arc::new(RunCodeService::new(GoJudgeRunner::new(executor_url)));
    synapse_server::app(Arc::new(CatalogService::new(repo)), run)
}
