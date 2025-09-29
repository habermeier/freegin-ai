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
            success INTEGER NOT NULL,
            latency_ms INTEGER NOT NULL,
            error_message TEXT,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DbError::QueryFailed)?;

    let _ = result.rows_affected();

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
