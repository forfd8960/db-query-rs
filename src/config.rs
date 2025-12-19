use anyhow::{Context, Result};
use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// OpenAI API key for natural language query translation (US4)
    pub openai_api_key: String,
    
    /// Server port (default: 3000)
    pub port: u16,
    
    /// SQLite database path for metadata cache (default: ./db-query/db_query.db)
    pub database_path: String,
    
    /// Log level for tracing (default: info)
    pub log_level: String,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// Required variables:
    /// - OPENAI_API_KEY: OpenAI API key
    /// 
    /// Optional variables:
    /// - PORT: Server port (default: 3000)
    /// - DATABASE_PATH: SQLite database path (default: ./db-query/db_query.db)
    /// - RUST_LOG: Log level (default: info)
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (for local development)
        dotenvy::dotenv().ok();
        
        let openai_api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY must be set in environment or .env file")?;
        
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("PORT must be a valid u16 number")?;
        
        let database_path = env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "./db-query/db_query.db".to_string());
        
        let log_level = env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string());
        
        Ok(Config {
            openai_api_key,
            port,
            database_path,
            log_level,
        })
    }
}
