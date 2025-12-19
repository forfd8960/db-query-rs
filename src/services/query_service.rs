use anyhow::Context;
use serde_json::{json, Map, Value};
use sqlx::{Column, PgPool, Row, TypeInfo};
use sqlx::types::uuid::Uuid;
use std::time::Instant;

use crate::models::{error::AppError, query::QueryResult};
use crate::validation::sql_validator::SqlValidator;

/// Query execution service
pub struct QueryService;

impl QueryService {
    /// Execute SQL query against PostgreSQL database
    /// 
    /// 1. Validates SQL is SELECT-only
    /// 2. Injects LIMIT 1000 if missing
    /// 3. Executes query
    /// 4. Transforms results to camelCase JSON
    pub async fn execute_query(
        connection_url: &str,
        sql: &str,
    ) -> Result<QueryResult, AppError> {
        // Validate SELECT-only
        SqlValidator::validate_select_only(sql)?;

        // Inject LIMIT if missing
        let safe_sql = SqlValidator::inject_limit_if_missing(sql)?;

        // Connect to PostgreSQL
        let pg_pool = PgPool::connect(connection_url)
            .await
            .context("Failed to connect to PostgreSQL")?;

        // Execute query and measure time
        let start = Instant::now();
        let rows = sqlx::query(&safe_sql)
            .fetch_all(&pg_pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Query execution failed: {}", e)))?;
        let execution_time = start.elapsed().as_secs_f64() * 1000.0; // milliseconds

        pg_pool.close().await;

        // Extract columns
        let columns: Vec<String> = if let Some(first_row) = rows.first() {
            first_row
                .columns()
                .iter()
                .map(|col| col.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        // Transform rows to JSON with camelCase keys
        let json_rows: Vec<Value> = rows
            .iter()
            .map(|row| {
                let mut obj = Map::new();
                for col in row.columns() {
                    let col_name = col.name();
                    let camel_case_name = to_camel_case(col_name);

                    // Extract value based on type
                    let value = Self::extract_value(row, col);
                    obj.insert(camel_case_name, value);
                }
                Value::Object(obj)
            })
            .collect();

        let row_count = json_rows.len();

        Ok(QueryResult {
            columns,
            rows: json_rows,
            row_count,
            execution_time: Some(execution_time),
        })
    }

    /// Extract value from row based on column type
    fn extract_value(row: &sqlx::postgres::PgRow, col: &sqlx::postgres::PgColumn) -> Value {
        let col_name = col.name();
        let type_info = col.type_info();

        // Handle common PostgreSQL types
        match type_info.name() {
            "BOOL" => row
                .try_get::<bool, _>(col_name)
                .map(Value::Bool)
                .unwrap_or(Value::Null),
            "INT2" | "INT4" => row
                .try_get::<i32, _>(col_name)
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            "INT8" => row
                .try_get::<i64, _>(col_name)
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            "FLOAT4" => row
                .try_get::<f32, _>(col_name)
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            "FLOAT8" => row
                .try_get::<f64, _>(col_name)
                .map(|v| json!(v))
                .unwrap_or(Value::Null),
            "TEXT" | "VARCHAR" | "CHAR" | "NAME" => row
                .try_get::<String, _>(col_name)
                .map(Value::String)
                .unwrap_or(Value::Null),
            "TIMESTAMPTZ" | "TIMESTAMP" => row
                .try_get::<chrono::NaiveDateTime, _>(col_name)
                .or_else(|_| row.try_get::<chrono::DateTime<chrono::Utc>, _>(col_name).map(|dt| dt.naive_utc()))
                .map(|dt| Value::String(dt.to_string()))
                .unwrap_or(Value::Null),
            "DATE" => row
                .try_get::<chrono::NaiveDate, _>(col_name)
                .map(|d| Value::String(d.to_string()))
                .unwrap_or(Value::Null),
            "UUID" => row
                .try_get::<Uuid, _>(col_name)
                .map(|u: Uuid| Value::String(u.to_string()))
                .unwrap_or(Value::Null),
            "JSON" | "JSONB" => row
                .try_get::<Value, _>(col_name)
                .unwrap_or(Value::Null),
            _ => {
                // Try as string for unknown types
                row.try_get::<String, _>(col_name)
                    .map(Value::String)
                    .unwrap_or(Value::Null)
            }
        }
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("user_id"), "userId");
        assert_eq!(to_camel_case("first_name"), "firstName");
        assert_eq!(to_camel_case("created_at"), "createdAt");
        assert_eq!(to_camel_case("id"), "id");
    }
}
