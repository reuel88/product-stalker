# Code Patterns Reference

Quick reference for common patterns used in this codebase.

## Database Access Pattern

### ✅ Correct Way (Using Connection Pool)

```rust
#[tauri::command]
pub async fn get_products(db: State<'_, DbState>) -> Result<Vec<ProductResponse>, AppError> {
    // Direct access to connection pool - no blocking
    let products = ProductService::get_all(db.conn()).await?;
    Ok(products.into_iter().map(ProductResponse::from).collect())
}
```

### ❌ Wrong Way (Don't Do This)

```rust
// DON'T use Mutex - blocks async runtime
let conn = db.conn.lock().unwrap();  // ❌ Blocks!
let products = Product::find().all(&*conn).await?;
```

## Layer Responsibilities

### Commands Layer - Only IPC Handling

```rust
#[tauri::command]
pub async fn create_product(
    input: CreateProductInput,
    db: State<'_, DbState>,
) -> Result<ProductResponse, AppError> {
    // 1. Parse UUID if needed
    // 2. Call service layer
    // 3. Convert to DTO
    // NO validation or business logic here!

    let product = ProductService::create(
        db.conn(),
        input.name,
        input.url,
        input.description,
        input.notes,
    ).await?;

    Ok(ProductResponse::from(product))
}
```

### Services Layer - Business Logic

```rust
impl ProductService {
    pub async fn create(
        conn: &DatabaseConnection,
        name: String,
        url: String,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<ProductModel, AppError> {
        // Validate inputs (business rules)
        Self::validate_name(&name)?;
        Self::validate_url(&url)?;

        // Generate business identifiers
        let id = Uuid::new_v4();

        // Call repository
        ProductRepository::create(conn, id, name, url, description, notes).await
    }

    fn validate_name(name: &str) -> Result<(), AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Name cannot be empty".to_string()));
        }
        Ok(())
    }
}
```

### Repositories Layer - Data Access Only

```rust
impl ProductRepository {
    pub async fn create(
        conn: &DatabaseConnection,
        id: Uuid,
        name: String,
        url: String,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<ProductModel, AppError> {
        // Pure data access - no validation or business logic
        let now = chrono::Utc::now();

        let active_model = ProductActiveModel {
            id: Set(id),
            name: Set(name),
            url: Set(url),
            description: Set(description),
            notes: Set(notes),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let product = active_model.insert(conn).await?;
        Ok(product)
    }
}
```

## Error Handling Pattern

### In Services

```rust
// Return AppError, not DbErr
pub async fn get_by_id(
    conn: &DatabaseConnection,
    id: Uuid,
) -> Result<ProductModel, AppError> {
    ProductRepository::find_by_id(conn, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", id)))
}
```

### In Commands

```rust
// Let AppError convert to InvokeError automatically
#[tauri::command]
pub async fn get_product(
    id: String,
    db: State<'_, DbState>
) -> Result<ProductResponse, AppError> {
    // Parse and validate UUID
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))?;

    let product = ProductService::get_by_id(db.conn(), uuid).await?;
    Ok(ProductResponse::from(product))
}
```

## Entity Definition Pattern

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Brief description of the entity
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "table_name")]
pub struct Model {
    /// Primary key (UUID v4)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Required fields
    pub name: String,

    /// Optional fields
    pub description: Option<String>,

    /// Timestamps
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // Define relations here
}

impl ActiveModelBehavior for ActiveModel {}
```

## Migration Pattern

### Simple Table Creation

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TableName::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TableName::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TableName::Name).string().not_null())
                    .col(ColumnDef::new(TableName::Optional).text().null())
                    .to_owned(),
            )
            .await?;

        // Add indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_table_name")
                    .table(TableName::Table)
                    .col(TableName::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_table_name").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(TableName::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum TableName {
    Table,
    Id,
    Name,
    Optional,
}
```

### Adding a Foreign Key

```rust
// In the migration
.col(
    ColumnDef::new(PriceHistory::ProductId)
        .string()
        .not_null()
)
.foreign_key(
    ForeignKey::create()
        .name("fk_price_history_product")
        .from(PriceHistory::Table, PriceHistory::ProductId)
        .to(Products::Table, Products::Id)
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::Cascade)
)
```

### Adding a Column (Safe for Production)

```rust
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Always add new columns as nullable - safe for existing data
    manager
        .alter_table(
            Table::alter()
                .table(Products::Table)
                .add_column(ColumnDef::new(Products::NewColumn).string().null())
                .to_owned(),
        )
        .await
}
```

## DTO Conversion Pattern

```rust
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

impl From<ProductModel> for ProductResponse {
    fn from(model: ProductModel) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

// Usage in command
let products = ProductService::get_all(db.conn()).await?;
Ok(products.into_iter().map(ProductResponse::from).collect())
```

## Query Patterns

### Simple Find All

```rust
pub async fn find_all(conn: &DatabaseConnection) -> Result<Vec<ProductModel>, AppError> {
    let products = Product::find().all(conn).await?;
    Ok(products)
}
```

### Find by ID

```rust
pub async fn find_by_id(
    conn: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<ProductModel>, AppError> {
    let product = Product::find_by_id(id).one(conn).await?;
    Ok(product)
}
```

### Find with Filter

```rust
pub async fn find_by_name_contains(
    conn: &DatabaseConnection,
    search: &str,
) -> Result<Vec<ProductModel>, AppError> {
    let products = Product::find()
        .filter(product::Column::Name.contains(search))
        .all(conn)
        .await?;
    Ok(products)
}
```

### Find with Sorting

```rust
pub async fn find_all_sorted(
    conn: &DatabaseConnection,
) -> Result<Vec<ProductModel>, AppError> {
    let products = Product::find()
        .order_by_desc(product::Column::CreatedAt)
        .all(conn)
        .await?;
    Ok(products)
}
```

### Find with Pagination

```rust
pub async fn find_paginated(
    conn: &DatabaseConnection,
    page: u64,
    per_page: u64,
) -> Result<Vec<ProductModel>, AppError> {
    let products = Product::find()
        .order_by_desc(product::Column::CreatedAt)
        .limit(per_page)
        .offset(page * per_page)
        .all(conn)
        .await?;
    Ok(products)
}
```

### Update Pattern

```rust
pub async fn update(
    conn: &DatabaseConnection,
    model: ProductModel,
    name: Option<String>,
    url: Option<String>,
) -> Result<ProductModel, AppError> {
    let mut active_model: ProductActiveModel = model.into();

    // Only update fields that are provided
    if let Some(name) = name {
        active_model.name = Set(name);
    }
    if let Some(url) = url {
        active_model.url = Set(url);
    }

    // Always update timestamp
    active_model.updated_at = Set(chrono::Utc::now());

    let updated = active_model.update(conn).await?;
    Ok(updated)
}
```

### Delete Pattern

```rust
pub async fn delete_by_id(
    conn: &DatabaseConnection,
    id: Uuid,
) -> Result<u64, AppError> {
    let result = Product::delete_by_id(id).exec(conn).await?;
    Ok(result.rows_affected)
}
```

## Testing Patterns

### Setup In-Memory Database

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DbErr};

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory database");

        // Run migrations
        Migrator::up(&db, None)
            .await
            .expect("Failed to run migrations");

        db
    }

    #[tokio::test]
    async fn test_create_product() {
        let db = setup_db().await;

        let product = ProductService::create(
            &db,
            "Test Product".to_string(),
            "https://example.com".to_string(),
            None,
            None,
        )
        .await
        .expect("Failed to create product");

        assert_eq!(product.name, "Test Product");
    }
}
```

## Common Mistakes to Avoid

### ❌ Don't Use Mutex for Connection

```rust
// Bad - blocks async runtime
let conn = db.conn.lock().unwrap();
```

### ❌ Don't Put Business Logic in Commands

```rust
// Bad - validation in command layer
#[tauri::command]
pub async fn create_product(input: CreateProductInput, db: State<'_, DbState>) -> Result<...> {
    if input.name.is_empty() {  // ❌ Should be in service layer
        return Err(...);
    }
}
```

### ❌ Don't Use Entity Directly in Commands

```rust
// Bad - SeaORM details leaking into commands
#[tauri::command]
pub async fn get_products(db: State<'_, DbState>) -> Result<...> {
    let products = Product::find().all(db.conn()).await?;  // ❌ Should use service
}
```

### ❌ Don't Block Async Runtime

```rust
// Bad - blocking call in async function
pub async fn something() {
    std::thread::sleep(Duration::from_secs(1));  // ❌ Use tokio::time::sleep
}
```

## AvailabilityCheck Entity Pattern

```rust
use sea_orm::entity::prelude::*;

/// Availability status enum with Schema.org parsing
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    InStock,
    OutOfStock,
    BackOrder,
    Unknown,
}

impl AvailabilityStatus {
    /// Parse Schema.org availability value
    pub fn from_schema_org(value: &str) -> Self {
        let normalized = value.to_lowercase();
        if normalized.contains("instock") { Self::InStock }
        else if normalized.contains("outofstock") { Self::OutOfStock }
        else if normalized.contains("backorder") || normalized.contains("preorder") { Self::BackOrder }
        else { Self::Unknown }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InStock => "in_stock",
            Self::OutOfStock => "out_of_stock",
            Self::BackOrder => "back_order",
            Self::Unknown => "unknown",
        }
    }
}

/// Entity with foreign key relation
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "availability_checks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub status: String,
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub checked_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef { Relation::Product.def() }
}
```

## AppSetting Entity Pattern (EAV Model)

```rust
/// Scope enum for hierarchical settings
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SettingScope {
    #[default]
    Global,
    User(String),
    Workspace(String),
    Org(String),
}

impl SettingScope {
    pub fn scope_type(&self) -> &'static str {
        match self {
            SettingScope::Global => "global",
            SettingScope::User(_) => "user",
            SettingScope::Workspace(_) => "workspace",
            SettingScope::Org(_) => "org",
        }
    }

    pub fn scope_id(&self) -> Option<&str> {
        match self {
            SettingScope::Global => None,
            SettingScope::User(id) | SettingScope::Workspace(id) | SettingScope::Org(id) => Some(id),
        }
    }
}

/// EAV-style settings entity
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "app_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub scope_type: String,
    pub scope_id: Option<String>,
    pub key: String,
    #[sea_orm(column_type = "Text")]
    pub value: String,  // JSON-encoded value
    pub updated_at: DateTimeUtc,
}
```

## Scraper Module Patterns

### Orchestrator Pattern

```rust
/// ScraperService coordinates multiple extraction strategies
pub struct ScraperService;

impl ScraperService {
    pub async fn check_availability_with_headless(
        url: &str,
        enable_headless: bool,
    ) -> Result<ScrapingResult, AppError> {
        // Step 1: Validate URL scheme
        Self::validate_url_scheme(url)?;

        // Step 2: Fetch HTML (with headless fallback if bot protection)
        let html = http_client::fetch_html_with_fallback(url, enable_headless).await?;

        // Step 3: Try Schema.org extraction first
        if let Ok(result) = Self::try_schema_org_extraction(&html, url) {
            return Ok(result);
        }

        // Step 4: Fall back to site-specific parsers
        Self::try_site_specific_extraction(&html, url)
    }
}
```

### Site-Specific Adapter Pattern

```rust
/// Chemist Warehouse adapter - checks URL and parses Next.js data
pub fn is_chemist_warehouse_url(url: &str) -> bool {
    url.contains("chemistwarehouse.com.au")
}

pub fn parse_chemist_warehouse_data(page_props: &Value) -> Result<ScrapingResult, AppError> {
    let product = page_props.get("product")
        .ok_or_else(|| AppError::Scraping("No product in pageProps".to_string()))?;

    let availability = product.get("availability")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Scraping("No availability field".to_string()))?;

    let price = product.get("price")
        .and_then(|v| v.as_str())
        .map(|p| price_parser::parse_price(p, "AUD"))
        .unwrap_or_default();

    Ok(ScrapingResult {
        status: parse_availability_status(availability),
        raw_availability: Some(availability.to_string()),
        price,
    })
}
```

## Background Task Pattern

```rust
/// Start background availability checker with Tauri async runtime
pub fn start_availability_checker(app_handle: AppHandle, db: Arc<DbState>) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Get interval from settings
            let interval = match SettingService::get_background_check_interval(db.conn()).await {
                Ok(mins) => Duration::from_secs(mins as u64 * 60),
                Err(_) => Duration::from_secs(60 * 60), // Default 1 hour
            };

            tokio::time::sleep(interval).await;

            // Check if background checks are enabled
            if let Ok(enabled) = SettingService::get_background_check_enabled(db.conn()).await {
                if enabled {
                    let _ = AvailabilityService::check_all_products_with_notification(
                        db.conn(),
                        &app_handle,
                    ).await;
                }
            }
        }
    });
}
```

## Service Composition Pattern

```rust
/// NotificationService composes with AvailabilityService
impl NotificationService {
    pub fn prepare_back_in_stock_notification(
        product_name: &str,
        previous_status: &str,
        current_status: &str,
    ) -> Option<NotificationData> {
        // Only notify if transitioning TO in_stock
        if current_status == "in_stock" && previous_status != "in_stock" {
            Some(NotificationData {
                title: format!("{} is back in stock!", product_name),
                body: "Click to view the product.".to_string(),
            })
        } else {
            None
        }
    }
}

/// AvailabilityService uses NotificationService
impl AvailabilityService {
    pub async fn check_product_with_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<CheckWithNotification, AppError> {
        let previous = Self::get_latest(conn, product_id).await?;
        let check = Self::check_product(conn, product_id).await?;

        let notification = previous.as_ref().and_then(|prev| {
            NotificationService::prepare_back_in_stock_notification(
                &product.name,
                &prev.status,
                &check.status,
            )
        });

        Ok(CheckWithNotification { check, notification, daily_comparison })
    }
}
```

## Extended Error Handling

```rust
pub enum AppError {
    Database(DbErr),           // DATABASE_ERROR
    NotFound(String),          // NOT_FOUND
    Validation(String),        // VALIDATION_ERROR
    Internal(String),          // INTERNAL_ERROR
    Http(reqwest::Error),      // HTTP_ERROR
    Scraping(String),          // SCRAPING_ERROR
    BotProtection(String),     // BOT_PROTECTION
    HttpStatus { status: u16, url: String },  // HTTP_STATUS_ERROR
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Http(_) => "HTTP_ERROR",
            AppError::Scraping(_) => "SCRAPING_ERROR",
            AppError::BotProtection(_) => "BOT_PROTECTION",
            AppError::HttpStatus { .. } => "HTTP_STATUS_ERROR",
        }
    }
}
```

## Summary

**Key Principles:**
1. Commands = IPC only
2. Services = Business logic
3. Repositories = Data access
4. Use connection pool directly (no Mutex)
5. Return AppError, not DbErr
6. Keep it simple and explicit
7. Orchestrator pattern for multi-step operations
8. Site-specific adapters for custom parsing
9. Background tasks via Tauri async_runtime::spawn
10. Service composition for cross-cutting concerns
