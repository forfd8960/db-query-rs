use chrono::{DateTime, Utc};

/// Table metadata model (persisted in SQLite)
#[derive(Debug, Clone)]
pub struct TableMetadata {
    pub id: i64,
    pub database_id: i64,
    pub schema_name: String,
    pub table_name: String,
    pub table_type: String, // "TABLE" or "VIEW"
    pub cached_at: DateTime<Utc>,
    pub columns: Vec<ColumnMetadata>,
}

/// Column metadata model (persisted in SQLite)
#[derive(Debug, Clone)]
pub struct ColumnMetadata {
    pub id: i64,
    pub table_id: i64,
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub ordinal_position: i32,
    pub is_primary_key: bool,
}

/// Raw metadata row from PostgreSQL information_schema
#[derive(Debug)]
pub struct RawMetadataRow {
    pub schema_name: String,
    pub table_name: String,
    pub table_type: String,
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: String, // "YES" or "NO"
    pub column_default: Option<String>,
    pub ordinal_position: i32,
}
