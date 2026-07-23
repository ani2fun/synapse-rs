//! The GitHub REST shapes the forge adapter sends and decodes. Deliberately partial — every
//! struct names only the fields the adapter actually uses, so an unrelated change to GitHub's
//! (large) payloads cannot break a decode.

use serde::{Deserialize, Serialize};

// ── refs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RefResponse {
    pub object: RefObject,
}

#[derive(Debug, Deserialize)]
pub struct RefObject {
    pub sha: String,
}

#[derive(Debug, Serialize)]
pub struct CreateRef<'a> {
    /// Fully qualified — `refs/heads/<branch>`.
    #[serde(rename = "ref")]
    pub ref_name: &'a str,
    pub sha: &'a str,
}

// ── contents ─────────────────────────────────────────────────────────────────

/// `GET /contents/{path}` on a FILE. A directory answers with an array instead, which fails this
/// decode — correctly, since a directory is not something this adapter can commit over.
#[derive(Debug, Deserialize)]
pub struct ContentsResponse {
    /// The BLOB sha, which is what `PUT /contents` wants for an update — not a commit sha.
    pub sha: String,
}

#[derive(Debug, Serialize)]
pub struct PutContents<'a> {
    pub message: &'a str,
    /// Base64, no line breaks.
    pub content: String,
    pub branch: &'a str,
    /// The blob sha being replaced. Absent creates a new file; STALE is a 409, which is the
    /// forge's own concurrency check and the one that actually guards the branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct PutContentsResponse {
    pub commit: CommitRef,
}

#[derive(Debug, Deserialize)]
pub struct CommitRef {
    pub sha: String,
}

// ── pull requests ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CreatePull<'a> {
    pub title: &'a str,
    /// `owner:branch` for a fork, a bare branch for the same repository.
    pub head: &'a str,
    pub base: &'a str,
    pub body: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct PullResponse {
    pub number: u64,
    pub html_url: String,
    /// `"open" | "closed"` — a merged pull request reads as closed, hence `merged` below.
    #[serde(default)]
    pub state: String,
    /// Present on the single-pull-request response.
    #[serde(default)]
    pub merged: bool,
    /// Present on both the list and the detail responses; a merged pull request has it set.
    #[serde(default)]
    pub merged_at: Option<String>,
}

impl PullResponse {
    pub fn is_merged(&self) -> bool {
        self.merged || self.merged_at.is_some()
    }
}

/// GitHub's error envelope — the `message` is worth surfacing verbatim, because it is usually the
/// exact reason ("No commits between main and …", "Reference already exists").
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    #[serde(default)]
    pub message: String,
}
