//! AI Provider abstraction module.

use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
};

/// An enum representing the available AI providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    /// OpenAI models (GPT-4, etc.).
    OpenAI,
    /// Google models (Gemini).
    Google,
    /// Hugging Face hosted models.
    HuggingFace,
    /// Anthropic models (Claude).
    Anthropic,
    /// Cohere models.
    Cohere,
    /// Groq models (ultra-fast inference).
    Groq,
    /// DeepSeek models (pay-per-use, very low cost).
    DeepSeek,
    /// Together AI models.
    Together,
    /// Cloudflare Workers AI models (serverless GPU inference).
    Cloudflare,
    /// Cerebras AI models (ultra-fast inference, 1M tokens/day free).
    Cerebras,
    /// Mistral AI models (free tier with rate limits).
    Mistral,
    /// Clarifai AI models (1K requests/month free).
    Clarifai,
    /// GitHub Models (requires GitHub PAT with models:read scope).
    GitHubModels,
    /// OpenRouter (aggregator with limited free tier).
    OpenRouter,
}

/// A common trait for all AI provider clients.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Sends a request to the provider's API to generate content.
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError>;
}

impl Provider {
    /// Returns the canonical string identifier for the provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Google => "google",
            Provider::HuggingFace => "huggingface",
            Provider::Anthropic => "anthropic",
            Provider::Cohere => "cohere",
            Provider::Groq => "groq",
            Provider::DeepSeek => "deepseek",
            Provider::Together => "together",
            Provider::Cloudflare => "cloudflare",
            Provider::Cerebras => "cerebras",
            Provider::Mistral => "mistral",
            Provider::Clarifai => "clarifai",
            Provider::GitHubModels => "github",
            Provider::OpenRouter => "openrouter",
        }
    }

    /// Attempts to resolve a provider from a string alias (case-insensitive).
    pub fn from_alias(value: &str) -> Option<Self> {
        match value.to_lowercase().as_str() {
            "openai" | "gpt" => Some(Provider::OpenAI),
            "google" | "gemini" => Some(Provider::Google),
            "huggingface" | "hugging_face" | "hf" => Some(Provider::HuggingFace),
            "anthropic" | "claude" => Some(Provider::Anthropic),
            "cohere" => Some(Provider::Cohere),
            "groq" => Some(Provider::Groq),
            "deepseek" => Some(Provider::DeepSeek),
            "together" | "togetherai" | "together_ai" => Some(Provider::Together),
            "cloudflare" | "cf" | "workers" | "workers_ai" => Some(Provider::Cloudflare),
            "cerebras" => Some(Provider::Cerebras),
            "mistral" | "mistralai" | "mistral_ai" => Some(Provider::Mistral),
            "clarifai" => Some(Provider::Clarifai),
            "github" | "github_models" | "githubmodels" => Some(Provider::GitHubModels),
            "openrouter" | "open_router" => Some(Provider::OpenRouter),
            _ => None,
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub mod cerebras;
pub mod clarifai;
pub mod cloudflare;
pub mod deepseek;
pub mod github_models;
pub mod google;
pub mod groq;
pub mod hugging_face;
pub mod mistral;
pub mod openai;
pub mod openrouter;
pub mod router;
pub mod together;

pub use router::ProviderRouter;
