use anyhow::{Context, Result};
use openai_api_rust::chat::*;
use openai_api_rust::*;

use crate::models::{error::AppError, metadata::TableMetadata};

/// LLM service for natural language to SQL translation
pub struct LlmService {
    client: OpenAI,
}

impl LlmService {
    /// Initialize OpenAI client from environment variable
    pub fn new() -> Result<Self, AppError> {
        let auth = Auth::from_env()
            .map_err(|e| AppError::ConfigError(format!("Failed to initialize OpenAI client: {}. Ensure OPENAI_API_KEY is set.", e)))?;
        
        let client = OpenAI::new(auth, "https://api.openai.com/v1/");
        
        Ok(Self { client })
    }

    /// Generate SQL from natural language query
    /// 
    /// 1. Builds prompt with schema context
    /// 2. Calls OpenAI Chat Completion API
    /// 3. Parses response to extract SQL
    pub async fn generate_sql_from_nl(
        &self,
        nl_query: &str,
        tables: &[TableMetadata],
    ) -> Result<String, AppError> {
        // Build prompt with schema
        let prompt = Self::build_prompt_with_schema(nl_query, tables);

        // Call OpenAI API
        let body = ChatBody {
            model: "gpt-4".to_string(),
            max_tokens: Some(500),
            temperature: Some(0.0), // Deterministic output
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: vec![Message {
                role: Role::User,
                content: prompt,
            }],
        };

        let response = self
            .client
            .chat_completion_create(&body)
            .map_err(|e| AppError::LlmError(format!("OpenAI API call failed: {}", e)))?;

        // Extract SQL from response
        if let Some(choice) = response.choices.first() {
            let sql = Self::parse_llm_response(&choice.message.as_ref().unwrap().content);
            Ok(sql)
        } else {
            Err(AppError::LlmError(
                "No response from OpenAI API".to_string(),
            ))
        }
    }

    /// Build prompt with database schema context
    /// 
    /// Uses template from research.md with table/column metadata
    pub fn build_prompt_with_schema(nl_query: &str, tables: &[TableMetadata]) -> String {
        let mut schema_text = String::new();

        for table in tables {
            schema_text.push_str(&format!(
                "\nTable: {}.{} ({})\n",
                table.schema_name, table.table_name, table.table_type
            ));

            schema_text.push_str("Columns:\n");
            for col in &table.columns {
                let nullable = if col.is_nullable { "NULL" } else { "NOT NULL" };
                let pk = if col.is_primary_key { " PRIMARY KEY" } else { "" };
                schema_text.push_str(&format!(
                    "  - {} ({}, {}{})\n",
                    col.column_name, col.data_type, nullable, pk
                ));
            }
        }

        format!(
            r#"You are a PostgreSQL SQL query generator. Generate ONLY a SELECT statement based on the user's request.

Database Schema:
{}

Rules:
1. Generate ONLY SELECT statements (no INSERT, UPDATE, DELETE, DROP)
2. Use proper PostgreSQL syntax
3. Reference only tables and columns that exist in the schema
4. Include appropriate WHERE clauses for filtering
5. Add ORDER BY if sorting is mentioned
6. Return ONLY the SQL query, no explanations
7. Do not include markdown code fences or formatting

User Request: {}

SQL Query:"#,
            schema_text.trim(),
            nl_query
        )
    }

    /// Parse LLM response to extract SQL
    /// 
    /// Strips markdown code fences (```sql...```) and cleans formatting
    pub fn parse_llm_response(response: &str) -> String {
        let mut sql = response.trim().to_string();

        // Remove markdown code fences
        if sql.starts_with("```sql") {
            sql = sql.trim_start_matches("```sql").to_string();
        } else if sql.starts_with("```") {
            sql = sql.trim_start_matches("```").to_string();
        }

        if sql.ends_with("```") {
            sql = sql.trim_end_matches("```").to_string();
        }

        // Trim whitespace
        sql = sql.trim().to_string();

        // Remove trailing semicolon if present
        sql = sql.trim_end_matches(';').trim().to_string();

        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llm_response_with_code_fence() {
        let response = "```sql\nSELECT * FROM users LIMIT 10\n```";
        let sql = LlmService::parse_llm_response(response);
        assert_eq!(sql, "SELECT * FROM users LIMIT 10");
    }

    #[test]
    fn test_parse_llm_response_without_code_fence() {
        let response = "SELECT * FROM users WHERE id > 10";
        let sql = LlmService::parse_llm_response(response);
        assert_eq!(sql, "SELECT * FROM users WHERE id > 10");
    }

    #[test]
    fn test_parse_llm_response_with_trailing_semicolon() {
        let response = "SELECT * FROM users;";
        let sql = LlmService::parse_llm_response(response);
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_build_prompt_with_schema() {
        use chrono::Utc;
        use crate::models::metadata::ColumnMetadata;

        let tables = vec![TableMetadata {
            id: 1,
            database_id: 1,
            schema_name: "public".to_string(),
            table_name: "users".to_string(),
            table_type: "TABLE".to_string(),
            cached_at: Utc::now(),
            columns: vec![
                ColumnMetadata {
                    id: 1,
                    table_id: 1,
                    column_name: "id".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: false,
                    column_default: None,
                    ordinal_position: 1,
                    is_primary_key: true,
                },
                ColumnMetadata {
                    id: 2,
                    table_id: 1,
                    column_name: "username".to_string(),
                    data_type: "varchar".to_string(),
                    is_nullable: false,
                    column_default: None,
                    ordinal_position: 2,
                    is_primary_key: false,
                },
            ],
        }];

        let prompt = LlmService::build_prompt_with_schema("Get all users", &tables);
        assert!(prompt.contains("Table: public.users"));
        assert!(prompt.contains("id (integer, NOT NULL PRIMARY KEY)"));
        assert!(prompt.contains("username (varchar, NOT NULL)"));
        assert!(prompt.contains("Get all users"));
    }
}
