//! The AUTHORING context: a reader who spots a problem in a lesson proposes the fix from inside
//! Synapse, and the server opens a pull request against the content repository for them.
//!
//! The shape of the trade this context makes: contributors get an editor and a preview instead of
//! a git tutorial, while the content repository stays the single source of truth and every word
//! still passes a human review before it ships. Nothing here writes to the served content tree —
//! the only way a change reaches readers is a merge, followed by the git-sync sidecar.

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;
