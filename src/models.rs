//! Core data structures shared across web handlers and provider connectors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::providers::Provider;

/// Represents an incoming request to the `/generate` endpoint or CLI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AIRequest {
    /// The model to use for the generation (e.g., "gpt-4o", "gemini-1.5-pro").
    pub model: String,
    /// The user's prompt.
    pub prompt: String,
    /// Optional tags for classifying the request (e.g., "provider:hf").
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional context snippets that accompany the prompt.
    #[serde(default)]
    pub context: Vec<String>,
    /// Arbitrary metadata provided by the caller.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Routing hints that guide provider selection.
    #[serde(default)]
    pub hints: RequestHints,
}

/// Represents the response sent back to the client.
#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    /// The generated content from the AI model.
    pub content: String,
    /// The provider that ultimately handled the request.
    pub provider: Provider,
}

/// Hint parameters that influence provider/model selection.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RequestHints {
    /// Desired complexity of the task.
    #[serde(default)]
    pub complexity: Option<RequestComplexity>,
    /// Desired quality/cost trade-off.
    #[serde(default)]
    pub quality: Option<RequestQuality>,
    /// Desired response speed.
    #[serde(default)]
    pub speed: Option<RequestSpeed>,
    /// Guardrail strictness preferences.
    #[serde(default)]
    pub guardrail: Option<RequestGuardrail>,
    /// Preferred response format.
    #[serde(default)]
    pub response_format: Option<ResponseFormat>,
    /// Optional explicit provider override.
    #[serde(default)]
    pub provider: Option<String>,
    /// Desired workload category.
    #[serde(default)]
    pub workload: Option<Workload>,
}

/// Complexity levels for the requested task.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequestComplexity {
    /// Trivial or short requests.
    Low,
    /// Moderate requests requiring standard reasoning.
    Medium,
    /// Difficult, long, or resource-intensive requests.
    High,
}

/// Quality/cost tiers for provider selection.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequestQuality {
    /// Optimise for lower cost over quality.
    Standard,
    /// Balanced cost versus quality.
    Balanced,
    /// Highest quality regardless of cost.
    Premium,
}

/// Desired response speed.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequestSpeed {
    /// Prioritise latency over completeness.
    Fast,
    /// Normal latency with balanced quality.
    Normal,
}

/// Guardrail strictness configuration.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RequestGuardrail {
    /// Apply strict guardrails and moderation.
    Strict,
    /// Relax guardrails for more permissive responses.
    Lenient,
}

/// Preferred format for the model response.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    /// Plain text response.
    Text,
    /// Markdown formatted response.
    Markdown,
    /// JSON structured response.
    Json,
}

/// Workload categories used to organise models in the catalog.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Workload {
    /// General conversation and Q&A.
    Chat,
    /// Text summarization tasks.
    Summarization,
    /// Code generation and analysis.
    Code,
    /// Information extraction from text.
    Extraction,
    /// Creative writing and ideation.
    Creative,
    /// Text classification and labeling.
    Classification,
}

impl Workload {
    /// Returns all valid workload variant names.
    pub fn variants() -> &'static [&'static str] {
        &[
            "chat",
            "summarization",
            "code",
            "extraction",
            "creative",
            "classification",
        ]
    }
}
