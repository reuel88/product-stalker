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

## Summary

**Key Principles:**
1. Commands = IPC only
2. Services = Business logic
3. Repositories = Data access
4. Use connection pool directly (no Mutex)
5. Return AppError, not DbErr
6. Keep it simple and explicit
