use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use crate::storage::repository::Repository;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub repository: Arc<Repository>,
}

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        .with_state(state)
}

/// API v1 routes
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Database connection management (US1)
        .route("/databases", get(placeholder_handler).post(placeholder_handler))
        .route("/databases/:db_name", put(placeholder_handler))
        // Metadata browsing (US2)
        .route("/databases/:db_name/metadata", get(placeholder_handler))
        // Query execution (US3)
        .route("/databases/:db_name/query", post(placeholder_handler))
        // Natural language query (US4)
        .route("/databases/:db_name/nl-query", post(placeholder_handler))
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Placeholder handler for endpoints not yet implemented
async fn placeholder_handler() -> &'static str {
    "Endpoint not implemented yet"
}
