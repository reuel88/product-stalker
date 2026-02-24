use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExchangeRate::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ExchangeRate::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ExchangeRate::FromCurrency).text().not_null())
                    .col(ColumnDef::new(ExchangeRate::ToCurrency).text().not_null())
                    .col(ColumnDef::new(ExchangeRate::Rate).double().not_null())
                    .col(
                        ColumnDef::new(ExchangeRate::Source)
                            .text()
                            .not_null()
                            .default("api"),
                    )
                    .col(
                        ColumnDef::new(ExchangeRate::FetchedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_exchange_rates_currency_pair")
                    .table(ExchangeRate::Table)
                    .col(ExchangeRate::FromCurrency)
                    .col(ExchangeRate::ToCurrency)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExchangeRate::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum ExchangeRate {
    #[iden = "exchange_rates"]
    Table,
    Id,
    FromCurrency,
    ToCurrency,
    Rate,
    Source,
    FetchedAt,
}
