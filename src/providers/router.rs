//! Provider routing utilities and fallback logic.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
    time::Instant,
};

use tracing::{debug, warn};

use crate::{
    catalog::{CatalogStore, ModelEntry},
    config::AppConfig,
    credentials::CredentialStore,
    error::AppError,
    health::HealthTracker,
    models::{AIRequest, AIResponse, RequestComplexity, RequestQuality, RequestSpeed},
    usage::UsageLogger,
};

use super::{
    deepseek::DeepSeekClient, google::GoogleClient, groq::GroqClient,
    hugging_face::HuggingFaceClient, together::TogetherClient, AIProvider, Provider,
};

/// Coordinates AI providers and encapsulates routing logic.
pub struct ProviderRouter {
    providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>, // for fallback order we maintain vector
    fallback_order: Vec<Provider>,
    usage_logger: Option<UsageLogger>,
    catalog: Option<CatalogStore>,
    health_tracker: Option<HealthTracker>,
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
            .field("catalog", &self.catalog.is_some())
            .field("health_tracker", &self.health_tracker.is_some())
            .finish()
    }
}

impl ProviderRouter {
    /// Builds a router from application configuration and credential store.
    pub async fn from_config(
        config: &AppConfig,
        store: &CredentialStore,
        usage_logger: Option<UsageLogger>,
        catalog: Option<CatalogStore>,
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

        // Groq - check encrypted storage first, then config
        let groq_cfg = config.providers.groq.as_ref();
        let groq_token_cfg = groq_cfg.and_then(|cfg| {
            let trimmed = cfg.api_key.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let groq_token = match groq_token_cfg {
            Some(token) => Some(token),
            None => store.get_token(Provider::Groq).await?,
        };
        if let Some(token) = groq_token {
            let base_url = store
                .resolve_base_url(Provider::Groq, groq_cfg.map(|cfg| cfg.api_base_url.as_str()))
                .to_string();
            let client = GroqClient::new(token, base_url)?;
            drop(providers.insert(Provider::Groq, Arc::new(client)));
            fallback_order.push(Provider::Groq);
        } else {
            debug!(provider = "groq", "Provider not configured (missing credentials)");
        }

        // DeepSeek - check encrypted storage first, then config
        let deepseek_cfg = config.providers.deepseek.as_ref();
        let deepseek_token_cfg = deepseek_cfg.and_then(|cfg| {
            let trimmed = cfg.api_key.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let deepseek_token = match deepseek_token_cfg {
            Some(token) => Some(token),
            None => store.get_token(Provider::DeepSeek).await?,
        };
        if let Some(token) = deepseek_token {
            let base_url = store
                .resolve_base_url(
                    Provider::DeepSeek,
                    deepseek_cfg.map(|cfg| cfg.api_base_url.as_str()),
                )
                .to_string();
            let client = DeepSeekClient::new(token, base_url)?;
            drop(providers.insert(Provider::DeepSeek, Arc::new(client)));
            fallback_order.push(Provider::DeepSeek);
        } else {
            debug!(provider = "deepseek", "Provider not configured (missing credentials)");
        }

        // Together AI - check encrypted storage first, then config
        let together_cfg = config.providers.together.as_ref();
        let together_token_cfg = together_cfg.and_then(|cfg| {
            let trimmed = cfg.api_key.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        let together_token = match together_token_cfg {
            Some(token) => Some(token),
            None => store.get_token(Provider::Together).await?,
        };
        if let Some(token) = together_token {
            let base_url = store
                .resolve_base_url(
                    Provider::Together,
                    together_cfg.map(|cfg| cfg.api_base_url.as_str()),
                )
                .to_string();
            let client = TogetherClient::new(token, base_url)?;
            drop(providers.insert(Provider::Together, Arc::new(client)));
            fallback_order.push(Provider::Together);
        } else {
            debug!(provider = "together", "Provider not configured (missing credentials)");
        }

        Self::from_map_internal(providers, fallback_order, usage_logger, catalog)
    }

    /// Convenience constructor for scenarios that build providers manually
    /// (e.g., tests or custom embedding environments).
    pub fn from_map(
        providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>,
        fallback_order: Vec<Provider>,
    ) -> Result<Self, AppError> {
        Self::from_map_internal(providers, fallback_order, None, None)
    }

    fn from_map_internal(
        providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>>,
        fallback_order: Vec<Provider>,
        usage_logger: Option<UsageLogger>,
        catalog: Option<CatalogStore>,
    ) -> Result<Self, AppError> {
        if providers.is_empty() {
            return Err(AppError::ConfigError(
                "No AI providers supplied to router".into(),
            ));
        }

        // Create health tracker from usage logger's pool if available
        let health_tracker = usage_logger.as_ref().map(|logger| {
            HealthTracker::new(logger.pool())
        });

        Ok(Self {
            providers,
            fallback_order,
            usage_logger,
            catalog,
            health_tracker,
        })
    }

    /// Attempts to fulfil the request by delegating to an appropriate provider.
    pub async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        for provider in self.select_candidates(request) {
            // Check provider health before attempting to use it
            if let Some(health_tracker) = &self.health_tracker {
                match health_tracker.is_available(provider).await {
                    Ok(false) => {
                        debug!(provider = %provider, "Skipping unavailable provider");
                        continue;
                    }
                    Err(err) => {
                        warn!(provider = %provider, error = %err, "Failed to check provider health");
                        // Continue anyway - don't let health check errors block routing
                    }
                    Ok(true) => {}
                }
            }

            if let Some(client) = self.providers.get(&provider) {
                let mut routed_request = request.clone();
                if routed_request.model.is_empty() {
                    if let Some(model) = self.pick_model(provider, &routed_request).await? {
                        routed_request.model = model;
                    }
                }

                let start = Instant::now();
                match client.generate(&routed_request).await {
                    Ok(response) => {
                        // Record successful call
                        if let Some(health_tracker) = &self.health_tracker {
                            if let Err(err) = health_tracker.record_success(provider).await {
                                warn!(provider = %provider, error = %err, "Failed to record provider success");
                            }
                        }

                        if let Some(logger) = &self.usage_logger {
                            if let Err(err) = logger
                                .log(
                                    provider,
                                    Some(routed_request.model.as_str()),
                                    true,
                                    start.elapsed().as_millis() as i64,
                                    None,
                                )
                                .await
                            {
                                warn!(provider = %provider, error = %err, "Failed to log provider usage");
                            }
                        }
                        return Ok(response);
                    }
                    Err(err) => {
                        let error_message = err.to_string();

                        // Record failed call with error classification
                        if let Some(health_tracker) = &self.health_tracker {
                            if let Err(health_err) = health_tracker
                                .record_failure(provider, &error_message)
                                .await
                            {
                                warn!(provider = %provider, error = %health_err, "Failed to record provider failure");
                            }
                        }

                        if let Some(logger) = &self.usage_logger {
                            if let Err(log_err) = logger
                                .log(
                                    provider,
                                    Some(routed_request.model.as_str()),
                                    false,
                                    start.elapsed().as_millis() as i64,
                                    Some(error_message.clone()),
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

        if let Some(provider) = self.provider_from_hints(request) {
            if seen.insert(provider) {
                ordered.push(provider);
            }
        }

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

        for provider in self.hint_preferred_providers(request) {
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

    fn provider_from_hints(&self, request: &AIRequest) -> Option<Provider> {
        request
            .hints
            .provider
            .as_ref()
            .and_then(|alias| Provider::from_alias(alias))
            .filter(|provider| self.providers.contains_key(provider))
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

        // Only match provider-specific model name patterns
        // For ambiguous names, rely on fallback_order to try providers
        if model.contains("gemini") && self.providers.contains_key(&Provider::Google) {
            return Some(Provider::Google);
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
        if model.contains("deepseek") && self.providers.contains_key(&Provider::DeepSeek) {
            return Some(Provider::DeepSeek);
        }
        if model.contains("llama") && model.contains("groq") && self.providers.contains_key(&Provider::Groq) {
            return Some(Provider::Groq);
        }

        // For generic model names (e.g., "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free"),
        // don't guess - let the fallback_order try providers in health priority order.
        // The router will automatically skip unhealthy providers and find the first
        // available one that accepts the model.
        None
    }

    fn hint_preferred_providers(&self, request: &AIRequest) -> Vec<Provider> {
        let mut picks = Vec::new();

        if matches!(request.hints.quality, Some(RequestQuality::Premium))
            || matches!(request.hints.complexity, Some(RequestComplexity::High))
        {
            if self.providers.contains_key(&Provider::HuggingFace) {
                picks.push(Provider::HuggingFace);
            }
        }

        if matches!(request.hints.speed, Some(RequestSpeed::Fast))
            && self.providers.contains_key(&Provider::Google)
        {
            picks.push(Provider::Google);
        }

        picks
    }

    async fn pick_model(
        &self,
        provider: Provider,
        request: &AIRequest,
    ) -> Result<Option<String>, AppError> {
        // If request already has a model specified, use it
        if !request.model.is_empty() {
            return Ok(Some(request.model.clone()));
        }

        // Otherwise, look up the default model for this provider/workload in the catalog
        let workload = request.hints.workload;
        if let Some(catalog) = &self.catalog {
            let models = catalog
                .active_models(provider, workload)
                .await?
                .into_iter()
                .collect::<Vec<ModelEntry>>();
            if let Some(entry) = models.first() {
                return Ok(Some(entry.model.clone()));
            }
        }

        Ok(None)
    }
}
