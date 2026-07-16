//! The submission HTTP surface (oracle: `SubmissionRoutes`, step-15 scope): POST → **202** +
//! poll, public GET, list newest-first. DTO↔domain mapping lives ONLY here. Identity grafts the
//! bearer seams (owner scoping, delete) in its own step.

mod dto;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use synapse_shared::api::ApiError;
use synapse_shared::submission::{SubmissionAcceptedDto, SubmissionDto, SubmitRequestDto};
use uuid::Uuid;

use crate::catalog::infrastructure::FileSystemContentRepository;
use crate::execution::infrastructure::GoJudgeRunner;
use crate::submission::application::SubmitSolution;
use crate::submission::domain::SubmissionId;
use crate::submission::infrastructure::{FsProblemTests, PostgresSubmissionRepository};

/// The production wiring: Postgres store · filesystem suites · the go-judge-backed runner.
pub type LiveSubmitSolution =
    SubmitSolution<PostgresSubmissionRepository, FsProblemTests<FileSystemContentRepository>, GoJudgeRunner>;

type SubmitState = State<Arc<LiveSubmitSolution>>;
type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ApiError>)>;

pub fn routes(service: Arc<LiveSubmitSolution>) -> Router {
    Router::new()
        .route("/api/submissions", post(submit_solution).get(list_submissions))
        .route("/api/submissions/{id}", get(get_submission))
        .with_state(service)
}

/// Submit a solution — stored and judged in the background; poll the returned id.
#[utoipa::path(
    post,
    path = "/api/submissions",
    operation_id = "submitSolution",
    request_body = SubmitRequestDto,
    responses(
        (status = 202, description = "Stored; judging in background", body = SubmissionAcceptedDto),
        (status = 404, description = "Not a problem", body = ApiError)
    )
)]
pub(crate) async fn submit_solution(
    State(service): SubmitState,
    Json(request): Json<SubmitRequestDto>,
) -> ApiResult<SubmissionAcceptedDto> {
    tracing::info!(path = request.path.join("/"), "POST /api/submissions");
    match service
        .submit(request.path, request.language, request.source)
        .await
    {
        Ok(id) => Ok((
            StatusCode::ACCEPTED,
            Json(SubmissionAcceptedDto { id: id.to_string() }),
        )),
        Err(error) => Err(dto::to_error(&error)),
    }
}

/// Poll one submission (public — ids are unguessable UUIDs).
#[utoipa::path(
    get,
    path = "/api/submissions/{id}",
    operation_id = "getSubmission",
    params(("id" = String, Path, description = "The submission id")),
    responses(
        (status = 200, description = "The submission", body = SubmissionDto),
        (status = 400, description = "Not a submission id", body = ApiError),
        (status = 404, description = "Unknown submission", body = ApiError)
    )
)]
pub(crate) async fn get_submission(
    State(service): SubmitState,
    Path(raw): Path<String>,
) -> ApiResult<SubmissionDto> {
    let Ok(id) = raw.parse::<Uuid>() else {
        return Err(dto::bad_id(&raw));
    };
    match service.get(SubmissionId(id)).await {
        Ok(submission) => Ok((StatusCode::OK, Json(dto::to_dto(&submission)))),
        Err(error) => Err(dto::to_error(&error)),
    }
}

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    path: String,
}

/// Submissions for a lesson, newest first. (The identity step makes this private — scoped to
/// the caller — exactly as the oracle staged it.)
#[utoipa::path(
    get,
    path = "/api/submissions",
    operation_id = "listSubmissions",
    params(("path" = String, Query, description = "The lesson's directory-mirror path")),
    responses((status = 200, description = "Newest first", body = [SubmissionDto]))
)]
pub(crate) async fn list_submissions(
    State(service): SubmitState,
    Query(query): Query<ListQuery>,
) -> ApiResult<Vec<SubmissionDto>> {
    let segments: Vec<String> = query
        .path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    match service.list_for(&segments, None).await {
        Ok(submissions) => Ok((
            StatusCode::OK,
            Json(submissions.iter().map(dto::to_dto).collect()),
        )),
        Err(error) => Err(dto::to_error(&error)),
    }
}
