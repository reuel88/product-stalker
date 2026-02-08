use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    // SQLite: Use TEXT for UUIDs (stored as strings)
                    // SeaORM will handle serialization/deserialization
                    .col(
                        ColumnDef::new(Products::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Products::Name).string().not_null())
                    .col(ColumnDef::new(Products::Url).string().not_null())
                    .col(ColumnDef::new(Products::Description).text().null())
                    .col(ColumnDef::new(Products::Notes).text().null())
                    // SQLite: timestamps stored as TEXT in ISO 8601 format
                    .col(ColumnDef::new(Products::CreatedAt).text().not_null())
                    .col(ColumnDef::new(Products::UpdatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create indexes for common queries
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_products_name")
                    .table(Products::Table)
                    .col(Products::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_products_created_at")
                    .table(Products::Table)
                    .col(Products::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_products_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_products_name")
                    .to_owned(),
            )
            .await?;

        // Drop table
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Name,
    Url,
    Description,
    Notes,
    CreatedAt,
    UpdatedAt,
}
