//! Cerebras AI provider connector implementing the `AIProvider` trait.
//!
//! Cerebras provides ultra-fast AI inference powered by the Wafer-Scale Engine.
//! Free tier: 1 million tokens/day
//! Get API key: https://cloud.cerebras.ai/
//! Models: Llama 3.1 8B, Llama 3.1 70B
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://api.cerebras.ai/v1

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Cerebras AI API.
#[derive(Debug, Clone)]
pub struct CerebrasClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl CerebrasClient {
    /// Creates a new `CerebrasClient`.
    ///
    /// # Arguments
    /// * `api_key` - Cerebras API key
    /// * `base_url` - Base URL, e.g., https://api.cerebras.ai/v1
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "Cerebras API key cannot be empty".into(),
            ));
        }
        Ok(Self {
            api_key,
            base_url,
            http_client: Client::new(),
        })
    }
}

#[async_trait]
impl AIProvider for CerebrasClient {
    /// Sends a generation request to the Cerebras AI API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = CerebrasRequestBody {
            model: if request.model.is_empty() {
                "llama-3.1-70b".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![CerebrasMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
        };

        let response = self
            .http_client
            .post(&api_url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::ApiError(format!(
                "Cerebras AI request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<CerebrasResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Cerebras,
        })
    }
}

#[derive(Serialize)]
struct CerebrasRequestBody {
    model: String,
    messages: Vec<CerebrasMessage>,
}

#[derive(Serialize)]
struct CerebrasMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CerebrasResponseBody {
    choices: Vec<CerebrasChoice>,
}

#[derive(Deserialize)]
struct CerebrasChoice {
    message: CerebrasMessageContent,
}

#[derive(Deserialize)]
struct CerebrasMessageContent {
    content: Option<String>,
}
