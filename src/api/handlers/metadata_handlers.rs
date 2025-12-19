use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    api::{
        dto::{ColumnMetadataDto, DatabaseMetadataResponse, TableMetadataDto},
        routes::AppState,
    },
    models::error::AppError,
    services::{database_service::DatabaseService, metadata_service::MetadataService},
};

/// GET /api/v1/databases/{db_name}/metadata - Get database metadata
pub async fn get_metadata(
    State(state): State<AppState>,
    Path(db_name): Path<String>,
) -> Result<Json<DatabaseMetadataResponse>, AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());
    let metadata_service = MetadataService::new(state.repository.pool().clone());

    // Get database connection
    let db_connection = db_service.get_database(&db_name).await?;

    // Fetch metadata (cached or from PostgreSQL)
    let tables = metadata_service
        .get_or_fetch_metadata(db_connection.id, &db_connection.connection_url)
        .await?;

    // Convert to DTOs with camelCase
    let table_dtos: Vec<TableMetadataDto> = tables
        .iter()
        .map(|table| {
            let column_dtos: Vec<ColumnMetadataDto> = table
                .columns
                .iter()
                .map(|col| ColumnMetadataDto {
                    column_name: col.column_name.clone(),
                    data_type: col.data_type.clone(),
                    is_nullable: col.is_nullable,
                    column_default: col.column_default.clone(),
                    is_primary_key: col.is_primary_key,
                })
                .collect();

            TableMetadataDto {
                schema_name: table.schema_name.clone(),
                table_name: table.table_name.clone(),
                table_type: table.table_type.clone(),
                columns: column_dtos,
            }
        })
        .collect();

    Ok(Json(DatabaseMetadataResponse { tables: table_dtos }))
}
