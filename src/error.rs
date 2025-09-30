//! Custom error types exposed across the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// The primary error type for the application.
#[derive(Debug, Error)]
pub enum AppError {
    /// Error related to configuration loading or parsing.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error related to database operations.
    #[error("Database error: {0}")]
    DatabaseError(#[from] crate::database::DbError),

    /// Error from an external AI provider's API.
    #[error("API provider error: {0}")]
    ApiError(String),

    /// Network error while communicating with an external service.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Represents a scenario where no provider was available to handle a request.
    #[error("No available AI provider to handle the request. Run 'freegin-ai status' to check provider health and 'freegin-ai list-services' to verify configuration.")]
    NoProviderAvailable,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConfigError(msg) | AppError::ApiError(msg) | AppError::NetworkError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::DatabaseError(db_err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal database issue: {db_err}"),
            ),
            AppError::NoProviderAvailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "All AI providers are currently unavailable or have exceeded their quotas."
                    .to_string(),
            ),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
