use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Create retailers table
        manager
            .create_table(
                Table::create()
                    .table(Retailers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Retailers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Retailers::Domain)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Retailers::Name).string().not_null())
                    .col(ColumnDef::new(Retailers::CreatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // 2. Create product_retailers table
        manager
            .create_table(
                Table::create()
                    .table(ProductRetailers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductRetailers::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProductRetailers::ProductId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductRetailers::RetailerId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProductRetailers::Url).string().not_null())
                    .col(ColumnDef::new(ProductRetailers::Label).string().null())
                    .col(
                        ColumnDef::new(ProductRetailers::CreatedAt)
                            .text()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_retailers_product")
                            .from(ProductRetailers::Table, ProductRetailers::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_product_retailers_retailer")
                            .from(ProductRetailers::Table, ProductRetailers::RetailerId)
                            .to(Retailers::Table, Retailers::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for product_retailers
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_product_retailers_product_id")
                    .table(ProductRetailers::Table)
                    .col(ProductRetailers::ProductId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_product_retailers_retailer_id")
                    .table(ProductRetailers::Table)
                    .col(ProductRetailers::RetailerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_product_retailers_product_url")
                    .table(ProductRetailers::Table)
                    .col(ProductRetailers::ProductId)
                    .col(ProductRetailers::Url)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 3. Data migration: create retailers and product_retailers from existing products
        db.execute_unprepared(
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

        // Create product_retailers from existing products
        db.execute_unprepared(
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

        // 4. Add product_retailer_id column to availability_checks
        db.execute_unprepared(
            "ALTER TABLE availability_checks ADD COLUMN product_retailer_id TEXT NULL",
        )
        .await?;

        // 5. Backfill product_retailer_id in availability_checks
        db.execute_unprepared(
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

        // 6. Make products.url nullable (SQLite table rebuild)
        db.execute_unprepared(
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

        db.execute_unprepared(
            "INSERT INTO products_new SELECT id, name, url, description, notes, currency, created_at, updated_at FROM products",
        )
        .await?;

        db.execute_unprepared("DROP TABLE products").await?;
        db.execute_unprepared("ALTER TABLE products_new RENAME TO products")
            .await?;

        // Recreate product indexes
        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_products_name ON products (name)")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove product_retailer_id from availability_checks
        // SQLite doesn't support DROP COLUMN before 3.35, use table rebuild
        db.execute_unprepared(
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

        db.execute_unprepared(
            "INSERT INTO availability_checks_new SELECT id, product_id, status, raw_availability, error_message, checked_at, price_minor_units, price_currency, raw_price FROM availability_checks",
        )
        .await?;

        db.execute_unprepared("DROP TABLE availability_checks")
            .await?;
        db.execute_unprepared("ALTER TABLE availability_checks_new RENAME TO availability_checks")
            .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_product_id ON availability_checks (product_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_availability_checks_checked_at ON availability_checks (checked_at)",
        )
        .await?;

        // Make products.url NOT NULL again
        db.execute_unprepared(
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

        db.execute_unprepared(
            "INSERT INTO products_new SELECT id, name, COALESCE(url, ''), description, notes, currency, created_at, updated_at FROM products",
        )
        .await?;

        db.execute_unprepared("DROP TABLE products").await?;
        db.execute_unprepared("ALTER TABLE products_new RENAME TO products")
            .await?;

        db.execute_unprepared("CREATE INDEX IF NOT EXISTS idx_products_name ON products (name)")
            .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_products_created_at ON products (created_at)",
        )
        .await?;

        // Drop product_retailers and retailers tables
        manager
            .drop_table(Table::drop().table(ProductRetailers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Retailers::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Retailers {
    Table,
    Id,
    Domain,
    Name,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ProductRetailers {
    Table,
    Id,
    ProductId,
    RetailerId,
    Url,
    Label,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
}
