//! Google Gemini provider connector implementing the `AIProvider` trait.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// A client for interacting with the Google Gemini API.
#[derive(Debug, Clone)]
pub struct GoogleClient {
    api_key: String,
    base_url: String,
    http_client: Client,
}

impl GoogleClient {
    /// Creates a new `GoogleClient`.
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        let http_client = Client::builder()
            .user_agent(format!("freegin-ai/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|err| AppError::ConfigError(format!("Failed to build HTTP client: {err}")))?;

        Ok(Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_owned(),
            http_client,
        })
    }
}

#[async_trait]
impl AIProvider for GoogleClient {
    /// Sends a generation request to the Google Gemini API.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let body = GoogleRequestBody {
            contents: vec![GoogleContent {
                parts: vec![GooglePart {
                    text: request.prompt.clone(),
                }],
            }],
        };

        let http_response = self
            .http_client
            .post(&api_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !http_response.status().is_success() {
            let status = http_response.status();
            let error_text = http_response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read error body>".into());
            return Err(AppError::ApiError(format!(
                "Google Gemini request failed with status {status}: {error_text}"
            )));
        }

        let response = http_response
            .json::<GoogleResponseBody>()
            .await
            .map_err(|e| AppError::ApiError(e.to_string()))?;

        let content = response
            .candidates
            .get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(AIResponse {
            content,
            provider: Provider::Google,
        })
    }
}

#[derive(Serialize)]
struct GoogleRequestBody {
    contents: Vec<GoogleContent>,
}

#[derive(Serialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Deserialize, Debug)]
struct GoogleResponseBody {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize, Debug)]
struct GoogleCandidate {
    content: GoogleContentResponse,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GoogleContentResponse {
    parts: Vec<GooglePartResponse>,
}

#[derive(Deserialize, Debug)]
struct GooglePartResponse {
    text: String,
}
