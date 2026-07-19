//! The Postgres readership adapter — modelled on `PostgresSubmissionAllowlist`, which is the
//! lightest store in the codebase and the right shape to copy: four verbs, a flat row, no ADT
//! flattening and no JSONB.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::Row;
use sqlx::postgres::PgRow;

use crate::insights::{InsightsError, LessonViewCount, LessonViewStore};

pub struct PostgresLessonViews {
    pool: PgPool,
}

impl PostgresLessonViews {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn store_failed(error: &sqlx::Error) -> InsightsError {
    InsightsError::StoreFailed(error.to_string())
}

fn count(row: &PgRow) -> LessonViewCount {
    LessonViewCount {
        lesson_path: row.get("lesson_path"),
        views: row.get::<i64, _>("views"),
        authed_views: row.get::<i64, _>("authed_views"),
        last_viewed: row.get::<DateTime<Utc>, _>("last_viewed"),
    }
}

impl LessonViewStore for PostgresLessonViews {
    async fn record(&self, lesson_path: &str, authed: bool) -> Result<(), InsightsError> {
        sqlx::query("insert into lesson_view (lesson_path, authed) values ($1, $2)")
            .bind(lesson_path)
            .bind(authed)
            .execute(&self.pool)
            .await
            .map_err(|e| store_failed(&e))?;
        Ok(())
    }

    async fn top(&self, limit: i64) -> Result<Vec<LessonViewCount>, InsightsError> {
        let rows = sqlx::query(
            "select lesson_path, \
                    count(*)                                as views, \
                    count(*) filter (where authed)          as authed_views, \
                    max(viewed_at)                          as last_viewed \
             from lesson_view \
             group by lesson_path \
             order by views desc, lesson_path \
             limit $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| store_failed(&e))?;
        Ok(rows.iter().map(count).collect())
    }
}
