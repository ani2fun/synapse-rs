//! The progress HTTP surface: per-user completion. POST marks one lesson complete (bearer
//! REQUIRED — never silently anonymous), GET lists the caller's completed paths (anonymous → `[]`,
//! store untouched), DELETE resets the caller's progress and NOTHING else (submissions survive).
//! The bearer skeleton is `identity::http::optional_user`; only the anonymous policy + the per-verb
//! 401 copy stay local (the submission surface's exact shape).

use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use synapse_shared::api::ApiError;
use synapse_shared::progress::{MarkProgressRequestDto, ProgressListDto};
use synapse_shared::submission::DeleteResultDto;

use crate::identity::http::LiveIdentityService;
use crate::progress::{PostgresProblemProgress, ProblemProgressStore, ProgressError};

#[derive(Clone)]
pub struct ProgressRoutesState {
    pub progress: Arc<PostgresProblemProgress>,
    pub identity: Arc<LiveIdentityService>,
}

type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<ApiError>)>;

pub fn routes(state: ProgressRoutesState) -> Router {
    Router::new()
        .route(
            "/api/progress",
            post(mark_progress).get(list_progress).delete(reset_progress),
        )
        .with_state(state)
}

/// The progress-local name for the shared bearer skeleton (`identity::http::optional_user`, which
/// owns the never-silently-anonymous rule).
async fn caller_user(
    state: &ProgressRoutesState,
    headers: &HeaderMap,
) -> Result<Option<crate::identity::domain::AuthenticatedUser>, (StatusCode, Json<ApiError>)> {
    crate::identity::http::optional_user(&state.identity, headers).await
}

fn needs_token(verb: &str) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiError {
            error: format!("{verb} requires a bearer token"),
            detail: Some("Sign in first".to_owned()),
            hint: None,
        }),
    )
}

fn store_error(error: &ProgressError) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            error: "Progress unavailable".to_owned(),
            detail: Some(error.to_string()),
            hint: None,
        }),
    )
}

/// Mark one lesson complete for the caller (idempotent), then return the caller's full completed
/// list so the client has an authoritative post-mark snapshot. Bearer required.
#[utoipa::path(
    post,
    path = "/api/progress",
    operation_id = "markProgress",
    request_body = MarkProgressRequestDto,
    responses(
        (status = 200, description = "Marked complete; the caller's completed paths", body = ProgressListDto),
        (status = 401, description = "Anonymous", body = ApiError),
        (status = 500, description = "Store failed", body = ApiError)
    )
)]
pub(crate) async fn mark_progress(
    State(state): State<ProgressRoutesState>,
    headers: HeaderMap,
    Json(request): Json<MarkProgressRequestDto>,
) -> ApiResult<ProgressListDto> {
    let Some(user) = caller_user(&state, &headers).await? else {
        return Err(needs_token("Saving progress"));
    };
    state
        .progress
        .mark(&user.id.0, &request.path)
        .await
        .map_err(|e| store_error(&e))?;
    match state.progress.list_for(&user.id.0).await {
        Ok(completed) => Ok((StatusCode::OK, Json(ProgressListDto { completed }))),
        Err(error) => Err(store_error(&error)),
    }
}

/// The caller's completed lesson paths — private: anonymous callers get `[]` and the store is
/// never touched (mirrors `list_submissions`).
#[utoipa::path(
    get,
    path = "/api/progress",
    operation_id = "listProgress",
    responses((status = 200, description = "The caller's completed lesson paths", body = ProgressListDto))
)]
pub(crate) async fn list_progress(
    State(state): State<ProgressRoutesState>,
    headers: HeaderMap,
) -> ApiResult<ProgressListDto> {
    let Some(user) = caller_user(&state, &headers).await? else {
        return Ok((
            StatusCode::OK,
            Json(ProgressListDto {
                completed: Vec::new(),
            }),
        ));
    };
    match state.progress.list_for(&user.id.0).await {
        Ok(completed) => Ok((StatusCode::OK, Json(ProgressListDto { completed }))),
        Err(error) => Err(store_error(&error)),
    }
}

/// Reset all of the caller's progress ("reset progress"). Clears only these rows — the caller's
/// submission history is a separate store and survives.
#[utoipa::path(
    delete,
    path = "/api/progress",
    operation_id = "resetProgress",
    responses(
        (status = 200, description = "Progress cleared", body = DeleteResultDto),
        (status = 401, description = "Anonymous", body = ApiError),
        (status = 500, description = "Store failed", body = ApiError)
    )
)]
pub(crate) async fn reset_progress(
    State(state): State<ProgressRoutesState>,
    headers: HeaderMap,
) -> ApiResult<DeleteResultDto> {
    let Some(user) = caller_user(&state, &headers).await? else {
        return Err(needs_token("Resetting progress"));
    };
    match state.progress.reset_for(&user.id.0).await {
        Ok(deleted) => Ok((StatusCode::OK, Json(DeleteResultDto { deleted }))),
        Err(error) => Err(store_error(&error)),
    }
}
