//! Defines the API routes and handlers for the web server.

use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};

use crate::{
    error::AppError,
    models::{AIRequest, AIResponse},
    providers::ProviderRouter,
};

/// Shared application state passed into route handlers.
#[derive(Clone, Debug)]
pub struct AppState {
    provider_router: Arc<ProviderRouter>,
}

impl AppState {
    /// Creates a new `AppState` instance.
    pub fn new(provider_router: Arc<ProviderRouter>) -> Self {
        Self { provider_router }
    }

    fn provider_router(&self) -> &ProviderRouter {
        self.provider_router.as_ref()
    }
}

/// Creates the main API router for the application.
pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/generate", post(generate_handler))
        .with_state(state)
}

/// Handler for the `/api/v1/generate` endpoint.
async fn generate_handler(
    State(state): State<AppState>,
    Json(payload): Json<AIRequest>,
) -> Result<Json<AIResponse>, AppError> {
    tracing::info!(model = %payload.model, tags = ?payload.tags, "Received generation request");

    let response = state.provider_router().generate(&payload).await?;

    Ok(Json(response))
}
