//! DeepSeek provider connector implementing the `AIProvider` trait.
//!
//! DeepSeek provides pay-as-you-go inference with state-of-the-art models.
//! Pricing: Very low cost ($0.028-$2.19 per million tokens)
//! Get API key: https://platform.deepseek.com/api_keys

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the DeepSeek API.
#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl DeepSeekClient {
    /// Creates a new `DeepSeekClient`.
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "DeepSeek API key cannot be empty".into(),
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
impl AIProvider for DeepSeekClient {
    /// Sends a generation request to the DeepSeek API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = DeepSeekRequestBody {
            model: if request.model.is_empty() {
                "deepseek-chat".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![DeepSeekMessage {
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
                "DeepSeek request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<DeepSeekResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::DeepSeek,
        })
    }
}

#[derive(Serialize)]
struct DeepSeekRequestBody {
    model: String,
    messages: Vec<DeepSeekMessage>,
}

#[derive(Serialize)]
struct DeepSeekMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct DeepSeekResponseBody {
    choices: Vec<DeepSeekChoice>,
}

#[derive(Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessageContent,
}

#[derive(Deserialize)]
struct DeepSeekMessageContent {
    content: Option<String>,
}