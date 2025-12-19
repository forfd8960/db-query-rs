use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Request DTOs
// ============================================================================

/// Request to add a new database connection
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDatabaseRequest {
    pub url: String,
}

/// Request to update a database connection
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatabaseRequest {
    pub url: String,
}

/// Request to execute SQL query
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteQueryRequest {
    pub sql: String,
}

/// Request to execute natural language query
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalLanguageQueryRequest {
    pub nl_query: String,
}

// ============================================================================
// Response DTOs
// ============================================================================

/// Database connection response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConnectionDto {
    pub id: i64,
    pub name: String,
    pub connection_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List of database connections
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseListResponse {
    pub databases: Vec<DatabaseConnectionDto>,
}

/// Table metadata response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableMetadataDto {
    pub schema_name: String,
    pub table_name: String,
    pub table_type: String, // "TABLE" or "VIEW"
    pub columns: Vec<ColumnMetadataDto>,
}

/// Column metadata response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMetadataDto {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
}

/// Database metadata response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMetadataResponse {
    pub tables: Vec<TableMetadataDto>,
}

/// Query result response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResultDto {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>, // Array of objects with camelCase keys
    pub row_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time: Option<f64>, // milliseconds
}

/// Natural language query result response (camelCase)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalLanguageQueryResultDto {
    pub nl_query: String,
    pub generated_sql: String,
    pub result: QueryResultDto,
}
