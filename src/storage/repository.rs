use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;
use std::path::Path;
use tokio::fs;

/// SQLite repository for metadata caching
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    /// Create a new repository with the given SQLite database path
    /// 
    /// Creates the database directory if it doesn't exist
    /// Runs migrations on startup to initialize schema
    pub async fn new(database_path: &str) -> Result<Self> {
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(database_path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .await
                    .context("Failed to create database directory")?;
            }
        }

        // Create connection pool with mode=rwc to create database file if it doesn't exist
        let connection_string = format!("sqlite://{}?mode=rwc", database_path);
        let pool = SqlitePool::connect(&connection_string)
            .await
            .context("Failed to connect to SQLite database")?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    /// Run SQL migrations from migrations/ directory
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Read and execute the initial schema migration
        let migration_sql = include_str!("../../migrations/001_initial_schema.sql");
        
        sqlx::raw_sql(migration_sql)
            .execute(pool)
            .await
            .context("Failed to run migrations")?;

        Ok(())
    }

    /// Get the connection pool for database operations
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Health check - verify database is accessible
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .context("Database health check failed")?;
        Ok(())
    }
}
