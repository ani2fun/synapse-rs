//! The Postgres allowlist adapter (oracle: `PostgresSubmissionAllowlist`). One probe today;
//! the management verbs (list/grant/revoke) join with the admin-panel step.

use sqlx::PgPool;
use sqlx::Row;

use crate::submission::application::{SubmissionAllowlist, SubmissionError};

pub struct PostgresSubmissionAllowlist {
    pool: PgPool,
}

impl PostgresSubmissionAllowlist {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SubmissionAllowlist for PostgresSubmissionAllowlist {
    async fn is_allowed(&self, username: &str) -> Result<bool, SubmissionError> {
        let row = sqlx::query("select 1 as one from submission_allowlist where username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SubmissionError::StoreFailed(e.to_string()))?;
        let allowed = row.is_some_and(|r| r.get::<i32, _>("one") == 1);
        tracing::debug!(username, allowed, "submission allowlist checked");
        Ok(allowed)
    }
}
