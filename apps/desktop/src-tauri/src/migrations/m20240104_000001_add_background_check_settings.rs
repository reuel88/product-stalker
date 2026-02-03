use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add background_check_enabled column
        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .add_column(
                        ColumnDef::new(Settings::BackgroundCheckEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Add background_check_interval_minutes column
        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .add_column(
                        ColumnDef::new(Settings::BackgroundCheckIntervalMinutes)
                            .integer()
                            .not_null()
                            .default(60), // Default to 1 hour
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite 3.35.0+ supports DROP COLUMN natively for columns without indexes,
        // constraints, or references. This project bundles SQLite 3.46.0+ via sqlx-sqlite.
        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .drop_column(Settings::BackgroundCheckEnabled)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .drop_column(Settings::BackgroundCheckIntervalMinutes)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    BackgroundCheckEnabled,
    BackgroundCheckIntervalMinutes,
}
