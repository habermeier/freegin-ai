//! Database interaction logic using `sqlx` and SQLite.
//!
//! This module keeps all persistence logic in one place so other modules can
//! depend on well-defined functions instead of scattering SQL across the code.

use std::path::Path;

use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Pool, Sqlite,
};
use thiserror::Error;

/// Custom error type for database operations.
#[derive(Debug, Error)]
pub enum DbError {
    /// Represents a failure to connect to the database.
    #[error("Failed to connect to the database: {0}")]
    ConnectionFailed(sqlx::Error),

    /// Represents a failure during a database query.
    #[error("Database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

/// A handle to the database connection pool.
pub type DbPool = Pool<Sqlite>;

/// Initializes the database connection pool.
///
/// # Arguments
/// * `database_url` - The SQLite connection string.
pub async fn init_db(database_url: &str) -> Result<DbPool, DbError> {
    create_sqlite_parent_dir(database_url);

    let options = SqliteConnectOptions::from_str(database_url)
        .map_err(DbError::ConnectionFailed)?
        .create_if_missing(true)
        .disable_statement_logging();

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(DbError::ConnectionFailed)
}

/// Ensures the database schema exists.
pub async fn ensure_schema(pool: &DbPool) -> Result<(), DbError> {
    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_credentials (
            provider TEXT PRIMARY KEY,
            nonce BLOB NOT NULL,
            ciphertext BLOB NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_usage (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            model TEXT,
            success INTEGER NOT NULL,
            latency_ms INTEGER NOT NULL,
            error_message TEXT,
            prompt_tokens INTEGER,
            completion_tokens INTEGER,
            total_tokens INTEGER,
            input_cost_micros INTEGER,
            output_cost_micros INTEGER,
            total_cost_micros INTEGER,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_models (
            provider TEXT NOT NULL,
            workload TEXT NOT NULL,
            model TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            priority INTEGER NOT NULL DEFAULT 100,
            rationale TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(provider, workload, model)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_model_suggestions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            workload TEXT NOT NULL,
            model TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            rationale TEXT,
            metadata TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(provider, workload, model)
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_health (
            provider TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'available',
            last_error TEXT,
            last_error_at TEXT,
            retry_after TEXT,
            consecutive_failures INTEGER NOT NULL DEFAULT 0,
            last_success_at TEXT,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    // Migrate existing databases - add columns if they don't exist
    migrate_provider_usage_columns(pool).await?;

    // Create indexes for performance
    let result = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_provider_models_active
        ON provider_models(provider, workload, status, priority)
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_provider_model_suggestions
        ON provider_model_suggestions(provider, workload, status)
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    let result = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_provider_usage_provider_model_time
        ON provider_usage(provider, model, created_at)
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

    Ok(())
}

async fn migrate_provider_usage_columns(pool: &DbPool) -> Result<(), DbError> {
    // Check if 'model' column exists in provider_usage
    let check_result = sqlx::query("SELECT model FROM provider_usage LIMIT 1")
        .fetch_optional(pool)
        .await;

    if check_result.is_err() {
        // Column doesn't exist, add it
        let result = sqlx::query("ALTER TABLE provider_usage ADD COLUMN model TEXT")
            .execute(pool)
            .await
            .map_err(DbError::QueryFailed)?;
        let _ = result.rows_affected();
    }

    // Check and add cost tracking columns
    let columns = vec![
        "prompt_tokens",
        "completion_tokens",
        "total_tokens",
        "input_cost_micros",
        "output_cost_micros",
        "total_cost_micros",
    ];

    for column in columns {
        let check_query = format!("SELECT {} FROM provider_usage LIMIT 1", column);
        let check_result = sqlx::query(&check_query).fetch_optional(pool).await;

        if check_result.is_err() {
            let alter_query = format!("ALTER TABLE provider_usage ADD COLUMN {} INTEGER", column);
            let result = sqlx::query(&alter_query)
                .execute(pool)
                .await
                .map_err(DbError::QueryFailed)?;
            let _ = result.rows_affected();
        }
    }

    Ok(())
}

fn create_sqlite_parent_dir(database_url: &str) {
    if let Some(path) = extract_sqlite_path(database_url) {
        if let Some(parent) = path.parent() {
            if let Err(err) = std::fs::create_dir_all(parent) {
                eprintln!("freegin-ai: failed to create database directory {parent:?}: {err}");
            }
        }
    }
}

fn extract_sqlite_path(database_url: &str) -> Option<std::path::PathBuf> {
    let trimmed = database_url.strip_prefix("sqlite:")?;
    if trimmed.starts_with("memory") || trimmed == ":memory:" {
        return None;
    }
    let path = trimmed.trim_start_matches("//");
    if path.is_empty() {
        None
    } else {
        Some(Path::new(path).to_path_buf())
    }
}
