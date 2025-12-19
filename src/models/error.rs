use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// Application error types
#[derive(Debug)]
pub enum AppError {
    /// Database operation failed
    DatabaseError(String),
    
    /// Validation error (invalid input, unsafe SQL, etc.)
    ValidationError(String),
    
    /// LLM service error (OpenAI API failure, timeout, etc.)
    LlmError(String),
    
    /// Resource not found
    NotFound(String),
    
    /// Configuration error
    ConfigError(String),
    
    /// Internal server error
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::LlmError(msg) => write!(f, "LLM service error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Error response payload (camelCase for JSON responses)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::DatabaseError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", msg)
            }
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg)
            }
            AppError::LlmError(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, "LLM_SERVICE_ERROR", msg)
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg)
            }
            AppError::ConfigError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", msg)
            }
            AppError::InternalError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg)
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

/// Convert from anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError(err.to_string())
    }
}

/// Convert from sqlx::Error to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}
