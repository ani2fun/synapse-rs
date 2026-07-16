//! Shared IT plumbing: the real assembled router over a filesystem repo.

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use synapse_server::catalog::application::CatalogService;
use synapse_server::catalog::infrastructure::FileSystemContentRepository;
use synapse_server::execution::application::RunCodeService;
use synapse_server::execution::infrastructure::GoJudgeRunner;
use synapse_server::submission::application::SubmitSolution;
use synapse_server::submission::infrastructure::{FsProblemTests, PostgresSubmissionRepository};

/// The full app over a content root (integration tests drive the REAL stack, middleware and
/// all). A nonexistent root is valid — the catalog is simply empty.
#[allow(dead_code)] // each IT binary compiles common on its own; not all use every helper
pub fn app_over(content_root: &Path) -> Router {
    // Port 9 (discard) refuses connections — tests that need a live sandbox point the
    // executor elsewhere via `app_with_executor`.
    app_with(content_root, "http://127.0.0.1:9", None)
}

/// The full app with an explicit go-judge base URL.
#[allow(dead_code)] // each IT binary compiles common on its own; not all use every helper
pub fn app_with_executor(content_root: &Path, executor_url: &str) -> Router {
    app_with(content_root, executor_url, None)
}

/// The full app with an explicit database too (the gated Postgres ITs). Without one, a LAZY
/// pool pointed at a refusing port stands in — routes that never touch the store stay green.
pub fn app_with(content_root: &Path, executor_url: &str, pool: Option<sqlx::PgPool>) -> Router {
    let pool = pool.unwrap_or_else(|| {
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://nobody:nowhere@127.0.0.1:9/none")
            .unwrap_or_else(|e| unreachable!("lazy pools do not connect: {e}"))
    });
    let repo = FileSystemContentRepository::new(content_root, true);
    let runner = Arc::new(RunCodeService::new(GoJudgeRunner::new(executor_url)));
    let submit = Arc::new(SubmitSolution::new(
        Arc::new(PostgresSubmissionRepository::new(pool)),
        Arc::new(FsProblemTests::new(FileSystemContentRepository::new(
            content_root,
            true,
        ))),
        Arc::clone(&runner),
    ));
    synapse_server::app(Arc::new(CatalogService::new(repo)), runner, submit)
}
