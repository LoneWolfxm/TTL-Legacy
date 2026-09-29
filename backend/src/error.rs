use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// Unified JSON error body returned by all handlers.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    #[serde(skip)]
    status: StatusCode,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details: None,
            status,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        (status, Json(self)).into_response()
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("not found")]
    NotFound,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("2FA required")]
    TwoFactorRequired,
    #[error("2FA not enabled")]
    TwoFactorNotEnabled,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("email delivery failed: {0}")]
    EmailDelivery(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            AppError::InvalidInput(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_input"),
            AppError::Db(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            AppError::TwoFactorRequired => (StatusCode::UNAUTHORIZED, "two_factor_required"),
            AppError::TwoFactorNotEnabled => (StatusCode::BAD_REQUEST, "two_factor_not_enabled"),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::EmailDelivery(_) => (StatusCode::BAD_GATEWAY, "email_delivery_failed"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        ApiError::new(status, code, self.to_string()).into_response()
    }
}

/// Convert a poisoned lock error into a typed internal error so handlers can
/// propagate it with `?` instead of panicking via `unwrap()`/`expect()`.
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        AppError::Internal("lock poisoned".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisoned_lock_maps_to_internal_error() {
        let err: AppError = std::sync::PoisonError::new(()).into();
        assert!(matches!(err, AppError::Internal(_)));
    }

    #[test]
    fn internal_error_maps_to_500() {
        let response = AppError::Internal("boom".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn invalid_input_maps_to_422() {
        let response = AppError::InvalidInput("bad".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn not_found_maps_to_404() {
        let response = AppError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
