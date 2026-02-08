use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the old settings table
        manager
            .drop_table(
                Table::drop()
                    .table(OldSettings::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Recreate the old settings table structure
        manager
            .create_table(
                Table::create()
                    .table(OldSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OldSettings::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OldSettings::Theme)
                            .string()
                            .not_null()
                            .default("system"),
                    )
                    .col(
                        ColumnDef::new(OldSettings::ShowInTray)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OldSettings::LaunchAtLogin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(OldSettings::EnableLogging)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OldSettings::LogLevel)
                            .string()
                            .not_null()
                            .default("info"),
                    )
                    .col(
                        ColumnDef::new(OldSettings::EnableNotifications)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OldSettings::SidebarExpanded)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OldSettings::BackgroundCheckEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(OldSettings::BackgroundCheckIntervalMinutes)
                            .integer()
                            .not_null()
                            .default(60),
                    )
                    .col(
                        ColumnDef::new(OldSettings::EnableHeadlessBrowser)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(OldSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum OldSettings {
    #[iden = "settings"]
    Table,
    Id,
    Theme,
    ShowInTray,
    LaunchAtLogin,
    EnableLogging,
    LogLevel,
    EnableNotifications,
    SidebarExpanded,
    BackgroundCheckEnabled,
    BackgroundCheckIntervalMinutes,
    EnableHeadlessBrowser,
    UpdatedAt,
}
