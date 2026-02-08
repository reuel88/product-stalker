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

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Scraping error: {0}")]
    Scraping(String),

    #[error("Bot protection: {0}")]
    BotProtection(String),

    #[error("HTTP {status} for URL: {url}")]
    HttpStatus { status: u16, url: String },
}

impl AppError {
    /// Get the error code for this error type
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Http(_) => "HTTP_ERROR",
            AppError::Scraping(_) => "SCRAPING_ERROR",
            AppError::BotProtection(_) => "BOT_PROTECTION",
            AppError::HttpStatus { .. } => "HTTP_STATUS_ERROR",
        }
    }
}

/// JSON error response for frontend
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorResponse {
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

    /// Create an error response from an AppError.
    ///
    /// Extracts the inner message without the variant prefix (e.g., "Database error: ").
    /// The frontend uses the `code` field for error categorization, so the `error`
    /// field contains only the human-readable message.
    pub fn from_app_error(err: &AppError) -> Self {
        let message = match err {
            AppError::Database(db_err) => db_err.to_string(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Internal(msg) => msg.clone(),
            AppError::Http(http_err) => http_err.to_string(),
            AppError::Scraping(msg) => msg.clone(),
            AppError::BotProtection(msg) => msg.clone(),
            AppError::HttpStatus { status, url } => format!("HTTP {} for URL: {}", status, url),
        };

        Self::new(message, err.code())
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

    // From trait tests

    #[test]
    fn test_from_db_err() {
        let db_err = sea_orm::DbErr::Custom("test error".to_string());
        let app_err: AppError = db_err.into();
        assert!(matches!(app_err, AppError::Database(_)));
    }

    #[test]
    fn test_from_db_err_record_not_found() {
        let db_err = sea_orm::DbErr::RecordNotFound("entity".to_string());
        let app_err: AppError = db_err.into();
        assert!(matches!(app_err, AppError::Database(_)));
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
            AppError::Scraping("test".to_string()),
            AppError::BotProtection("test".to_string()),
            AppError::HttpStatus {
                status: 404,
                url: "test".to_string(),
            },
        ];

        let codes: Vec<&str> = errors.iter().map(|e| e.code()).collect();
        let unique_codes: std::collections::HashSet<&str> = codes.iter().cloned().collect();

        assert_eq!(
            codes.len(),
            unique_codes.len(),
            "All error codes should be unique"
        );
    }

    // Http and Scraping error tests

    #[test]
    fn test_scraping_error_display() {
        let err = AppError::Scraping("Failed to parse HTML".to_string());
        assert_eq!(err.to_string(), "Scraping error: Failed to parse HTML");
    }

    #[test]
    fn test_scraping_code() {
        let err = AppError::Scraping("test".to_string());
        assert_eq!(err.code(), "SCRAPING_ERROR");
    }

    #[test]
    fn test_error_response_from_scraping() {
        let err = AppError::Scraping("No JSON-LD found".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "No JSON-LD found");
        assert_eq!(response.code, "SCRAPING_ERROR");
    }

    // Bot protection error tests

    #[test]
    fn test_bot_protection_error_display() {
        let err = AppError::BotProtection("Cloudflare challenge detected".to_string());
        assert_eq!(
            err.to_string(),
            "Bot protection: Cloudflare challenge detected"
        );
    }

    #[test]
    fn test_bot_protection_code() {
        let err = AppError::BotProtection("test".to_string());
        assert_eq!(err.code(), "BOT_PROTECTION");
    }

    #[test]
    fn test_error_response_from_bot_protection() {
        let err = AppError::BotProtection("Chrome required".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "Chrome required");
        assert_eq!(response.code, "BOT_PROTECTION");
    }

    // HTTP status error tests

    #[test]
    fn test_http_status_error_display() {
        let err = AppError::HttpStatus {
            status: 403,
            url: "https://example.com/product".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "HTTP 403 for URL: https://example.com/product"
        );
    }

    #[test]
    fn test_http_status_error_display_503() {
        let err = AppError::HttpStatus {
            status: 503,
            url: "https://example.com/api".to_string(),
        };
        assert_eq!(err.to_string(), "HTTP 503 for URL: https://example.com/api");
    }

    #[test]
    fn test_http_status_code() {
        let err = AppError::HttpStatus {
            status: 404,
            url: "test".to_string(),
        };
        assert_eq!(err.code(), "HTTP_STATUS_ERROR");
    }

    #[test]
    fn test_error_response_from_http_status() {
        let err = AppError::HttpStatus {
            status: 403,
            url: "https://example.com/blocked".to_string(),
        };
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(
            response.error,
            "HTTP 403 for URL: https://example.com/blocked"
        );
        assert_eq!(response.code, "HTTP_STATUS_ERROR");
    }

    #[test]
    fn test_http_status_url_with_numbers_not_confused() {
        // This test verifies that a URL containing "403" doesn't cause issues
        // The status should be checked via the discrete field, not string matching
        let err = AppError::HttpStatus {
            status: 404, // Different status than what's in the URL
            url: "https://example.com/product-403-special".to_string(),
        };
        // The error message will contain "403" in the URL but the status is 404
        assert_eq!(
            err.to_string(),
            "HTTP 404 for URL: https://example.com/product-403-special"
        );
        // The actual status field is what matters for bot protection detection
        if let AppError::HttpStatus { status, .. } = err {
            assert_eq!(status, 404);
            assert!(status != 403); // This is the key check - status is deterministic
        }
    }
}
