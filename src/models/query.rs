use serde_json::Value;

/// Query result model
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Value>, // JSON objects with camelCase keys
    pub row_count: usize,
    pub execution_time: Option<f64>, // milliseconds
}

/// Natural language query request (transient - not persisted)
#[derive(Debug, Clone)]
pub struct NaturalLanguageRequest {
    pub nl_query: String,
    pub generated_sql: String,
    pub result: QueryResult,
}
