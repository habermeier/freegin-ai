//! Model catalog management for providers and workloads.

use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    database::{DbError, DbPool},
    error::AppError,
    models::Workload,
    providers::Provider,
};

/// Store for managing model catalog entries and suggestions.
#[derive(Clone, Debug)]
pub struct CatalogStore {
    pool: Arc<DbPool>,
}

impl CatalogStore {
    /// Creates a new catalog store with the given database pool.
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Lists all models matching the optional provider and workload filters.
    pub async fn list_models(
        &self,
        provider: Option<Provider>,
        workload: Option<Workload>,
    ) -> Result<Vec<ModelEntry>, AppError> {
        let mut query = String::from(
            "SELECT provider, workload, model, status, priority, rationale, metadata, created_at, updated_at \
             FROM provider_models",
        );
        let mut filters = Vec::new();
        let mut args: Vec<String> = Vec::new();

        if let Some(p) = provider {
            filters.push("provider = ?".to_string());
            args.push(p.as_str().to_string());
        }
        if let Some(w) = workload {
            filters.push("workload = ?".to_string());
            args.push(workload_key(w));
        }
        if !filters.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&filters.join(" AND "));
        }
        query.push_str(" ORDER BY provider, workload, priority ASC, updated_at DESC");

        let mut sql = sqlx::query(&query);
        for arg in args {
            sql = sql.bind(arg);
        }

        let rows = sql
            .fetch_all(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| ModelEntry {
                provider: Provider::from_alias(row.get::<String, _>("provider").as_str())
                    .unwrap_or(Provider::HuggingFace),
                workload: workload_from_key(row.get::<String, _>("workload").as_str()),
                model: row.get("model"),
                status: row.get("status"),
                priority: row.get("priority"),
                rationale: row.get("rationale"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    /// Lists all suggestions matching the optional provider and workload filters.
    pub async fn list_suggestions(
        &self,
        provider: Option<Provider>,
        workload: Option<Workload>,
    ) -> Result<Vec<SuggestionEntry>, AppError> {
        let mut query = String::from(
            "SELECT id, provider, workload, model, status, rationale, metadata, created_at, updated_at \
             FROM provider_model_suggestions",
        );
        let mut filters = Vec::new();
        let mut args: Vec<String> = Vec::new();
        if let Some(p) = provider {
            filters.push("provider = ?".to_string());
            args.push(p.as_str().to_string());
        }
        if let Some(w) = workload {
            filters.push("workload = ?".to_string());
            args.push(workload_key(w));
        }
        if !filters.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&filters.join(" AND "));
        }
        query.push_str(" ORDER BY status ASC, created_at DESC");

        let mut sql = sqlx::query(&query);
        for arg in args {
            sql = sql.bind(arg);
        }

        let rows = sql
            .fetch_all(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| SuggestionEntry {
                id: row.get("id"),
                provider: Provider::from_alias(row.get::<String, _>("provider").as_str())
                    .unwrap_or(Provider::HuggingFace),
                workload: workload_from_key(row.get::<String, _>("workload").as_str()),
                model: row.get("model"),
                status: row.get("status"),
                rationale: row.get("rationale"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    /// Inserts or updates a suggestion for a given provider, workload, and model.
    pub async fn upsert_suggestion(
        &self,
        provider: Provider,
        workload: Workload,
        model: String,
        rationale: Option<String>,
        metadata: Option<String>,
        status: &str,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"INSERT INTO provider_model_suggestions
               (provider, workload, model, status, rationale, metadata, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(provider, workload, model) DO UPDATE SET
                   status = excluded.status,
                   rationale = excluded.rationale,
                   metadata = excluded.metadata,
                   updated_at = excluded.updated_at"#,
        )
        .bind(provider.as_str())
        .bind(workload_key(workload))
        .bind(model)
        .bind(status)
        .bind(rationale)
        .bind(metadata)
        .bind(&now)
        .bind(&now)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();
        Ok(())
    }

    /// Adopts a model (suggestion or new) into the active roster for a provider/workload.
    pub async fn adopt_model(
        &self,
        provider: Provider,
        workload: Workload,
        model: String,
        rationale: Option<String>,
        metadata: Option<String>,
        priority: i64,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"INSERT INTO provider_models
               (provider, workload, model, status, priority, rationale, metadata, created_at, updated_at)
               VALUES (?, ?, ?, 'active', ?, ?, ?, ?, ?)
               ON CONFLICT(provider, workload, model) DO UPDATE SET
                   status = 'active',
                   priority = excluded.priority,
                   rationale = excluded.rationale,
                   metadata = excluded.metadata,
                   updated_at = excluded.updated_at"#,
        )
        .bind(provider.as_str())
        .bind(workload_key(workload))
        .bind(&model)
        .bind(priority)
        .bind(rationale.clone())
        .bind(metadata.clone())
        .bind(&now)
        .bind(&now)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();

        let result = sqlx::query(
            r#"UPDATE provider_model_suggestions
               SET status = 'adopted', updated_at = ?
             WHERE provider = ? AND workload = ? AND model = ?"#,
        )
        .bind(&now)
        .bind(provider.as_str())
        .bind(workload_key(workload))
        .bind(model)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let _ = result.rows_affected();
        Ok(())
    }

    /// Retires a model from active use for a given provider/workload.
    pub async fn retire_model(
        &self,
        provider: Provider,
        workload: Workload,
        model: &str,
    ) -> Result<bool, AppError> {
        let now = Utc::now().to_rfc3339();
        let result = sqlx::query(
            r#"UPDATE provider_models
               SET status = 'retired', updated_at = ?
             WHERE provider = ? AND workload = ? AND model = ?"#,
        )
        .bind(&now)
        .bind(provider.as_str())
        .bind(workload_key(workload))
        .bind(model)
        .execute(&*self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Returns all active models for a provider, optionally filtered by workload.
    pub async fn active_models(
        &self,
        provider: Provider,
        workload: Option<Workload>,
    ) -> Result<Vec<ModelEntry>, AppError> {
        let mut query = String::from(
            "SELECT provider, workload, model, status, priority, rationale, metadata, created_at, updated_at \
             FROM provider_models WHERE status = 'active' AND provider = ?",
        );
        if workload.is_some() {
            query.push_str(" AND workload = ?");
        }
        query.push_str(" ORDER BY priority ASC, updated_at DESC");

        let mut sql = sqlx::query(&query).bind(provider.as_str());
        if let Some(w) = workload {
            sql = sql.bind(workload_key(w));
        }

        let rows = sql
            .fetch_all(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        Ok(rows
            .into_iter()
            .map(|row| ModelEntry {
                provider,
                workload: workload_from_key(row.get::<String, _>("workload").as_str()),
                model: row.get("model"),
                status: row.get("status"),
                priority: row.get("priority"),
                rationale: row.get("rationale"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    /// Seeds default models if none exist for a provider/workload combination.
    pub async fn seed_defaults(&self) -> Result<(), AppError> {
        // Default models per provider/workload
        let defaults = vec![
            // Groq defaults (ultra-fast inference)
            (
                Provider::Groq,
                Workload::Chat,
                "llama-3.3-70b-versatile",
                10,
                "Fast, versatile Llama model",
            ),
            (
                Provider::Groq,
                Workload::Code,
                "llama-3.3-70b-versatile",
                10,
                "Versatile model suitable for code",
            ),
            (
                Provider::Groq,
                Workload::Summarization,
                "llama-3.3-70b-versatile",
                20,
                "Fast summarization",
            ),
            (
                Provider::Groq,
                Workload::Creative,
                "llama-3.3-70b-versatile",
                15,
                "Creative and versatile",
            ),
            // DeepSeek defaults (pay-as-you-go, very low cost)
            (
                Provider::DeepSeek,
                Workload::Chat,
                "deepseek-chat",
                20,
                "Powerful reasoning and chat",
            ),
            (
                Provider::DeepSeek,
                Workload::Code,
                "deepseek-chat",
                15,
                "Strong coding capabilities",
            ),
            (
                Provider::DeepSeek,
                Workload::Summarization,
                "deepseek-chat",
                25,
                "Effective summarization",
            ),
            (
                Provider::DeepSeek,
                Workload::Extraction,
                "deepseek-chat",
                20,
                "Information extraction",
            ),
            (
                Provider::DeepSeek,
                Workload::Creative,
                "deepseek-chat",
                25,
                "Creative writing",
            ),
            (
                Provider::DeepSeek,
                Workload::Classification,
                "deepseek-chat",
                25,
                "Text classification",
            ),
            // Together AI defaults
            (
                Provider::Together,
                Workload::Chat,
                "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free",
                30,
                "Free Llama model",
            ),
            (
                Provider::Together,
                Workload::Code,
                "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free",
                25,
                "Code-capable free model",
            ),
            // Google Gemini defaults
            (
                Provider::Google,
                Workload::Chat,
                "gemini-2.0-flash",
                40,
                "Fast multimodal Gemini",
            ),
            (
                Provider::Google,
                Workload::Code,
                "gemini-2.0-flash",
                35,
                "Gemini with code capabilities",
            ),
            (
                Provider::Google,
                Workload::Summarization,
                "gemini-2.0-flash",
                40,
                "Fast summarization",
            ),
            // Cloudflare Workers AI defaults (serverless GPU inference)
            (
                Provider::Cloudflare,
                Workload::Chat,
                "@cf/meta/llama-3.3-70b-instruct",
                18,
                "Serverless Llama 3.3 70B",
            ),
            (
                Provider::Cloudflare,
                Workload::Code,
                "@cf/meta/llama-3.3-70b-instruct",
                18,
                "Serverless code-capable model",
            ),
            (
                Provider::Cloudflare,
                Workload::Creative,
                "@cf/openai/gpt-oss-120b",
                20,
                "OpenAI open-source 120B model",
            ),
            // Cerebras AI defaults (ultra-fast, 1M tokens/day free)
            (
                Provider::Cerebras,
                Workload::Chat,
                "llama-3.1-70b",
                12,
                "Ultra-fast Llama 3.1 70B",
            ),
            (
                Provider::Cerebras,
                Workload::Code,
                "llama-3.1-70b",
                12,
                "Fast code-capable model",
            ),
            (
                Provider::Cerebras,
                Workload::Summarization,
                "llama-3.1-8b",
                15,
                "Fast summarization with 8B model",
            ),
            // Mistral AI defaults (free tier with rate limits)
            (
                Provider::Mistral,
                Workload::Chat,
                "mistral-small-latest",
                22,
                "Mistral Small for chat",
            ),
            (
                Provider::Mistral,
                Workload::Code,
                "mistral-small-latest",
                22,
                "Mistral Small for code",
            ),
            (
                Provider::Mistral,
                Workload::Summarization,
                "mistral-small-latest",
                25,
                "Mistral Small for summarization",
            ),
            // Clarifai AI defaults (1K requests/month free)
            (
                Provider::Clarifai,
                Workload::Chat,
                "gpt-4",
                45,
                "GPT-4 via Clarifai",
            ),
            (
                Provider::Clarifai,
                Workload::Code,
                "gpt-4",
                45,
                "GPT-4 code via Clarifai",
            ),
            // GitHub Models defaults (50-150 RPD depending on plan)
            (
                Provider::GitHubModels,
                Workload::Chat,
                "gpt-4o",
                35,
                "GPT-4o via GitHub",
            ),
            (
                Provider::GitHubModels,
                Workload::Code,
                "gpt-4o",
                35,
                "GPT-4o code via GitHub",
            ),
            // OpenRouter defaults (50 req/day for :free models)
            (
                Provider::OpenRouter,
                Workload::Chat,
                "deepseek/deepseek-r1:free",
                50,
                "DeepSeek R1 free via OpenRouter",
            ),
            (
                Provider::OpenRouter,
                Workload::Code,
                "deepseek/deepseek-r1:free",
                50,
                "DeepSeek R1 code via OpenRouter",
            ),
        ];

        for (provider, workload, model, priority, rationale) in defaults {
            // Check if any active models exist for this provider/workload
            let existing = self.active_models(provider, Some(workload)).await?;
            if existing.is_empty() {
                let now = Utc::now().to_rfc3339();
                let result = sqlx::query(
                    r#"INSERT OR IGNORE INTO provider_models
                       (provider, workload, model, status, priority, rationale, metadata, created_at, updated_at)
                       VALUES (?, ?, ?, 'active', ?, ?, NULL, ?, ?)"#,
                )
                .bind(provider.as_str())
                .bind(workload_key(workload))
                .bind(model)
                .bind(priority)
                .bind(rationale)
                .bind(&now)
                .bind(&now)
                .execute(&*self.pool)
                .await
                .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;
                let _ = result.rows_affected();
            }
        }

        Ok(())
    }

    /// Gets usage statistics for a provider/workload combination.
    pub async fn usage_stats(
        &self,
        provider: Provider,
        workload: Option<Workload>,
    ) -> Result<UsageStats, AppError> {
        let mut query = String::from(
            r#"SELECT
                COUNT(*) as total_calls,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful_calls,
                AVG(latency_ms) as avg_latency,
                MAX(latency_ms) as max_latency
            FROM provider_usage
            WHERE provider = ?"#,
        );

        if workload.is_some() {
            query.push_str(" AND model IN (SELECT model FROM provider_models WHERE provider = ? AND workload = ?)");
        }

        let mut sql = sqlx::query(&query).bind(provider.as_str());
        if let Some(w) = workload {
            sql = sql.bind(provider.as_str()).bind(workload_key(w));
        }

        let row = sql
            .fetch_one(&*self.pool)
            .await
            .map_err(|err| AppError::DatabaseError(DbError::QueryFailed(err)))?;

        let total_calls: i64 = row.get("total_calls");
        let successful_calls: i64 = row.get("successful_calls");
        let avg_latency: Option<f64> = row.try_get("avg_latency").ok();
        let max_latency: Option<i64> = row.try_get("max_latency").ok();

        let success_rate = if total_calls > 0 {
            (successful_calls as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        };

        Ok(UsageStats {
            total_calls,
            successful_calls,
            success_rate,
            avg_latency_ms: avg_latency.unwrap_or(0.0),
            max_latency_ms: max_latency.unwrap_or(0),
        })
    }
}

/// Usage statistics for a provider/workload.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total number of calls.
    pub total_calls: i64,
    /// Number of successful calls.
    pub successful_calls: i64,
    /// Success rate as a percentage.
    pub success_rate: f64,
    /// Average latency in milliseconds.
    pub avg_latency_ms: f64,
    /// Maximum latency in milliseconds.
    pub max_latency_ms: i64,
}

/// A catalog entry representing an active or retired model for a provider/workload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// The provider (huggingface, google, etc.).
    pub provider: Provider,
    /// The workload category.
    pub workload: Workload,
    /// The model identifier.
    pub model: String,
    /// Status (active or retired).
    pub status: String,
    /// Priority (lower = higher priority).
    pub priority: i64,
    /// Human-readable reason for selection.
    pub rationale: Option<String>,
    /// JSON metadata (optional).
    pub metadata: Option<String>,
    /// RFC3339 timestamp when created.
    pub created_at: String,
    /// RFC3339 timestamp when last updated.
    pub updated_at: String,
}

/// A suggestion entry representing a candidate model for adoption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionEntry {
    /// Database ID.
    pub id: i64,
    /// The provider (huggingface, google, etc.).
    pub provider: Provider,
    /// The workload category.
    pub workload: Workload,
    /// The model identifier.
    pub model: String,
    /// Status (pending, trial, adopted).
    pub status: String,
    /// Why this model was suggested.
    pub rationale: Option<String>,
    /// JSON metadata with cost estimates, etc.
    pub metadata: Option<String>,
    /// RFC3339 timestamp when created.
    pub created_at: String,
    /// RFC3339 timestamp when last updated.
    pub updated_at: String,
}

fn workload_key(workload: Workload) -> String {
    match workload {
        Workload::Chat => "chat",
        Workload::Summarization => "summarization",
        Workload::Code => "code",
        Workload::Extraction => "extraction",
        Workload::Creative => "creative",
        Workload::Classification => "classification",
    }
    .to_string()
}

fn workload_from_key(key: &str) -> Workload {
    match key {
        "chat" => Workload::Chat,
        "summarization" => Workload::Summarization,
        "code" => Workload::Code,
        "extraction" => Workload::Extraction,
        "creative" => Workload::Creative,
        "classification" => Workload::Classification,
        _ => Workload::Chat,
    }
}
