use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AppSettings::ScopeType)
                            .string()
                            .not_null()
                            .default("global"),
                    )
                    .col(ColumnDef::new(AppSettings::ScopeId).string().null())
                    .col(ColumnDef::new(AppSettings::Key).string().not_null())
                    .col(ColumnDef::new(AppSettings::Value).text().not_null())
                    .col(
                        ColumnDef::new(AppSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index on (scope_type, scope_id, key)
        manager
            .create_index(
                Index::create()
                    .name("idx_app_settings_scope_key")
                    .table(AppSettings::Table)
                    .col(AppSettings::ScopeType)
                    .col(AppSettings::ScopeId)
                    .col(AppSettings::Key)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index on scope_type for efficient filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_app_settings_scope_type")
                    .table(AppSettings::Table)
                    .col(AppSettings::ScopeType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppSettings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum AppSettings {
    Table,
    Id,
    ScopeType,
    ScopeId,
    Key,
    Value,
    UpdatedAt,
}
