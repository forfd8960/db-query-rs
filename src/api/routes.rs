
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use crate::storage::repository::Repository;
use crate::api::handlers::{database_handlers, metadata_handlers, query_handlers};

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
        .route("/databases", 
            get(database_handlers::list_databases)
            .post(database_handlers::add_database))
        .route("/databases/{db_name}", 
            put(database_handlers::update_database))
        // Metadata browsing (US2)
        .route("/databases/{db_name}/metadata", 
            get(metadata_handlers::get_metadata))
        // Query execution (US3)
        .route("/databases/{db_name}/query", 
            post(query_handlers::execute_query))
        // Natural language query (US4)
        .route("/databases/{db_name}/nl-query", 
            post(query_handlers::execute_nl_query))
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}
