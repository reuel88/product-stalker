use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add enable_headless_browser column
        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .add_column(
                        ColumnDef::new(Settings::EnableHeadlessBrowser)
                            .boolean()
                            .not_null()
                            .default(true), // Enabled by default for best experience
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Settings::Table)
                    .drop_column(Settings::EnableHeadlessBrowser)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    EnableHeadlessBrowser,
}
