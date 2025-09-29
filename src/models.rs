//! Core data structures shared across web handlers and provider connectors.

use serde::{Deserialize, Serialize};

use crate::providers::Provider;

/// Represents an incoming request to the `/generate` endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct AIRequest {
    /// The model to use for the generation (e.g., "gpt-4o", "gemini-1.5-pro").
    pub model: String,
    /// The user's prompt.
    pub prompt: String,
    /// Optional tags for classifying the request (e.g., "code-generation", "python").
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Represents the response sent back to the client.
#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    /// The generated content from the AI model.
    pub content: String,
    /// The provider that ultimately handled the request.
    pub provider: Provider,
}
