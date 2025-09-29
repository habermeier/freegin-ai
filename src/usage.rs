//! Provider usage logging utilities.

use std::sync::Arc;

use chrono::Utc;

use crate::{
    database::{DbError, DbPool},
    error::AppError,
    providers::Provider,
};

/// Records provider invocation metrics for routing decisions.
#[derive(Clone, Debug)]
pub struct UsageLogger {
    pool: Arc<DbPool>,
}

impl UsageLogger {
    /// Creates a new usage logger backed by the SQLite pool.
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Persists a usage record.
    pub async fn log(
        &self,
        provider: Provider,
        success: bool,
        latency_ms: i64,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let success_flag = if success { 1 } else { 0 };

        let result = sqlx::query(
            r#"INSERT INTO provider_usage (provider, success, latency_ms, error_message, created_at)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(provider.as_str())
        .bind(success_flag)
        .bind(latency_ms)
        .bind(error_message)
        .bind(now)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();

        Ok(())
    }
}
