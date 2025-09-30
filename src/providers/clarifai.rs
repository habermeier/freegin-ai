//! Clarifai AI provider connector implementing the `AIProvider` trait.
//!
//! Clarifai provides AI models for vision, language, and audio.
//! Free tier: 1,000 requests/month
//! Get API key: https://clarifai.com/settings/security (Personal Access Token)
//! Models: Various GPT and Llama models
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://api.clarifai.com/v2/ext/openai/v1

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Clarifai AI API.
#[derive(Debug, Clone)]
pub struct ClarifaiClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl ClarifaiClient {
    /// Creates a new `ClarifaiClient`.
    ///
    /// # Arguments
    /// * `api_key` - Clarifai Personal Access Token (PAT)
    /// * `base_url` - Base URL, e.g., https://api.clarifai.com/v2/ext/openai/v1
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "Clarifai API key cannot be empty".into(),
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
impl AIProvider for ClarifaiClient {
    /// Sends a generation request to the Clarifai AI API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = ClarifaiRequestBody {
            model: if request.model.is_empty() {
                "gpt-4".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![ClarifaiMessage {
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
                "Clarifai AI request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<ClarifaiResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Clarifai,
        })
    }
}

#[derive(Serialize)]
struct ClarifaiRequestBody {
    model: String,
    messages: Vec<ClarifaiMessage>,
}

#[derive(Serialize)]
struct ClarifaiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClarifaiResponseBody {
    choices: Vec<ClarifaiChoice>,
}

#[derive(Deserialize)]
struct ClarifaiChoice {
    message: ClarifaiMessageContent,
}

#[derive(Deserialize)]
struct ClarifaiMessageContent {
    content: Option<String>,
}
