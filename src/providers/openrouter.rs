//! OpenRouter provider connector implementing the `AIProvider` trait.
//!
//! OpenRouter provides unified access to 300+ AI models from various providers.
//! Free tier: 50 requests/day for :free models (1000/day with $10+ credits)
//! Get API key: https://openrouter.ai/keys
//! Models: Various :free variants available
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://openrouter.ai/api/v1

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the OpenRouter API.
#[derive(Debug, Clone)]
pub struct OpenRouterClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl OpenRouterClient {
    /// Creates a new `OpenRouterClient`.
    ///
    /// # Arguments
    /// * `api_key` - OpenRouter API key
    /// * `base_url` - Base URL, e.g., https://openrouter.ai/api/v1
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "OpenRouter API key cannot be empty".into(),
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
impl AIProvider for OpenRouterClient {
    /// Sends a generation request to the OpenRouter API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = OpenRouterRequestBody {
            model: if request.model.is_empty() {
                "deepseek/deepseek-r1:free".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![OpenRouterMessage {
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
                "OpenRouter request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<OpenRouterResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::OpenRouter,
        })
    }
}

#[derive(Serialize)]
struct OpenRouterRequestBody {
    model: String,
    messages: Vec<OpenRouterMessage>,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponseBody {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessageContent,
}

#[derive(Deserialize)]
struct OpenRouterMessageContent {
    content: Option<String>,
}
