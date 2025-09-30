//! Provider health tracking and availability management.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use sqlx::Row;

use crate::{
    database::{DbError, DbPool},
    error::AppError,
    providers::Provider,
};

/// Manages provider health status and availability.
#[derive(Clone, Debug)]
pub struct HealthTracker {
    pool: Arc<DbPool>,
}

/// Health status for a provider.
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Provider name.
    pub provider: Provider,
    /// Current status.
    pub status: HealthStatus,
    /// Last error message.
    pub last_error: Option<String>,
    /// When the last error occurred.
    pub last_error_at: Option<DateTime<Utc>>,
    /// When to retry (if backing off).
    pub retry_after: Option<DateTime<Utc>>,
    /// Number of consecutive failures.
    pub consecutive_failures: i64,
    /// Last successful call.
    pub last_success_at: Option<DateTime<Utc>>,
}

/// Health status of a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Provider is available.
    Available,
    /// Provider is experiencing issues (rate limit, temporary error).
    Degraded,
    /// Provider is unavailable (out of credits, auth failure, etc.).
    Unavailable,
}

impl HealthStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Available => "available",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unavailable => "unavailable",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "degraded" => HealthStatus::Degraded,
            "unavailable" => HealthStatus::Unavailable,
            _ => HealthStatus::Available,
        }
    }
}

impl HealthTracker {
    /// Creates a new health tracker.
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Records a successful API call.
    pub async fn record_success(&self, provider: Provider) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"INSERT INTO provider_health (provider, status, consecutive_failures, last_success_at, updated_at)
               VALUES (?, 'available', 0, ?, ?)
               ON CONFLICT(provider) DO UPDATE SET
                   status = 'available',
                   consecutive_failures = 0,
                   last_success_at = excluded.last_success_at,
                   updated_at = excluded.updated_at"#,
        )
        .bind(provider.as_str())
        .bind(&now)
        .bind(&now)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();
        Ok(())
    }

    /// Records a failed API call with error classification.
    pub async fn record_failure(
        &self,
        provider: Provider,
        error_message: &str,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        let error_type = classify_error(error_message);

        let (status, retry_after) = match error_type {
            ErrorType::RateLimit => {
                // Exponential backoff starting at 1 minute
                let backoff = calculate_backoff(1);
                (HealthStatus::Degraded, Some(now + Duration::minutes(backoff)))
            }
            ErrorType::OutOfCredits | ErrorType::AuthFailure => {
                // Don't retry for 24 hours
                (HealthStatus::Unavailable, Some(now + Duration::hours(24)))
            }
            ErrorType::ServiceUnavailable => {
                // Retry after 5 minutes
                (HealthStatus::Degraded, Some(now + Duration::minutes(5)))
            }
            ErrorType::Transient => {
                // Quick retry, 30 seconds
                (HealthStatus::Degraded, Some(now + Duration::seconds(30)))
            }
        };

        let now_str = now.to_rfc3339();
        let retry_str = retry_after.map(|dt| dt.to_rfc3339());

        let result = sqlx::query(
            r#"INSERT INTO provider_health (provider, status, last_error, last_error_at, retry_after, consecutive_failures, updated_at)
               VALUES (?, ?, ?, ?, ?, 1, ?)
               ON CONFLICT(provider) DO UPDATE SET
                   status = excluded.status,
                   last_error = excluded.last_error,
                   last_error_at = excluded.last_error_at,
                   retry_after = excluded.retry_after,
                   consecutive_failures = provider_health.consecutive_failures + 1,
                   updated_at = excluded.updated_at"#,
        )
        .bind(provider.as_str())
        .bind(status.as_str())
        .bind(error_message)
        .bind(&now_str)
        .bind(retry_str)
        .bind(&now_str)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();
        Ok(())
    }

    /// Checks if a provider is available for use.
    pub async fn is_available(&self, provider: Provider) -> Result<bool, AppError> {
        let health = self.get_health(provider).await?;

        match health.status {
            HealthStatus::Available => Ok(true),
            HealthStatus::Degraded | HealthStatus::Unavailable => {
                // Check if retry_after has passed
                if let Some(retry_after) = health.retry_after {
                    Ok(Utc::now() >= retry_after)
                } else {
                    // No retry_after set, consider degraded as available
                    Ok(health.status == HealthStatus::Degraded)
                }
            }
        }
    }

    /// Gets the health status for a provider.
    pub async fn get_health(&self, provider: Provider) -> Result<ProviderHealth, AppError> {
        let row = sqlx::query(
            r#"SELECT provider, status, last_error, last_error_at, retry_after, consecutive_failures, last_success_at
               FROM provider_health
               WHERE provider = ?"#,
        )
        .bind(provider.as_str())
        .fetch_optional(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        if let Some(row) = row {
            Ok(ProviderHealth {
                provider,
                status: HealthStatus::from_str(row.get("status")),
                last_error: row.get("last_error"),
                last_error_at: row
                    .get::<Option<String>, _>("last_error_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                retry_after: row
                    .get::<Option<String>, _>("retry_after")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                consecutive_failures: row.get("consecutive_failures"),
                last_success_at: row
                    .get::<Option<String>, _>("last_success_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        } else {
            // No health record means provider hasn't been used yet
            Ok(ProviderHealth {
                provider,
                status: HealthStatus::Available,
                last_error: None,
                last_error_at: None,
                retry_after: None,
                consecutive_failures: 0,
                last_success_at: None,
            })
        }
    }

    /// Gets health for all providers.
    pub async fn get_all_health(&self) -> Result<Vec<ProviderHealth>, AppError> {
        let providers = [
            Provider::Groq,
            Provider::DeepSeek,
            Provider::Together,
            Provider::HuggingFace,
            Provider::Google,
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Cohere,
        ];

        let mut results = Vec::new();
        for provider in providers {
            results.push(self.get_health(provider).await?);
        }
        Ok(results)
    }
}

/// Error type classification for backoff strategy.
enum ErrorType {
    /// Rate limit exceeded.
    RateLimit,
    /// Out of credits/quota.
    OutOfCredits,
    /// Authentication failure.
    AuthFailure,
    /// Service temporarily unavailable.
    ServiceUnavailable,
    /// Transient network error.
    Transient,
}

/// Classifies error messages to determine handling strategy.
fn classify_error(error_msg: &str) -> ErrorType {
    let error_lower = error_msg.to_lowercase();

    // Rate limit patterns
    if error_lower.contains("rate limit")
        || error_lower.contains("too many requests")
        || error_lower.contains("429")
    {
        return ErrorType::RateLimit;
    }

    // Out of credits patterns
    if error_lower.contains("insufficient credits")
        || error_lower.contains("quota exceeded")
        || error_lower.contains("out of credits")
        || error_lower.contains("billing")
        || error_lower.contains("payment required")
        || error_lower.contains("402")
    {
        return ErrorType::OutOfCredits;
    }

    // Auth failure patterns
    if error_lower.contains("unauthorized")
        || error_lower.contains("forbidden")
        || error_lower.contains("invalid api key")
        || error_lower.contains("invalid token")
        || error_lower.contains("authentication failed")
        || error_lower.contains("401")
        || error_lower.contains("403")
    {
        return ErrorType::AuthFailure;
    }

    // Service unavailable patterns
    if error_lower.contains("service unavailable")
        || error_lower.contains("502")
        || error_lower.contains("503")
        || error_lower.contains("504")
        || error_lower.contains("gateway")
    {
        return ErrorType::ServiceUnavailable;
    }

    // Default to transient
    ErrorType::Transient
}

/// Calculates exponential backoff in minutes.
fn calculate_backoff(consecutive_failures: i64) -> i64 {
    // Start at 1 minute, double each time, cap at 60 minutes
    let backoff = 2_i64.pow(consecutive_failures.min(6) as u32);
    backoff.min(60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification_rate_limit() {
        assert!(matches!(
            classify_error("Rate limit exceeded"),
            ErrorType::RateLimit
        ));
        assert!(matches!(
            classify_error("Too many requests"),
            ErrorType::RateLimit
        ));
        assert!(matches!(classify_error("HTTP 429"), ErrorType::RateLimit));
    }

    #[test]
    fn test_error_classification_out_of_credits() {
        assert!(matches!(
            classify_error("Insufficient credits"),
            ErrorType::OutOfCredits
        ));
        assert!(matches!(
            classify_error("Quota exceeded"),
            ErrorType::OutOfCredits
        ));
        assert!(matches!(
            classify_error("Payment required"),
            ErrorType::OutOfCredits
        ));
    }

    #[test]
    fn test_error_classification_auth_failure() {
        assert!(matches!(
            classify_error("Unauthorized"),
            ErrorType::AuthFailure
        ));
        assert!(matches!(
            classify_error("Invalid API key"),
            ErrorType::AuthFailure
        ));
        assert!(matches!(
            classify_error("HTTP 401 Forbidden"),
            ErrorType::AuthFailure
        ));
    }

    #[test]
    fn test_error_classification_service_unavailable() {
        assert!(matches!(
            classify_error("Service unavailable"),
            ErrorType::ServiceUnavailable
        ));
        assert!(matches!(
            classify_error("Gateway timeout 504"),
            ErrorType::ServiceUnavailable
        ));
    }

    #[test]
    fn test_error_classification_transient() {
        assert!(matches!(
            classify_error("Connection reset by peer"),
            ErrorType::Transient
        ));
        assert!(matches!(
            classify_error("Some unknown error"),
            ErrorType::Transient
        ));
    }

    #[test]
    fn test_exponential_backoff() {
        assert_eq!(calculate_backoff(1), 2);
        assert_eq!(calculate_backoff(2), 4);
        assert_eq!(calculate_backoff(3), 8);
        assert_eq!(calculate_backoff(4), 16);
        assert_eq!(calculate_backoff(5), 32);
        assert_eq!(calculate_backoff(6), 60); // Capped at 60
        assert_eq!(calculate_backoff(7), 60); // Still capped
    }
}