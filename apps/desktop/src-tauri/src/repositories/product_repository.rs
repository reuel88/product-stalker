use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;

use crate::entities::prelude::*;
use crate::error::AppError;

/// Repository for product data access
///
/// Encapsulates all database operations for products.
/// This keeps SeaORM details isolated from business logic.
pub struct ProductRepository;

impl ProductRepository {
    /// Find all products
    pub async fn find_all(conn: &DatabaseConnection) -> Result<Vec<ProductModel>, AppError> {
        let products = Product::find().all(conn).await?;
        Ok(products)
    }

    /// Find a product by ID
    pub async fn find_by_id(
        conn: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<ProductModel>, AppError> {
        let product = Product::find_by_id(id).one(conn).await?;
        Ok(product)
    }

    /// Create a new product
    pub async fn create(
        conn: &DatabaseConnection,
        id: Uuid,
        name: String,
        url: String,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<ProductModel, AppError> {
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

    /// Update an existing product
    pub async fn update(
        conn: &DatabaseConnection,
        model: ProductModel,
        name: Option<String>,
        url: Option<String>,
        description: Option<Option<String>>,
        notes: Option<Option<String>>,
    ) -> Result<ProductModel, AppError> {
        let mut active_model: ProductActiveModel = model.into();

        if let Some(name) = name {
            active_model.name = Set(name);
        }
        if let Some(url) = url {
            active_model.url = Set(url);
        }
        if let Some(description) = description {
            active_model.description = Set(description);
        }
        if let Some(notes) = notes {
            active_model.notes = Set(notes);
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(conn).await?;
        Ok(updated)
    }

    /// Delete a product by ID
    pub async fn delete_by_id(conn: &DatabaseConnection, id: Uuid) -> Result<u64, AppError> {
        let result = Product::delete_by_id(id).exec(conn).await?;
        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

    async fn setup_test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Product);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_and_find_product() {
        let conn = setup_test_db().await;
        let id = Uuid::new_v4();

        let created = ProductRepository::create(
            &conn,
            id,
            "Test".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(created.name, "Test");

        let found = ProductRepository::find_by_id(&conn, id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let conn = setup_test_db().await;
        let products = ProductRepository::find_all(&conn).await.unwrap();
        assert!(products.is_empty());
    }

    #[tokio::test]
    async fn test_delete_product() {
        let conn = setup_test_db().await;
        let id = Uuid::new_v4();

        ProductRepository::create(
            &conn,
            id,
            "Test".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let rows = ProductRepository::delete_by_id(&conn, id).await.unwrap();
        assert_eq!(rows, 1);

        let found = ProductRepository::find_by_id(&conn, id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_product() {
        let conn = setup_test_db().await;
        let id = Uuid::new_v4();

        let created = ProductRepository::create(
            &conn,
            id,
            "Original".to_string(),
            "https://original.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let updated = ProductRepository::update(
            &conn,
            created,
            Some("Updated".to_string()),
            None,
            None,
            None,
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.url, "https://original.com");
    }
}
