//! Tauri-specific error conversion.
//!
//! This module provides the bridge between the workspace's AppError
//! and Tauri's InvokeError for IPC responses.
//!
//! Due to Rust's orphan rules, we can't implement `From<AppError> for InvokeError`
//! directly since both types are external. Instead, we use a newtype wrapper
//! `CommandError` that wraps `AppError` and implements the conversion.

use product_stalker_core::error::ErrorResponse;
use product_stalker_core::AppError;
use std::ops::Deref;

/// Wrapper around AppError for Tauri command returns.
///
/// This newtype allows us to implement `From<CommandError> for InvokeError`
/// without violating Rust's orphan rules.
#[derive(Debug)]
pub struct CommandError(pub AppError);

impl Deref for CommandError {
    type Target = AppError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        CommandError(err)
    }
}

impl From<sea_orm::DbErr> for CommandError {
    fn from(err: sea_orm::DbErr) -> Self {
        CommandError(AppError::Database(err))
    }
}

impl From<CommandError> for tauri::ipc::InvokeError {
    fn from(err: CommandError) -> Self {
        let response = ErrorResponse::from_app_error(&err.0);

        tauri::ipc::InvokeError::from(serde_json::to_string(&response).unwrap_or_else(|_| {
            serde_json::json!({
                "error": err.0.to_string(),
                "code": err.0.code()
            })
            .to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_error_from_app_error() {
        let app_err = AppError::NotFound("test".to_string());
        let cmd_err: CommandError = app_err.into();
        assert!(matches!(&cmd_err.0, AppError::NotFound(_)));
    }

    #[test]
    fn test_command_error_deref() {
        let cmd_err = CommandError(AppError::Validation("test".to_string()));
        assert_eq!(cmd_err.code(), "VALIDATION_ERROR");
    }

    #[test]
    fn test_command_error_from_db_err() {
        let db_err = sea_orm::DbErr::Custom("test".to_string());
        let cmd_err: CommandError = db_err.into();
        assert!(matches!(&cmd_err.0, AppError::Database(_)));
    }

    #[test]
    fn test_invoke_error_conversion() {
        let cmd_err = CommandError(AppError::NotFound("item 42".to_string()));
        let invoke_err: tauri::ipc::InvokeError = cmd_err.into();
        let err_string = format!("{:?}", invoke_err);
        assert!(err_string.contains("item 42"));
    }
}
