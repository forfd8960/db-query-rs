mod api;
mod config;
mod models;
mod services;
mod storage;
mod validation;

use anyhow::{Context, Result};
use std::sync::Arc;
use tower_http::{cors::{Any, CorsLayer}, normalize_path::NormalizePathLayer};
use tracing_subscriber;

use crate::{
    api::routes::{create_router, AppState},
    config::Config,
    storage::repository::Repository,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration from environment
    let config = Config::from_env()?;

    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting db-query-rs server...");

    // Initialize SQLite repository (runs migrations on startup)
    let repository = Repository::new(&config.database_path)
        .await
        .context("Failed to initialize SQLite repository")?;
    
    tracing::info!("SQLite database initialized at {}", config.database_path);

    // Health check
    repository.health_check().await?;
    tracing::info!("Database health check passed");

    // Create application state
    let app_state = AppState {
        repository: Arc::new(repository),
    };

    // Setup CORS middleware (allow all origins for development)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create router with CORS
    let app = create_router(app_state).layer(cors);

    // Bind server to configured port
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;

    tracing::info!("Server listening on {}", addr);

    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    tracing::info!("Server shut down gracefully");

    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            tracing::info!("Received terminate signal");
        }
    }
}
