//! `POST /api/run` (oracle: `ExecutionRoutes` + `ExecutionDtos`, step 10). A badly-running
//! program is a 200 with a non-`Accepted` status; the error channel is for the CALLER's
//! mistakes (422/413) and the BACKEND's failures (503/502). The rate-limit/identity gate
//! grafts on in its own step, as it did in the oracle (step 19).

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use synapse_shared::api::ApiError;
use synapse_shared::execution::{RunRequest, RunResult};

use crate::execution::application::{ExecutionError, RunCodeService};
use crate::execution::infrastructure::GoJudgeRunner;

pub type LiveRunService = RunCodeService<GoJudgeRunner>;

pub fn routes(service: Arc<LiveRunService>) -> Router {
    Router::new()
        .route("/api/run", post(run_code))
        .with_state(service)
}

/// Run one snippet in the sandbox.
#[utoipa::path(
    post,
    path = "/api/run",
    operation_id = "runCode",
    request_body = RunRequest,
    responses(
        (status = 200, description = "The run's outcome (including failed programs)", body = RunResult),
        (status = 422, description = "Unknown language", body = ApiError),
        (status = 413, description = "Payload over the byte caps", body = ApiError),
        (status = 502, description = "Backend failed", body = ApiError),
        (status = 503, description = "Backend unavailable", body = ApiError)
    )
)]
pub(crate) async fn run_code(
    State(service): State<Arc<LiveRunService>>,
    Json(request): Json<RunRequest>,
) -> Result<Json<RunResult>, (StatusCode, Json<ApiError>)> {
    tracing::info!(language = request.language, "POST /api/run");
    match service.run(&request).await {
        Ok(result) => Ok(Json(result)),
        Err(error) => Err(to_error(&error)),
    }
}

fn to_error(error: &ExecutionError) -> (StatusCode, Json<ApiError>) {
    let (status, message, detail, hint) = match error {
        ExecutionError::UnknownLanguage(alias) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Language '{alias}' is not runnable"),
            None,
            None,
        ),
        ExecutionError::PayloadTooLarge { field, bytes, limit } => (
            StatusCode::PAYLOAD_TOO_LARGE,
            format!("{field} too large"),
            Some(format!("{bytes} bytes exceeds the {limit}-byte cap")),
            None,
        ),
        ExecutionError::BackendUnavailable(detail) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Execution backend unavailable".to_owned(),
            Some(detail.clone()),
            Some("Is go-judge running? Set EXECUTOR_URL.".to_owned()),
        ),
        ExecutionError::BackendFailed(detail) => (
            StatusCode::BAD_GATEWAY,
            "Execution backend failed".to_owned(),
            Some(detail.clone()),
            None,
        ),
    };
    (
        status,
        Json(ApiError {
            error: message,
            detail,
            hint,
        }),
    )
}
