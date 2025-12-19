use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    api::{
        dto::{AddDatabaseRequest, DatabaseConnectionDto, DatabaseListResponse, UpdateDatabaseRequest},
        routes::AppState,
    },
    models::error::AppError,
    services::database_service::DatabaseService,
};

/// POST /api/v1/databases - Add a new database connection
pub async fn add_database(
    State(state): State<AppState>,
    Json(payload): Json<AddDatabaseRequest>,
) -> Result<(StatusCode, Json<DatabaseConnectionDto>), AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());
    
    let connection = db_service.add_database(&payload.url).await?;
    
    let dto = DatabaseConnectionDto {
        id: connection.id,
        name: connection.name.clone(),
        connection_url: connection.mask_password(),
        created_at: connection.created_at,
        updated_at: connection.updated_at,
    };
    
    Ok((StatusCode::CREATED, Json(dto)))
}

/// GET /api/v1/databases - List all database connections
pub async fn list_databases(
    State(state): State<AppState>,
) -> Result<Json<DatabaseListResponse>, AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());
    
    let connections = db_service.list_databases().await?;
    
    let dtos: Vec<DatabaseConnectionDto> = connections
        .iter()
        .map(|conn| DatabaseConnectionDto {
            id: conn.id,
            name: conn.name.clone(),
            connection_url: conn.mask_password(),
            created_at: conn.created_at,
            updated_at: conn.updated_at,
        })
        .collect();
    
    Ok(Json(DatabaseListResponse { databases: dtos }))
}

/// PUT /api/v1/databases/{db_name} - Update database connection URL
pub async fn update_database(
    State(state): State<AppState>,
    Path(db_name): Path<String>,
    Json(payload): Json<UpdateDatabaseRequest>,
) -> Result<Json<DatabaseConnectionDto>, AppError> {
    let db_service = DatabaseService::new(state.repository.pool().clone());
    
    let connection = db_service.update_database(&db_name, &payload.url).await?;
    
    let dto = DatabaseConnectionDto {
        id: connection.id,
        name: connection.name.clone(),
        connection_url: connection.mask_password(),
        created_at: connection.created_at,
        updated_at: connection.updated_at,
    };
    
    Ok(Json(dto))
}
