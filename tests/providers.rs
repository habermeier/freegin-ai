#![allow(missing_docs)]

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{self, Body},
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use freegin_ai::{
    error::AppError,
    models::{AIRequest, AIResponse, RequestHints},
    providers::{AIProvider, Provider, ProviderRouter},
    routes::{api_router, AppState},
};

struct EchoProvider {
    provider: Provider,
}

#[async_trait]
impl AIProvider for EchoProvider {
    async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError> {
        Ok(AIResponse {
            content: format!("echo: {}", request.prompt),
            provider: self.provider,
        })
    }
}

#[tokio::test]
async fn generate_returns_mock_response() -> anyhow::Result<()> {
    let mut providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>> = HashMap::new();
    drop(providers.insert(
        Provider::HuggingFace,
        Arc::new(EchoProvider {
            provider: Provider::HuggingFace,
        }),
    ));
    let router = ProviderRouter::from_map(providers, vec![Provider::HuggingFace])?;
    let state = AppState::new(Arc::new(router));
    let app = api_router(state);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/generate")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"model":"huggingface/awesome","prompt":"Hello","tags":["provider:hf"]}"#,
        ))?;

    let response = app.oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = body::to_bytes(response.into_body(), usize::MAX).await?;
    let payload: AIResponse = serde_json::from_slice(&bytes)?;

    assert_eq!(payload.content, "echo: Hello");
    assert_eq!(payload.provider, Provider::HuggingFace);

    Ok(())
}

#[tokio::test]
async fn router_prefers_provider_hint() -> anyhow::Result<()> {
    let mut providers: HashMap<Provider, Arc<dyn AIProvider + Send + Sync>> = HashMap::new();
    drop(providers.insert(
        Provider::Google,
        Arc::new(EchoProvider {
            provider: Provider::Google,
        }),
    ));
    drop(providers.insert(
        Provider::HuggingFace,
        Arc::new(EchoProvider {
            provider: Provider::HuggingFace,
        }),
    ));
    let router =
        ProviderRouter::from_map(providers, vec![Provider::Google, Provider::HuggingFace])?;

    let request = AIRequest {
        model: String::new(),
        prompt: "Hello".into(),
        tags: Vec::new(),
        context: Vec::new(),
        metadata: HashMap::new(),
        hints: RequestHints {
            provider: Some("huggingface".into()),
            ..RequestHints::default()
        },
    };

    let response = router.generate(&request).await?;
    assert_eq!(response.provider, Provider::HuggingFace);

    Ok(())
}
