use serde::Serialize;
use thiserror::Error;

/// Application error types
///
/// Provides structured error handling with error codes for frontend consumption.
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

impl AppError {
    /// Get the error code for this error type
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// Check if this is a database error
    pub fn is_database_error(&self) -> bool {
        matches!(self, AppError::Database(_))
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, AppError::NotFound(_))
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, AppError::Validation(_))
    }

    /// Check if this is an internal error
    pub fn is_internal_error(&self) -> bool {
        matches!(self, AppError::Internal(_))
    }
}

/// JSON error response for frontend
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }

    /// Create an error response from an AppError
    pub fn from_app_error(err: &AppError) -> Self {
        let message = match err {
            AppError::Database(db_err) => db_err.to_string(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Internal(msg) => msg.clone(),
        };

        Self::new(message, err.code())
    }
}

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(err: AppError) -> Self {
        let response = ErrorResponse::from_app_error(&err);

        tauri::ipc::InvokeError::from(serde_json::to_string(&response).unwrap_or_else(|_| {
            serde_json::json!({
                "error": err.to_string(),
                "code": err.code()
            })
            .to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Display trait tests

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

    #[test]
    fn test_database_error_display() {
        let db_err = sea_orm::DbErr::Custom("Connection failed".to_string());
        let err = AppError::Database(db_err);
        assert!(err.to_string().contains("Database error"));
        assert!(err.to_string().contains("Connection failed"));
    }

    // Error code tests

    #[test]
    fn test_not_found_code() {
        let err = AppError::NotFound("test".to_string());
        assert_eq!(err.code(), "NOT_FOUND");
    }

    #[test]
    fn test_validation_code() {
        let err = AppError::Validation("test".to_string());
        assert_eq!(err.code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_internal_code() {
        let err = AppError::Internal("test".to_string());
        assert_eq!(err.code(), "INTERNAL_ERROR");
    }

    #[test]
    fn test_database_code() {
        let db_err = sea_orm::DbErr::Custom("test".to_string());
        let err = AppError::Database(db_err);
        assert_eq!(err.code(), "DATABASE_ERROR");
    }

    // Type check tests

    #[test]
    fn test_is_not_found() {
        let err = AppError::NotFound("test".to_string());
        assert!(err.is_not_found());
        assert!(!err.is_validation_error());
        assert!(!err.is_internal_error());
        assert!(!err.is_database_error());
    }

    #[test]
    fn test_is_validation_error() {
        let err = AppError::Validation("test".to_string());
        assert!(err.is_validation_error());
        assert!(!err.is_not_found());
        assert!(!err.is_internal_error());
        assert!(!err.is_database_error());
    }

    #[test]
    fn test_is_internal_error() {
        let err = AppError::Internal("test".to_string());
        assert!(err.is_internal_error());
        assert!(!err.is_not_found());
        assert!(!err.is_validation_error());
        assert!(!err.is_database_error());
    }

    #[test]
    fn test_is_database_error() {
        let db_err = sea_orm::DbErr::Custom("test".to_string());
        let err = AppError::Database(db_err);
        assert!(err.is_database_error());
        assert!(!err.is_not_found());
        assert!(!err.is_validation_error());
        assert!(!err.is_internal_error());
    }

    // Debug trait tests

    #[test]
    fn test_debug_not_found() {
        let err = AppError::NotFound("item".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("item"));
    }

    #[test]
    fn test_debug_validation() {
        let err = AppError::Validation("invalid".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Validation"));
        assert!(debug_str.contains("invalid"));
    }

    #[test]
    fn test_debug_internal() {
        let err = AppError::Internal("failure".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Internal"));
        assert!(debug_str.contains("failure"));
    }

    #[test]
    fn test_debug_database() {
        let db_err = sea_orm::DbErr::Custom("db failure".to_string());
        let err = AppError::Database(db_err);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Database"));
    }

    // From trait tests

    #[test]
    fn test_from_db_err() {
        let db_err = sea_orm::DbErr::Custom("test error".to_string());
        let app_err: AppError = db_err.into();
        assert!(app_err.is_database_error());
    }

    #[test]
    fn test_from_db_err_record_not_found() {
        let db_err = sea_orm::DbErr::RecordNotFound("entity".to_string());
        let app_err: AppError = db_err.into();
        assert!(app_err.is_database_error());
        assert!(app_err.to_string().contains("entity"));
    }

    // ErrorResponse tests

    #[test]
    fn test_error_response_new() {
        let response = ErrorResponse::new("Something failed", "SOME_ERROR");
        assert_eq!(response.error, "Something failed");
        assert_eq!(response.code, "SOME_ERROR");
    }

    #[test]
    fn test_error_response_new_with_string() {
        let response = ErrorResponse::new(String::from("error"), String::from("CODE"));
        assert_eq!(response.error, "error");
        assert_eq!(response.code, "CODE");
    }

    #[test]
    fn test_error_response_from_not_found() {
        let err = AppError::NotFound("User 42".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "User 42");
        assert_eq!(response.code, "NOT_FOUND");
    }

    #[test]
    fn test_error_response_from_validation() {
        let err = AppError::Validation("Email is invalid".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "Email is invalid");
        assert_eq!(response.code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_error_response_from_internal() {
        let err = AppError::Internal("Unexpected state".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "Unexpected state");
        assert_eq!(response.code, "INTERNAL_ERROR");
    }

    #[test]
    fn test_error_response_from_database() {
        let db_err = sea_orm::DbErr::Custom("Connection timeout".to_string());
        let err = AppError::Database(db_err);
        let response = ErrorResponse::from_app_error(&err);
        assert!(response.error.contains("Connection timeout"));
        assert_eq!(response.code, "DATABASE_ERROR");
    }

    #[test]
    fn test_error_response_clone() {
        let response = ErrorResponse::new("error", "CODE");
        let cloned = response.clone();
        assert_eq!(response, cloned);
    }

    #[test]
    fn test_error_response_debug() {
        let response = ErrorResponse::new("test error", "TEST_CODE");
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("ErrorResponse"));
        assert!(debug_str.contains("test error"));
        assert!(debug_str.contains("TEST_CODE"));
    }

    #[test]
    fn test_error_response_partial_eq() {
        let response1 = ErrorResponse::new("error", "CODE");
        let response2 = ErrorResponse::new("error", "CODE");
        let response3 = ErrorResponse::new("different", "CODE");

        assert_eq!(response1, response2);
        assert_ne!(response1, response3);
    }

    // Serialization tests

    #[test]
    fn test_error_response_serialize() {
        let response = ErrorResponse::new("Something went wrong", "INTERNAL_ERROR");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""error":"Something went wrong""#));
        assert!(json.contains(r#""code":"INTERNAL_ERROR""#));
    }

    #[test]
    fn test_error_response_serialize_with_special_chars() {
        let response = ErrorResponse::new("Error: \"quoted\" & <special>", "CODE");
        let json = serde_json::to_string(&response).unwrap();

        // Verify it's valid JSON by parsing it back
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed["error"].as_str().unwrap(),
            "Error: \"quoted\" & <special>"
        );
    }

    #[test]
    fn test_error_response_serialize_with_unicode() {
        let response = ErrorResponse::new("错误消息 🚨", "UNICODE_ERROR");
        let json = serde_json::to_string(&response).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["error"].as_str().unwrap().contains("🚨"));
    }

    // Edge case tests

    #[test]
    fn test_empty_error_message() {
        let err = AppError::Internal(String::new());
        assert_eq!(err.to_string(), "Internal error: ");
        assert_eq!(err.code(), "INTERNAL_ERROR");
    }

    #[test]
    fn test_long_error_message() {
        let long_msg = "a".repeat(1000);
        let err = AppError::Validation(long_msg.clone());
        assert!(err.to_string().contains(&long_msg));
    }

    #[test]
    fn test_error_message_with_newlines() {
        let err = AppError::Internal("Line 1\nLine 2\nLine 3".to_string());
        assert!(err.to_string().contains("\n"));
    }

    #[test]
    fn test_all_error_types_have_unique_codes() {
        let errors = [
            AppError::Database(sea_orm::DbErr::Custom("test".to_string())),
            AppError::NotFound("test".to_string()),
            AppError::Validation("test".to_string()),
            AppError::Internal("test".to_string()),
        ];

        let codes: Vec<&str> = errors.iter().map(|e| e.code()).collect();
        let unique_codes: std::collections::HashSet<&str> = codes.iter().cloned().collect();

        assert_eq!(
            codes.len(),
            unique_codes.len(),
            "All error codes should be unique"
        );
    }
}
