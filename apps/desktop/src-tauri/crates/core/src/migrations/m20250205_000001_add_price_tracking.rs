use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add price_cents column (nullable integer for cents)
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .add_column(
                        ColumnDef::new(AvailabilityChecks::PriceCents)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add price_currency column (nullable text for ISO 4217 code)
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .add_column(
                        ColumnDef::new(AvailabilityChecks::PriceCurrency)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add raw_price column (nullable text for original Schema.org value)
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .add_column(ColumnDef::new(AvailabilityChecks::RawPrice).text().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .drop_column(AvailabilityChecks::RawPrice)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .drop_column(AvailabilityChecks::PriceCurrency)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AvailabilityChecks::Table)
                    .drop_column(AvailabilityChecks::PriceCents)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AvailabilityChecks {
    Table,
    PriceCents,
    PriceCurrency,
    RawPrice,
}
