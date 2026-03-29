use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("validation failed")]
    Validation(#[from] ValidationErrors),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("internal error")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, details) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string(), None),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string(), None),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string(), None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone(), None),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), None),
            AppError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                "validation failed".into(),
                Some(serde_json::to_value(e).unwrap_or_default()),
            ),
            AppError::Database(e) => {
                tracing::error!(?e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".into(),
                    None,
                )
            }
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), None),
            AppError::Internal(msg) => {
                tracing::error!(%msg, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".into(),
                    None,
                )
            }
        };

        let body = ErrorBody {
            error: message,
            details,
        };

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
