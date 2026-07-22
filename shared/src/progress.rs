//! The progress wire contract. Per-user lesson/problem completion — the account-owned truth
//! behind the reader's ✓ ticks. Lesson paths travel as full `/`-joined strings (the same shape
//! the reader stores locally and the sidebar links carry), never the `Vec<String>` segments the
//! submission API uses.

use serde::{Deserialize, Serialize};

/// `GET /api/progress` — every lesson path the caller has completed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ProgressListDto {
    pub completed: Vec<String>,
}

/// `POST /api/progress` body — mark one lesson path complete for the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct MarkProgressRequestDto {
    pub path: String,
}
