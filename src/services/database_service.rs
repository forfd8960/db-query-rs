use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool, PgPool};
use crate::models::{database::DatabaseConnection, error::AppError};

/// Database connection service
pub struct DatabaseService {
    sqlite_pool: SqlitePool,
}

impl DatabaseService {
    pub fn new(sqlite_pool: SqlitePool) -> Self {
        Self { sqlite_pool }
    }

    /// Add a new database connection
    /// 
    /// 1. Validates PostgreSQL URL format
    /// 2. Tests connection to verify credentials
    /// 3. Extracts database name from URL
    /// 4. Stores connection in SQLite
    pub async fn add_database(&self, connection_url: &str) -> Result<DatabaseConnection, AppError> {
        // Validate URL format
        crate::models::database::validate_postgres_url(connection_url)
            .map_err(|e| AppError::ValidationError(e))?;
        
        // Extract database name
        let db_name = crate::models::database::extract_db_name(connection_url)
            .map_err(|e| AppError::ValidationError(e))?;
        
        // Test connection to PostgreSQL
        self.test_postgres_connection(connection_url).await
            .map_err(|e| AppError::DatabaseError(format!("Connection test failed: {}", e)))?;
        
        // Insert into SQLite
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO database_connections (name, connection_url, created_at, updated_at)
             VALUES (?, ?, ?, ?)"
        )
        .bind(&db_name)
        .bind(connection_url)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.sqlite_pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AppError::ValidationError(format!("Database '{}' already exists", db_name))
            } else {
                AppError::DatabaseError(e.to_string())
            }
        })?;
        
        let id = result.last_insert_rowid();
        
        Ok(DatabaseConnection {
            id,
            name: db_name,
            connection_url: connection_url.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// List all database connections with credentials masked
    pub async fn list_databases(&self) -> Result<Vec<DatabaseConnection>, AppError> {
        let rows = sqlx::query(
            "SELECT id, name, connection_url, created_at, updated_at
             FROM database_connections
             ORDER BY created_at DESC"
        )
        .fetch_all(&self.sqlite_pool)
        .await?;
        
        let connections: Vec<DatabaseConnection> = rows
            .iter()
            .map(|row| {
                let created_at_str: String = row.get("created_at");
                let updated_at_str: String = row.get("updated_at");
                
                DatabaseConnection {
                    id: row.get("id"),
                    name: row.get("name"),
                    connection_url: row.get("connection_url"),
                    created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect();
        
        Ok(connections)
    }

    /// Update database connection URL
    /// 
    /// 1. Validates new URL format
    /// 2. Tests new connection
    /// 3. Updates in SQLite
    pub async fn update_database(&self, db_name: &str, new_url: &str) -> Result<DatabaseConnection, AppError> {
        // Validate URL format
        crate::models::database::validate_postgres_url(new_url)
            .map_err(|e| AppError::ValidationError(e))?;
        
        // Test new connection
        self.test_postgres_connection(new_url).await
            .map_err(|e| AppError::DatabaseError(format!("Connection test failed: {}", e)))?;
        
        // Check if database exists
        let existing = sqlx::query("SELECT id FROM database_connections WHERE name = ?")
            .bind(db_name)
            .fetch_optional(&self.sqlite_pool)
            .await?;
        
        if existing.is_none() {
            return Err(AppError::NotFound(format!("Database '{}' not found", db_name)));
        }
        
        // Update in SQLite
        let now = Utc::now();
        sqlx::query(
            "UPDATE database_connections
             SET connection_url = ?, updated_at = ?
             WHERE name = ?"
        )
        .bind(new_url)
        .bind(now.to_rfc3339())
        .bind(db_name)
        .execute(&self.sqlite_pool)
        .await?;
        
        // Fetch updated connection
        let row = sqlx::query(
            "SELECT id, name, connection_url, created_at, updated_at
             FROM database_connections
             WHERE name = ?"
        )
        .bind(db_name)
        .fetch_one(&self.sqlite_pool)
        .await?;
        
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");
        
        Ok(DatabaseConnection {
            id: row.get("id"),
            name: row.get("name"),
            connection_url: row.get("connection_url"),
            created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
            updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
        })
    }

    /// Get a database connection by name
    pub async fn get_database(&self, db_name: &str) -> Result<DatabaseConnection, AppError> {
        let row = sqlx::query(
            "SELECT id, name, connection_url, created_at, updated_at
             FROM database_connections
             WHERE name = ?"
        )
        .bind(db_name)
        .fetch_optional(&self.sqlite_pool)
        .await?;
        
        match row {
            Some(row) => {
                let created_at_str: String = row.get("created_at");
                let updated_at_str: String = row.get("updated_at");
                
                Ok(DatabaseConnection {
                    id: row.get("id"),
                    name: row.get("name"),
                    connection_url: row.get("connection_url"),
                    created_at: created_at_str.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: updated_at_str.parse().unwrap_or_else(|_| Utc::now()),
                })
            }
            None => Err(AppError::NotFound(format!("Database '{}' not found", db_name))),
        }
    }

    /// Test PostgreSQL connection
    async fn test_postgres_connection(&self, connection_url: &str) -> Result<()> {
        let pool = PgPool::connect(connection_url)
            .await
            .context("Failed to connect to PostgreSQL database")?;
        
        // Execute simple query to verify connection works
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .context("Connection test query failed")?;
        
        pool.close().await;
        
        Ok(())
    }
}
