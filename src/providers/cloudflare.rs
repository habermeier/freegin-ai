//! Cloudflare Workers AI provider connector implementing the `AIProvider` trait.
//!
//! Cloudflare Workers AI provides serverless GPU-powered inference on Cloudflare's global network.
//! Free tier: 10,000 Neurons/day (~100-10,000 requests depending on model), 100,000 requests/day platform limit
//! Get API key: https://dash.cloudflare.com/ (create API token with Workers AI permissions)
//! Models: Llama 3.3, OpenAI open models, Mistral, DeepSeek, and more
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://api.cloudflare.com/client/v4/accounts/{ACCOUNT_ID}/ai/v1

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Cloudflare Workers AI API.
#[derive(Debug, Clone)]
pub struct CloudflareClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl CloudflareClient {
    /// Creates a new `CloudflareClient`.
    ///
    /// # Arguments
    /// * `api_key` - Cloudflare API token with Workers AI permissions
    /// * `base_url` - Base URL including account ID, e.g., https://api.cloudflare.com/client/v4/accounts/{ACCOUNT_ID}/ai/v1
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        if api_key.trim().is_empty() {
            return Err(AppError::ConfigError(
                "Cloudflare API key cannot be empty".into(),
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
impl AIProvider for CloudflareClient {
    /// Sends a generation request to the Cloudflare Workers AI API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = CloudflareRequestBody {
            model: if request.model.is_empty() {
                "@cf/meta/llama-3.3-70b-instruct".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![CloudflareMessage {
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
                "Cloudflare Workers AI request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<CloudflareResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Cloudflare,
        })
    }
}

#[derive(Serialize)]
struct CloudflareRequestBody {
    model: String,
    messages: Vec<CloudflareMessage>,
}

#[derive(Serialize)]
struct CloudflareMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct CloudflareResponseBody {
    choices: Vec<CloudflareChoice>,
}

#[derive(Deserialize)]
struct CloudflareChoice {
    message: CloudflareMessageContent,
}

#[derive(Deserialize)]
struct CloudflareMessageContent {
    content: Option<String>,
}
