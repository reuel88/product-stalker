use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Single-statement DDL each, no transaction needed per CLAUDE.md
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .add_column(
                        ColumnDef::new(AvailabilityChecks::NormalizedPriceMinorUnits)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .add_column(
                        ColumnDef::new(AvailabilityChecks::NormalizedCurrency)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support DROP COLUMN directly, but SeaORM handles this
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .drop_column(AvailabilityChecks::NormalizedPriceMinorUnits)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .drop_column(AvailabilityChecks::NormalizedCurrency)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum AvailabilityChecks {
    Table,
    NormalizedPriceMinorUnits,
    NormalizedCurrency,
}
