//! Bridges the submission context's `SolvedRecorder` port to the `progress` context's store: an
//! accepted submission marks its lesson complete for the caller. Generic over the store PORT (not
//! the concrete adapter), so the dependency is on `progress::ProblemProgressStore`, an abstraction.
//! Errors are logged and swallowed — the port's fire-and-forget contract says a progress-store
//! hiccup must never fail (or block) judging.

use std::sync::Arc;

use crate::progress::ProblemProgressStore;
use crate::submission::application::SolvedRecorder;

pub struct ProgressRecorderAdapter<P> {
    progress: Arc<P>,
}

impl<P> ProgressRecorderAdapter<P> {
    pub fn new(progress: Arc<P>) -> Self {
        Self { progress }
    }
}

impl<P: ProblemProgressStore> SolvedRecorder for ProgressRecorderAdapter<P> {
    async fn record_solved(&self, user_id: &str, lesson_path: &str) {
        if let Err(error) = self.progress.mark(user_id, lesson_path).await {
            tracing::warn!(%error, lesson = %lesson_path, "could not record solved progress");
        }
    }
}
