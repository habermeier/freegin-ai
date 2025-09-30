//! GitHub Models provider connector implementing the `AIProvider` trait.
//!
//! GitHub Models provides access to various AI models through GitHub's platform.
//! Free tier: 50-150 requests/day (depending on GitHub Copilot plan)
//! Get API key: https://github.com/settings/tokens (create PAT with models:read scope)
//! Models: GPT-4, Llama, Phi, and more
//!
//! API format: OpenAI-compatible endpoints
//! Base URL: https://models.inference.ai.azure.com

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the GitHub Models API.
#[derive(Debug, Clone)]
pub struct GitHubModelsClient {
    github_token: String,
    base_url: String,
    http_client: Client,
}

impl GitHubModelsClient {
    /// Creates a new `GitHubModelsClient`.
    ///
    /// # Arguments
    /// * `github_token` - GitHub Personal Access Token with models:read scope
    /// * `base_url` - Base URL, e.g., https://models.inference.ai.azure.com
    pub fn new(github_token: String, base_url: String) -> Result<Self, AppError> {
        if github_token.trim().is_empty() {
            return Err(AppError::ConfigError(
                "GitHub token cannot be empty".into(),
            ));
        }
        Ok(Self {
            github_token,
            base_url,
            http_client: Client::new(),
        })
    }
}

#[async_trait]
impl AIProvider for GitHubModelsClient {
    /// Sends a generation request to the GitHub Models API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/chat/completions", self.base_url);

        let body = GitHubModelsRequestBody {
            model: if request.model.is_empty() {
                "gpt-4o".to_string()
            } else {
                request.model.clone()
            },
            messages: vec![GitHubModelsMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
        };

        let response = self
            .http_client
            .post(&api_url)
            .bearer_auth(&self.github_token)
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
                "GitHub Models request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_body = response
            .json::<GitHubModelsResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response_body
            .choices
            .get(0)
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::GitHubModels,
        })
    }
}

#[derive(Serialize)]
struct GitHubModelsRequestBody {
    model: String,
    messages: Vec<GitHubModelsMessage>,
}

#[derive(Serialize)]
struct GitHubModelsMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct GitHubModelsResponseBody {
    choices: Vec<GitHubModelsChoice>,
}

#[derive(Deserialize)]
struct GitHubModelsChoice {
    message: GitHubModelsMessageContent,
}

#[derive(Deserialize)]
struct GitHubModelsMessageContent {
    content: Option<String>,
}
