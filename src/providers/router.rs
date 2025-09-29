//! Provider routing utilities and fallback logic.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
    time::Instant,
};

use tracing::{debug, warn};

use crate::{
    config::AppConfig,
    credentials::CredentialStore,
    error::AppError,
    models::{AIRequest, AIResponse},
    usage::UsageLogger,
};

use super::{google::GoogleClient, hugging_face::HuggingFaceClient, AIProvider, Provider};

/// Coordinates AI providers and encapsulates routing logic.
pub struct ProviderRouter {
    providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>, // for fallback order we maintain vector
    fallback_order: Vec<Provider>,
    usage_logger: Option<UsageLogger>,
}

impl fmt::Debug for ProviderRouter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let providers: Vec<_> = self
            .providers
            .keys()
            .map(|provider| provider.as_str())
            .collect();
        f.debug_struct("ProviderRouter")
            .field("providers", &providers)
            .field("fallback_order", &self.fallback_order)
            .field("usage_logger", &self.usage_logger.is_some())
            .finish()
    }
}

impl ProviderRouter {
    /// Builds a router from application configuration and credential store.
    pub async fn from_config(
        config: &AppConfig,
        store: &CredentialStore,
        usage_logger: Option<UsageLogger>,
    ) -> Result<Self, AppError> {
        let mut providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>> = HashMap::new();
        let mut fallback_order: Vec<Provider> = Vec::new();

        // Hugging Face
        let hf_cfg = config.providers.hugging_face.as_ref();
        let hf_token_cfg = hf_cfg.and_then(|cfg| {
            let trimmed = cfg.api_key.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let hf_token = match hf_token_cfg {
            Some(token) => Some(token),
            None => store.get_token(Provider::HuggingFace).await?,
        };

        if let Some(token) = hf_token {
            let base_url = store
                .resolve_base_url(
                    Provider::HuggingFace,
                    hf_cfg.map(|cfg| cfg.api_base_url.as_str()),
                )
                .to_string();
            let client = HuggingFaceClient::new(token, base_url)?;
            drop(providers.insert(Provider::HuggingFace, Arc::new(client)));
            fallback_order.push(Provider::HuggingFace);
        } else {
            debug!(
                provider = "huggingface",
                "Provider not configured (missing credentials)"
            );
        }

        // Google Gemini
        if let Some(google_cfg) = config.providers.google.as_ref() {
            if !google_cfg.api_key.trim().is_empty() {
                let client =
                    GoogleClient::new(google_cfg.api_key.clone(), google_cfg.api_base_url.clone())?;
                drop(providers.insert(Provider::Google, Arc::new(client)));
                fallback_order.push(Provider::Google);
            } else {
                debug!(
                    provider = "google",
                    "Provider not configured (missing API key)"
                );
            }
        }

        Self::from_map_internal(providers, fallback_order, usage_logger)
    }

    /// Convenience constructor for scenarios that build providers manually
    /// (e.g., tests or custom embedding environments).
    pub fn from_map(
        providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>,
        fallback_order: Vec<Provider>,
    ) -> Result<Self, AppError> {
        Self::from_map_internal(providers, fallback_order, None)
    }

    fn from_map_internal(
        providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>,
        fallback_order: Vec<Provider>,
        usage_logger: Option<UsageLogger>,
    ) -> Result<Self, AppError> {
        if providers.is_empty() {
            return Err(AppError::ConfigError(
                "No AI providers supplied to router".into(),
            ));
        }

        Ok(Self {
            providers,
            fallback_order,
            usage_logger,
        })
    }

    /// Attempts to fulfil the request by delegating to an appropriate provider.
    pub async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        for provider in self.select_candidates(request) {
            if let Some(client) = self.providers.get(&provider) {
                let start = Instant::now();
                match client.generate(request).await {
                    Ok(response) => {
                        if let Some(logger) = &self.usage_logger {
                            if let Err(err) = logger
                                .log(provider, true, start.elapsed().as_millis() as i64, None)
                                .await
                            {
                                warn!(provider = %provider, error = %err, "Failed to log provider usage");
                            }
                        }
                        return Ok(response);
                    }
                    Err(err) => {
                        if let Some(logger) = &self.usage_logger {
                            if let Err(log_err) = logger
                                .log(
                                    provider,
                                    false,
                                    start.elapsed().as_millis() as i64,
                                    Some(err.to_string()),
                                )
                                .await
                            {
                                warn!(provider = %provider, error = %log_err, "Failed to log provider usage");
                            }
                        }
                        warn!(provider = %provider, error = %err, "Provider call failed; trying next candidate");
                    }
                }
            }
        }

        Err(AppError::NoProviderAvailable)
    }

    fn select_candidates(&self, request: &AIRequest) -> Vec<Provider> {
        let mut seen = HashSet::new();
        let mut ordered = Vec::new();

        if let Some(provider) = self.provider_from_tags(request) {
            if seen.insert(provider) {
                ordered.push(provider);
            }
        }

        if let Some(provider) = self.provider_from_model(request) {
            if seen.insert(provider) {
                ordered.push(provider);
            }
        }

        for provider in &self.fallback_order {
            if seen.insert(*provider) {
                ordered.push(*provider);
            }
        }

        ordered
    }

    fn provider_from_tags(&self, request: &AIRequest) -> Option<Provider> {
        for tag in &request.tags {
            if let Some(alias) = tag.strip_prefix("provider:") {
                if let Some(provider) = Provider::from_alias(alias.trim()) {
                    if self.providers.contains_key(&provider) {
                        return Some(provider);
                    }
                }
            }
        }
        None
    }

    fn provider_from_model(&self, request: &AIRequest) -> Option<Provider> {
        let model = request.model.to_lowercase();
        if model.contains("gemini") && self.providers.contains_key(&Provider::Google) {
            return Some(Provider::Google);
        }
        if model.contains("huggingface") && self.providers.contains_key(&Provider::HuggingFace) {
            return Some(Provider::HuggingFace);
        }
        if model.contains('/') && self.providers.contains_key(&Provider::HuggingFace) {
            // Hugging Face models frequently use org/model notation; assume HF if slash present
            return Some(Provider::HuggingFace);
        }
        if model.contains("gpt") && self.providers.contains_key(&Provider::OpenAI) {
            return Some(Provider::OpenAI);
        }
        if model.contains("claude") && self.providers.contains_key(&Provider::Anthropic) {
            return Some(Provider::Anthropic);
        }
        if model.contains("cohere") && self.providers.contains_key(&Provider::Cohere) {
            return Some(Provider::Cohere);
        }
        None
    }
}
