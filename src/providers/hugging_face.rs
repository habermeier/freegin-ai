//! Hugging Face provider connector implementing the `AIProvider` trait.

use async_trait::async_trait;
use reqwest::{header, Client};
use serde::Serialize;

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::{AIProvider, Provider},
};

/// Client for interacting with Hugging Face's Inference API.
#[derive(Debug, Clone)]
pub struct HuggingFaceClient {
    base_url: String,
    http_client: Client,
}

impl HuggingFaceClient {
    /// Creates a new `HuggingFaceClient`.
    pub fn new(api_key: String, base_url: String) -> Result<Self, AppError> {
        let mut headers = header::HeaderMap::new();
        drop(headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {api_key}")).map_err(|err| {
                AppError::ConfigError(format!("Invalid Hugging Face token: {err}"))
            })?,
        ));
        drop(headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        ));

        let http_client = Client::builder()
            .default_headers(headers)
            .user_agent(format!("freegin-ai/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|err| AppError::ConfigError(format!("Failed to build HTTP client: {err}")))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            http_client,
        })
    }
}

#[async_trait]
impl AIProvider for HuggingFaceClient {
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        let api_url = format!("{}/models/{}", self.base_url, request.model);
        let payload = HuggingFaceRequest {
            inputs: request.prompt.clone(),
            parameters: Some(HuggingFaceParameters {
                return_full_text: Some(false),
            }),
        };

        let http_response = self
            .http_client
            .post(&api_url)
            .json(&payload)
            .send()
            .await
            .map_err(|err| AppError::NetworkError(err.to_string()))?;

        if !http_response.status().is_success() {
            let status = http_response.status();
            let body = http_response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read error body>".into());
            return Err(AppError::ApiError(format!(
                "Hugging Face request failed with status {status}: {body}"
            )));
        }

        let value = http_response
            .json::<serde_json::Value>()
            .await
            .map_err(|err| AppError::ApiError(err.to_string()))?;

        let content = extract_generated_text(value).unwrap_or_else(|| "".to_string());

        Ok(AIResponse {
            content,
            provider: Provider::HuggingFace,
        })
    }
}

#[derive(Serialize)]
struct HuggingFaceRequest {
    inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HuggingFaceParameters>,
}

#[derive(Serialize, Default)]
struct HuggingFaceParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    return_full_text: Option<bool>,
}

fn extract_generated_text(value: serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                if let Some(text) = item.get("generated_text").and_then(|v| v.as_str()) {
                    return Some(text.to_string());
                }
                if let Some(children) = item.get("generated_texts") {
                    if let Some(first) = children.as_array().and_then(|arr| arr.get(0)) {
                        if let Some(text) = first.get("text").and_then(|v| v.as_str()) {
                            return Some(text.to_string());
                        }
                    }
                }
            }
            None
        }
        serde_json::Value::Object(map) => map
            .get("generated_text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_array_response() {
        let data = json!([
            {"generated_text": "Hello world"}
        ]);
        assert_eq!(extract_generated_text(data), Some("Hello world".into()));
    }

    #[test]
    fn parses_object_response() {
        let data = json!({"generated_text": "Hi"});
        assert_eq!(extract_generated_text(data), Some("Hi".into()));
    }

    #[test]
    fn handles_missing() {
        let data = json!({"foo": "bar"});
        assert_eq!(extract_generated_text(data), None);
    }
}
