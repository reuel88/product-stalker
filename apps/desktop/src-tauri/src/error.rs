use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(err: AppError) -> Self {
        let (code, message) = match &err {
            AppError::Database(db_err) => ("DATABASE_ERROR", db_err.to_string()),
            AppError::NotFound(msg) => ("NOT_FOUND", msg.clone()),
            AppError::Validation(msg) => ("VALIDATION_ERROR", msg.clone()),
            AppError::Internal(msg) => ("INTERNAL_ERROR", msg.clone()),
        };

        let response = ErrorResponse {
            error: message,
            code: code.to_string(),
        };

        tauri::ipc::InvokeError::from(
            serde_json::to_string(&response)
                .unwrap_or_else(|_| format!(r#"{{"error":"{}","code":"{}"}}"#, err, code)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error_display() {
        let err = AppError::NotFound("Product 123".to_string());
        assert_eq!(err.to_string(), "Not found: Product 123");
    }

    #[test]
    fn test_validation_error_display() {
        let err = AppError::Validation("Name cannot be empty".to_string());
        assert_eq!(err.to_string(), "Validation error: Name cannot be empty");
    }

    #[test]
    fn test_internal_error_display() {
        let err = AppError::Internal("Something went wrong".to_string());
        assert_eq!(err.to_string(), "Internal error: Something went wrong");
    }
}
