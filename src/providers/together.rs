//! Together AI provider connector implementing the `AIProvider` trait.
//!
//! Together AI provides access to diverse open-source models.
//! Free tier: Available with dedicated free models
//! Get API key: https://api.together.xyz/settings/api-keys

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Together AI API.
#[derive(Debug, Clone)]
pub struct TogetherClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl TogetherClient {
    /// Creates a new `TogetherClient`.
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "Together AI API key cannot be empty".into(),
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
impl AIProvider for TogetherClient {
    /// Sends a generation request to the Together AI API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = TogetherRequestBody {
            model: if request.model.is_empty() {
                "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![TogetherMessage {
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
                "Together AI request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<TogetherResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Together,
        })
    }
}

#[derive(Serialize)]
struct TogetherRequestBody {
    model: String,
    messages: Vec<TogetherMessage>,
}

#[derive(Serialize)]
struct TogetherMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct TogetherResponseBody {
    choices: Vec<TogetherChoice>,
}

#[derive(Deserialize)]
struct TogetherChoice {
    message: TogetherMessageContent,
}

#[derive(Deserialize)]
struct TogetherMessageContent {
    content: Option<String>,
}