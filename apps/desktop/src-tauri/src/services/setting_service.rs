use sea_orm::DatabaseConnection;

use crate::entities::setting::Model as SettingModel;
use crate::error::AppError;
use crate::repositories::{SettingRepository, UpdateSettingsParams};

/// Service layer for settings business logic
///
/// Validates inputs and orchestrates repository calls.
pub struct SettingService;

impl SettingService {
    /// Get current settings (creates defaults if first run)
    pub async fn get(conn: &DatabaseConnection) -> Result<SettingModel, AppError> {
        SettingRepository::get_or_create(conn).await
    }

    /// Update settings with validation
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateSettingsParams,
    ) -> Result<SettingModel, AppError> {
        // Validate theme if provided
        if let Some(ref theme) = params.theme {
            Self::validate_theme(theme)?;
        }

        // Validate log level if provided
        if let Some(ref level) = params.log_level {
            Self::validate_log_level(level)?;
        }

        // Get existing settings
        let settings = SettingRepository::get_or_create(conn).await?;

        // Update settings
        SettingRepository::update(conn, settings, params).await
    }

    fn validate_theme(theme: &str) -> Result<(), AppError> {
        match theme {
            "light" | "dark" | "system" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid theme: {}. Must be 'light', 'dark', or 'system'",
                theme
            ))),
        }
    }

    fn validate_log_level(level: &str) -> Result<(), AppError> {
        match level {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid log level: {}. Must be 'error', 'warn', 'info', 'debug', or 'trace'",
                level
            ))),
        }
    }
}
