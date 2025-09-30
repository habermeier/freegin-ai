//! Groq provider connector implementing the `AIProvider` trait.
//!
//! Groq provides ultra-fast inference with OpenAI-compatible API.
//! Free tier: 14,400 requests/day, 6,000 tokens/minute
//! Get API key: https://console.groq.com/keys

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Groq API.
#[derive(Debug, Clone)]
pub struct GroqClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl GroqClient {
    /// Creates a new `GroqClient`.
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError("Groq API key cannot be empty".into()));
        }
        Ok(Self {
            api_key,
            base_url,
            http_client: Client::new(),
        })
    }
}

#[async_trait]
impl AIProvider for GroqClient {
    /// Sends a generation request to the Groq API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = GroqRequestBody {
            model: if request.model.is_empty() {
                "llama-3.3-70b-versatile".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![GroqMessage {
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
                "Groq request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<GroqResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Groq,
        })
    }
}

#[derive(Serialize)]
struct GroqRequestBody {
    model: String,
    messages: Vec<GroqMessage>,
}

#[derive(Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GroqResponseBody {
    choices: Vec<GroqChoice>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqMessageContent,
}

#[derive(Deserialize)]
struct GroqMessageContent {
    content: Option<String>,
}