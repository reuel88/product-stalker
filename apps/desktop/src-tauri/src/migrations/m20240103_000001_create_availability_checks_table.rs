use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AvailabilityChecks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AvailabilityChecks::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AvailabilityChecks::ProductId)
                            .string()
                            .not_null(),
                    )
                    // Status: in_stock, out_of_stock, back_order, unknown
                    .col(
                        ColumnDef::new(AvailabilityChecks::Status)
                            .string()
                            .not_null(),
                    )
                    // Original schema.org value (e.g., "http://schema.org/InStock")
                    .col(
                        ColumnDef::new(AvailabilityChecks::RawAvailability)
                            .text()
                            .null(),
                    )
                    // Error message if check failed
                    .col(
                        ColumnDef::new(AvailabilityChecks::ErrorMessage)
                            .text()
                            .null(),
                    )
                    // When the check was performed
                    .col(
                        ColumnDef::new(AvailabilityChecks::CheckedAt)
                            .text()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_availability_checks_product")
                            .from(AvailabilityChecks::Table, AvailabilityChecks::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on product_id for filtering by product
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_availability_checks_product_id")
                    .table(AvailabilityChecks::Table)
                    .col(AvailabilityChecks::ProductId)
                    .to_owned(),
            )
            .await?;

        // Index on checked_at for sorting by time
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_availability_checks_checked_at")
                    .table(AvailabilityChecks::Table)
                    .col(AvailabilityChecks::CheckedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_availability_checks_checked_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx_availability_checks_product_id")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AvailabilityChecks::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AvailabilityChecks {
    Table,
    Id,
    ProductId,
    Status,
    RawAvailability,
    ErrorMessage,
    CheckedAt,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
