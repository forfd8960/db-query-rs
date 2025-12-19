use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, Row, SqlitePool};
use std::collections::HashMap;

use crate::models::{
    error::AppError,
    metadata::{ColumnMetadata, TableMetadata},
};

/// Metadata service for fetching and caching database schema
pub struct MetadataService {
    sqlite_pool: SqlitePool,
}

impl MetadataService {
    pub fn new(sqlite_pool: SqlitePool) -> Self {
        Self { sqlite_pool }
    }

    /// Get or fetch metadata for a database
    /// 
    /// 1. Check SQLite cache
    /// 2. If not cached, fetch from PostgreSQL
    /// 3. Cache in SQLite
    /// 4. Return metadata
    pub async fn get_or_fetch_metadata(
        &self,
        database_id: i64,
        connection_url: &str,
    ) -> Result<Vec<TableMetadata>, AppError> {
        // Check cache first
        if let Ok(cached) = self.get_cached_metadata(database_id).await {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }

        // Fetch from PostgreSQL
        let metadata = self.fetch_metadata_from_postgres(connection_url).await?;

        // Cache in SQLite
        self.cache_metadata_to_sqlite(database_id, &metadata).await?;

        // Return cached version with IDs
        self.get_cached_metadata(database_id).await
    }

    /// Fetch metadata from PostgreSQL using information_schema
    async fn fetch_metadata_from_postgres(
        &self,
        connection_url: &str,
    ) -> Result<Vec<TableMetadata>, AppError> {
        let pg_pool = PgPool::connect(connection_url)
            .await
            .context("Failed to connect to PostgreSQL")?;

        // Combined query to get tables, columns, and primary keys
        let query = r#"
            SELECT 
                t.table_schema,
                t.table_name,
                t.table_type,
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                c.ordinal_position,
                CASE 
                    WHEN tc.constraint_type = 'PRIMARY KEY' THEN true 
                    ELSE false 
                END as is_primary_key
            FROM information_schema.tables t
            INNER JOIN information_schema.columns c 
                ON t.table_schema = c.table_schema 
                AND t.table_name = c.table_name
            LEFT JOIN information_schema.key_column_usage kcu
                ON c.table_schema = kcu.table_schema
                AND c.table_name = kcu.table_name
                AND c.column_name = kcu.column_name
            LEFT JOIN information_schema.table_constraints tc
                ON kcu.constraint_name = tc.constraint_name
                AND tc.constraint_type = 'PRIMARY KEY'
            WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY t.table_schema, t.table_name, c.ordinal_position
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&pg_pool)
            .await
            .context("Failed to fetch metadata from PostgreSQL")?;

        pg_pool.close().await;

        // Group by table
        let mut tables_map: HashMap<(String, String), TableMetadata> = HashMap::new();

        for row in rows {
            let schema_name: String = row.get("table_schema");
            let table_name: String = row.get("table_name");
            let table_key = (schema_name.clone(), table_name.clone());

            let table = tables_map.entry(table_key.clone()).or_insert_with(|| {
                TableMetadata {
                    id: 0, // Will be set when cached
                    database_id: 0,
                    schema_name: schema_name.clone(),
                    table_name: table_name.clone(),
                    table_type: row.get("table_type"),
                    cached_at: Utc::now(),
                    columns: Vec::new(),
                }
            });

            let column = ColumnMetadata {
                id: 0, // Will be set when cached
                table_id: 0,
                column_name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get::<String, _>("is_nullable") == "YES",
                column_default: row.get("column_default"),
                ordinal_position: row.get("ordinal_position"),
                is_primary_key: row.get("is_primary_key"),
            };

            table.columns.push(column);
        }

        let tables: Vec<TableMetadata> = tables_map.into_values().collect();

        Ok(tables)
    }

    /// Cache metadata to SQLite
    async fn cache_metadata_to_sqlite(
        &self,
        database_id: i64,
        tables: &[TableMetadata],
    ) -> Result<(), AppError> {
        // Clear existing cache for this database
        sqlx::query("DELETE FROM table_metadata WHERE database_id = ?")
            .bind(database_id)
            .execute(&self.sqlite_pool)
            .await?;

        let now = Utc::now().to_rfc3339();

        for table in tables {
            // Insert table
            let table_result = sqlx::query(
                "INSERT INTO table_metadata (database_id, schema_name, table_name, table_type, cached_at)
                 VALUES (?, ?, ?, ?, ?)"
            )
            .bind(database_id)
            .bind(&table.schema_name)
            .bind(&table.table_name)
            .bind(&table.table_type)
            .bind(&now)
            .execute(&self.sqlite_pool)
            .await?;

            let table_id = table_result.last_insert_rowid();

            // Insert columns
            for column in &table.columns {
                sqlx::query(
                    "INSERT INTO column_metadata 
                     (table_id, column_name, data_type, is_nullable, column_default, ordinal_position, is_primary_key)
                     VALUES (?, ?, ?, ?, ?, ?, ?)"
                )
                .bind(table_id)
                .bind(&column.column_name)
                .bind(&column.data_type)
                .bind(column.is_nullable)
                .bind(&column.column_default)
                .bind(column.ordinal_position)
                .bind(column.is_primary_key)
                .execute(&self.sqlite_pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Get cached metadata from SQLite
    async fn get_cached_metadata(&self, database_id: i64) -> Result<Vec<TableMetadata>, AppError> {
        // Fetch tables
        let table_rows = sqlx::query(
            "SELECT id, schema_name, table_name, table_type, cached_at
             FROM table_metadata
             WHERE database_id = ?
             ORDER BY schema_name, table_name"
        )
        .bind(database_id)
        .fetch_all(&self.sqlite_pool)
        .await?;

        let mut tables = Vec::new();

        for table_row in table_rows {
            let table_id: i64 = table_row.get("id");
            let cached_at_str: String = table_row.get("cached_at");

            // Fetch columns for this table
            let column_rows = sqlx::query(
                "SELECT id, column_name, data_type, is_nullable, column_default, ordinal_position, is_primary_key
                 FROM column_metadata
                 WHERE table_id = ?
                 ORDER BY ordinal_position"
            )
            .bind(table_id)
            .fetch_all(&self.sqlite_pool)
            .await?;

            let columns: Vec<ColumnMetadata> = column_rows
                .iter()
                .map(|row| ColumnMetadata {
                    id: row.get("id"),
                    table_id,
                    column_name: row.get("column_name"),
                    data_type: row.get("data_type"),
                    is_nullable: row.get("is_nullable"),
                    column_default: row.get("column_default"),
                    ordinal_position: row.get("ordinal_position"),
                    is_primary_key: row.get("is_primary_key"),
                })
                .collect();

            tables.push(TableMetadata {
                id: table_id,
                database_id,
                schema_name: table_row.get("schema_name"),
                table_name: table_row.get("table_name"),
                table_type: table_row.get("table_type"),
                cached_at: cached_at_str.parse().unwrap_or_else(|_| Utc::now()),
                columns,
            });
        }

        Ok(tables)
    }
}
