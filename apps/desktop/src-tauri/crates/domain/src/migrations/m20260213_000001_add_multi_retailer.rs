use sea_orm::{DatabaseTransaction, Statement, TransactionTrait};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use a transaction to pin all operations to a single database connection.
        // SeaORM's DatabaseConnection for SQLite is a pool, and without a transaction,
        // each execute_unprepared call may use a different connection, causing
        // non-deterministic behavior during table rebuilds (DROP TABLE + RENAME).
        let txn: DatabaseTransaction = db.begin().await?;

        // 1. Preserve availability_checks data and remove its FK to products.
        // This prevents CASCADE deletion when we rebuild the products table.
        txn.execute_unprepared(
            "CREATE TABLE availability_checks_backup AS SELECT * FROM availability_checks",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE availability_checks")
            .await?;

        // 2. Make products.url nullable (SQLite table rebuild).
        // Safe: no foreign key references to products exist now.
        txn.execute_unprepared(
            r#"
            CREATE TABLE products_new (
                id TEXT NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NULL,
                description TEXT NULL,
                notes TEXT NULL,
                currency TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO products_new SELECT id, name, url, description, notes, currency, created_at, updated_at FROM products",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE products").await?;
        txn.execute_unprepared("ALTER TABLE products_new RENAME TO products")
            .await?;
        txn.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_products_name ON products (name)")
            .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at)",
        )
        .await?;

        // 3. Create retailers table (before availability_checks rebuild, which references product_retailers)
        txn.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS retailers (
                id TEXT NOT NULL PRIMARY KEY,
                domain TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .await?;

        // 4. Create product_retailers table
        txn.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS product_retailers (
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

        // 5. Recreate availability_checks with FKs to products and product_retailers + new column
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
            "INSERT INTO availability_checks (id, product_id, status, raw_availability, error_message, checked_at, price_minor_units, price_currency, raw_price) SELECT id, product_id, status, raw_availability, error_message, checked_at, price_minor_units, price_currency, raw_price FROM availability_checks_backup",
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

        // 6. Data migration: populate retailers and product_retailers from existing products
        txn.execute_unprepared(
            r#"
            INSERT OR IGNORE INTO retailers (id, domain, name, created_at)
            SELECT
                lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
                CASE
                    WHEN url LIKE 'http://%' THEN
                        substr(
                            substr(url, 8),
                            1,
                            CASE WHEN instr(substr(url, 8), '/') > 0
                                THEN instr(substr(url, 8), '/') - 1
                                ELSE length(substr(url, 8))
                            END
                        )
                    WHEN url LIKE 'https://%' THEN
                        substr(
                            substr(url, 9),
                            1,
                            CASE WHEN instr(substr(url, 9), '/') > 0
                                THEN instr(substr(url, 9), '/') - 1
                                ELSE length(substr(url, 9))
                            END
                        )
                    ELSE url
                END,
                CASE
                    WHEN url LIKE 'http://%' THEN
                        substr(
                            substr(url, 8),
                            1,
                            CASE WHEN instr(substr(url, 8), '/') > 0
                                THEN instr(substr(url, 8), '/') - 1
                                ELSE length(substr(url, 8))
                            END
                        )
                    WHEN url LIKE 'https://%' THEN
                        substr(
                            substr(url, 9),
                            1,
                            CASE WHEN instr(substr(url, 9), '/') > 0
                                THEN instr(substr(url, 9), '/') - 1
                                ELSE length(substr(url, 9))
                            END
                        )
                    ELSE url
                END,
                datetime('now')
            FROM products
            WHERE url IS NOT NULL AND url != ''
            "#,
        )
        .await?;

        txn.execute_unprepared(
            r#"
            INSERT INTO product_retailers (id, product_id, retailer_id, url, label, created_at)
            SELECT
                lower(hex(randomblob(4)) || '-' || hex(randomblob(2)) || '-4' || substr(hex(randomblob(2)),2) || '-' || substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)),2) || '-' || hex(randomblob(6))),
                p.id,
                r.id,
                p.url,
                NULL,
                datetime('now')
            FROM products p
            JOIN retailers r ON r.domain = CASE
                WHEN p.url LIKE 'http://%' THEN
                    substr(
                        substr(p.url, 8),
                        1,
                        CASE WHEN instr(substr(p.url, 8), '/') > 0
                            THEN instr(substr(p.url, 8), '/') - 1
                            ELSE length(substr(p.url, 8))
                        END
                    )
                WHEN p.url LIKE 'https://%' THEN
                    substr(
                        substr(p.url, 9),
                        1,
                        CASE WHEN instr(substr(p.url, 9), '/') > 0
                            THEN instr(substr(p.url, 9), '/') - 1
                            ELSE length(substr(p.url, 9))
                        END
                    )
                ELSE p.url
            END
            WHERE p.url IS NOT NULL AND p.url != ''
            "#,
        )
        .await?;

        // 7. Backfill product_retailer_id in availability_checks
        txn.execute_unprepared(
            r#"
            UPDATE availability_checks
            SET product_retailer_id = (
                SELECT pr.id FROM product_retailers pr
                WHERE pr.product_id = availability_checks.product_id
                LIMIT 1
            )
            "#,
        )
        .await?;

        txn.commit().await?;

        // Verify foreign key integrity after commit
        let violations = db
            .query_all(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA foreign_key_check".to_owned(),
            ))
            .await?;

        if !violations.is_empty() {
            return Err(DbErr::Custom(format!(
                "Foreign key constraint violations detected after migration: {} violations",
                violations.len()
            )));
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use a transaction to pin all operations to a single connection
        let txn: DatabaseTransaction = db.begin().await?;

        // 1. Drop product_retailers and retailers tables first
        txn.execute_unprepared("DROP TABLE IF EXISTS product_retailers")
            .await?;
        txn.execute_unprepared("DROP TABLE IF EXISTS retailers")
            .await?;

        // 2. Rebuild availability_checks without product_retailer_id column
        txn.execute_unprepared(
            r#"
            CREATE TABLE availability_checks_new (
                id TEXT NOT NULL PRIMARY KEY,
                product_id TEXT NOT NULL,
                status TEXT NOT NULL,
                raw_availability TEXT NULL,
                error_message TEXT NULL,
                checked_at TEXT NOT NULL,
                price_minor_units INTEGER NULL,
                price_currency TEXT NULL,
                raw_price TEXT NULL,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO availability_checks_new SELECT id, product_id, status, raw_availability, error_message, checked_at, price_minor_units, price_currency, raw_price FROM availability_checks",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE availability_checks")
            .await?;
        txn.execute_unprepared("ALTER TABLE availability_checks_new RENAME TO availability_checks")
            .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_product_id ON availability_checks (product_id)",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_checked_at ON availability_checks (checked_at)",
        )
        .await?;

        // 3. Make products.url NOT NULL again (table rebuild)
        // Safe: availability_checks FK was removed above, no FK references to products.
        txn.execute_unprepared(
            r#"
            CREATE TABLE products_new (
                id TEXT NOT NULL PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL DEFAULT '',
                description TEXT NULL,
                notes TEXT NULL,
                currency TEXT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO products_new SELECT id, name, COALESCE(url, ''), description, notes, currency, created_at, updated_at FROM products",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE products").await?;
        txn.execute_unprepared("ALTER TABLE products_new RENAME TO products")
            .await?;
        txn.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_products_name ON products (name)")
            .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at)",
        )
        .await?;

        // 4. Recreate availability_checks with FK to rebuilt products
        txn.execute_unprepared(
            r#"
            CREATE TABLE availability_checks_temp (
                id TEXT NOT NULL PRIMARY KEY,
                product_id TEXT NOT NULL,
                status TEXT NOT NULL,
                raw_availability TEXT NULL,
                error_message TEXT NULL,
                checked_at TEXT NOT NULL,
                price_minor_units INTEGER NULL,
                price_currency TEXT NULL,
                raw_price TEXT NULL,
                FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
            )
            "#,
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO availability_checks_temp SELECT * FROM availability_checks",
        )
        .await?;
        txn.execute_unprepared("DROP TABLE availability_checks")
            .await?;
        txn.execute_unprepared(
            "ALTER TABLE availability_checks_temp RENAME TO availability_checks",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_product_id ON availability_checks (product_id)",
        )
        .await?;
        txn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_checked_at ON availability_checks (checked_at)",
        )
        .await?;

        txn.commit().await?;

        // Verify foreign key integrity after commit
        let violations = db
            .query_all(Statement::from_string(
                db.get_database_backend(),
                "PRAGMA foreign_key_check".to_owned(),
            ))
            .await?;

        if !violations.is_empty() {
            return Err(DbErr::Custom(format!(
                "Foreign key constraint violations detected after rollback: {} violations",
                violations.len()
            )));
        }

        Ok(())
    }
}
