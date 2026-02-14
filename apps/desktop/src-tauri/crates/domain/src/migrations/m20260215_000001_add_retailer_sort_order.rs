use sea_orm::{DatabaseTransaction, Statement, TransactionTrait};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Single-statement DDL — no transaction needed for ALTER TABLE ADD COLUMN
        manager
            .alter_table(
                Table::alter()
                    .table(ProductRetailers::Table)
                    .add_column(
                        ColumnDef::new(ProductRetailers::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Backfill: oldest retailer per product gets 0, newest gets N-1
        db.execute_unprepared(
            "UPDATE product_retailers SET sort_order = (SELECT COUNT(*) FROM product_retailers AS pr2 WHERE pr2.product_id = product_retailers.product_id AND pr2.created_at < product_retailers.created_at)",
        )
        .await?;

        // Index for efficient ordering
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_product_retailers_sort_order ON product_retailers (sort_order)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // SQLite cannot DROP COLUMN — table rebuild required.
        // Must use a transaction to pin to a single connection.
        let txn: DatabaseTransaction = db.begin().await?;

        // 1. Backup availability_checks to remove FK to product_retailers (prevents CASCADE)
        txn.execute_unprepared(
            "CREATE TABLE availability_checks_backup AS SELECT * FROM availability_checks",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE availability_checks")
            .await?;

        // 2. Rebuild product_retailers without sort_order
        txn.execute_unprepared(
            r#"
            CREATE TABLE product_retailers_new (
                id TEXT NOT NULL PRIMARY KEY,
                product_id TEXT NOT NULL,
                retailer_id TEXT NOT NULL,
                url TEXT NOT NULL,
                label TEXT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
                FOREIGN KEY (retailer_id) REFERENCES retailers(id) ON DELETE RESTRICT
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO product_retailers_new SELECT id, product_id, retailer_id, url, label, created_at FROM product_retailers",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE product_retailers")
            .await?;
        txn.execute_unprepared("ALTER TABLE product_retailers_new RENAME TO product_retailers")
            .await?;

        // Restore indexes on product_retailers
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_product_retailers_product_id ON product_retailers (product_id)",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_product_retailers_retailer_id ON product_retailers (retailer_id)",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_product_retailers_product_url ON product_retailers (product_id, url)",
        )
        .await?;

        // 3. Recreate availability_checks with FK to rebuilt product_retailers
        txn.execute_unprepared(
            r#"
            CREATE TABLE availability_checks (
                id TEXT NOT NULL PRIMARY KEY,
                product_id TEXT NOT NULL,
                status TEXT NOT NULL,
                raw_availability TEXT NULL,
                error_message TEXT NULL,
                checked_at TEXT NOT NULL,
                price_minor_units INTEGER NULL,
                price_currency TEXT NULL,
                raw_price TEXT NULL,
                product_retailer_id TEXT NULL,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
                FOREIGN KEY (product_retailer_id) REFERENCES product_retailers(id) ON DELETE SET NULL
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO availability_checks SELECT * FROM availability_checks_backup",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE availability_checks_backup")
            .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_product_id ON availability_checks (product_id)",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_checked_at ON availability_checks (checked_at)",
        )
        .await?;

        // Verify FK integrity before commit
        let violations = txn
            .query_all(Statement::from_string(
                txn.get_database_backend(),
                "PRAGMA foreign_key_check".to_owned(),
            ))
            .await?;

        if !violations.is_empty() {
            return Err(DbErr::Custom(format!(
                "Foreign key constraint violations detected during rollback: {} violations",
                violations.len()
            )));
        }

        txn.commit().await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ProductRetailers {
    Table,
    SortOrder,
}
