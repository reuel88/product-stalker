use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VerifiedSession::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(VerifiedSession::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(VerifiedSession::Domain).string().not_null())
                    .col(
                        ColumnDef::new(VerifiedSession::CookiesJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerifiedSession::UserAgent)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerifiedSession::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(VerifiedSession::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add index on domain for fast lookups
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_verified_session_domain")
                    .table(VerifiedSession::Table)
                    .col(VerifiedSession::Domain)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(VerifiedSession::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum VerifiedSession {
    #[iden = "verified_sessions"]
    Table,
    Id,
    Domain,
    CookiesJson,
    UserAgent,
    ExpiresAt,
    CreatedAt,
}
