//! Mistral AI provider connector implementing the `AIProvider` trait.
//!
//! Mistral AI provides open and portable generative AI for developers and businesses.
//! Free tier: Available with rate limits
//! Get API key: https://console.mistral.ai/
//! Models: Mistral Large, Mistral Medium, Mistral Small, and more
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://api.mistral.ai/v1

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Mistral AI API.
#[derive(Debug, Clone)]
pub struct MistralClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl MistralClient {
    /// Creates a new `MistralClient`.
    ///
    /// # Arguments
    /// * `api_key` - Mistral API key
    /// * `base_url` - Base URL, e.g., https://api.mistral.ai/v1
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "Mistral API key cannot be empty".into(),
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
impl AIProvider for MistralClient {
    /// Sends a generation request to the Mistral AI API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = MistralRequestBody {
            model: if request.model.is_empty() {
                "mistral-small-latest".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![MistralMessage {
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
                "Mistral AI request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<MistralResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Mistral,
        })
    }
}

#[derive(Serialize)]
struct MistralRequestBody {
    model: String,
    messages: Vec<MistralMessage>,
}

#[derive(Serialize)]
struct MistralMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MistralResponseBody {
    choices: Vec<MistralChoice>,
}

#[derive(Deserialize)]
struct MistralChoice {
    message: MistralMessageContent,
}

#[derive(Deserialize)]
struct MistralMessageContent {
    content: Option<String>,
}
