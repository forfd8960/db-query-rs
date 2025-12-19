use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    api::{
        dto::{ExecuteQueryRequest, NaturalLanguageQueryRequest, NaturalLanguageQueryResultDto, QueryResultDto},
        routes::AppState,
    },
    models::error::AppError,
    services::{database_service::DatabaseService, llm_service::LlmService, metadata_service::MetadataService, query_service::QueryService},
};

/// POST /api/v1/databases/{db_name}/query - Execute SQL query
pub async fn execute_query(
    State(state): State<AppState>,
    Path(db_name): Path<String>,
    Json(payload): Json<ExecuteQueryRequest>,
) -> Result<Json<QueryResultDto>, AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());

    // Get database connection
    let db_connection = db_service.get_database(&db_name).await?;

    // Execute query
    let result = QueryService::execute_query(&db_connection.connection_url, &payload.sql).await?;

    // Convert to DTO
    let dto = QueryResultDto {
        columns: result.columns,
        rows: result.rows,
        row_count: result.row_count,
        execution_time: result.execution_time,
    };

    Ok(Json(dto))
}

/// POST /api/v1/databases/{db_name}/nl-query - Execute natural language query
pub async fn execute_nl_query(
    State(state): State<AppState>,
    Path(db_name): Path<String>,
    Json(payload): Json<NaturalLanguageQueryRequest>,
) -> Result<Json<NaturalLanguageQueryResultDto>, AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());
    let metadata_service = MetadataService::new(state.repository.pool().clone());

    // Validate NL query is not empty
    if payload.nl_query.trim().is_empty() {
        return Err(AppError::ValidationError(
            "Natural language query cannot be empty".to_string(),
        ));
    }

    // Get database connection
    let db_connection = db_service.get_database(&db_name).await?;

    // Get metadata for LLM context
    let tables = metadata_service
        .get_or_fetch_metadata(db_connection.id, &db_connection.connection_url)
        .await?;

    // Initialize LLM service
    let llm_service = LlmService::new()?;

    // Generate SQL from natural language
    let generated_sql = llm_service
        .generate_sql_from_nl(&payload.nl_query, &tables)
        .await?;

    // Execute the generated SQL (this validates it's SELECT-only and injects LIMIT)
    let result = QueryService::execute_query(&db_connection.connection_url, &generated_sql).await?;

    // Convert to DTO
    let result_dto = QueryResultDto {
        columns: result.columns,
        rows: result.rows,
        row_count: result.row_count,
        execution_time: result.execution_time,
    };

    let response = NaturalLanguageQueryResultDto {
        nl_query: payload.nl_query,
        generated_sql,
        result: result_dto,
    };

    Ok(Json(response))
}
