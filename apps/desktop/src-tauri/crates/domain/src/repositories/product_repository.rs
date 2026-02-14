use product_stalker_core::AppError;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;

use crate::entities::prelude::*;

/// Input for updating a product's fields.
///
/// Uses Option to indicate which fields should be updated:
/// - `None` = keep existing value unchanged
/// - `Some(value)` = update to new value
///
/// For nullable fields (`description`, `notes`), uses nested Option:
/// - `None` = keep existing value unchanged
/// - `Some(None)` = clear the field (set to NULL)
/// - `Some(Some(value))` = set to the new value
#[derive(Default)]
pub struct ProductUpdateInput {
    pub name: Option<String>,
    pub url: Option<Option<String>>,
    pub description: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub currency: Option<Option<String>>,
}

/// Parameters for creating a new product at the repository level
pub struct CreateProductRepoParams {
    pub name: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

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
        params: CreateProductRepoParams,
    ) -> Result<ProductModel, AppError> {
        let now = chrono::Utc::now();

        let active_model = ProductActiveModel {
            id: Set(id),
            name: Set(params.name),
            url: Set(params.url),
            description: Set(params.description),
            notes: Set(params.notes),
            currency: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let product = active_model.insert(conn).await?;
        Ok(product)
    }

    /// Find all product IDs that have no associated product_retailers
    pub async fn find_all_without_retailers(
        conn: &DatabaseConnection,
    ) -> Result<Vec<ProductModel>, AppError> {
        use crate::entities::prelude::ProductRetailerColumn;
        use sea_orm::{ColumnTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

        let products = Product::find()
            .join(
                JoinType::LeftJoin,
                crate::entities::product::Relation::ProductRetailers.def(),
            )
            .filter(ProductRetailerColumn::Id.is_null())
            .all(conn)
            .await?;
        Ok(products)
    }

    /// Update an existing product.
    ///
    /// # Arguments
    ///
    /// * `model` - The existing product model to update
    /// * `input` - The fields to update (see [`ProductUpdateInput`] for details)
    pub async fn update(
        conn: &DatabaseConnection,
        model: ProductModel,
        input: ProductUpdateInput,
    ) -> Result<ProductModel, AppError> {
        let mut active_model: ProductActiveModel = model.into();

        if let Some(name) = input.name {
            active_model.name = Set(name);
        }
        if let Some(url) = input.url {
            active_model.url = Set(url);
        }
        if let Some(description) = input.description {
            active_model.description = Set(description);
        }
        if let Some(notes) = input.notes {
            active_model.notes = Set(notes);
        }
        if let Some(currency) = input.currency {
            active_model.currency = Set(currency);
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
    use crate::test_utils::setup_products_db;

    fn params(name: &str, url: &str) -> CreateProductRepoParams {
        CreateProductRepoParams {
            name: name.to_string(),
            url: Some(url.to_string()),
            description: None,
            notes: None,
        }
    }

    #[tokio::test]
    async fn test_create_and_find_product() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        let created = ProductRepository::create(&conn, id, params("Test", "https://test.com"))
            .await
            .unwrap();

        assert_eq!(created.name, "Test");

        let found = ProductRepository::find_by_id(&conn, id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_find_all_empty() {
        let conn = setup_products_db().await;
        let products = ProductRepository::find_all(&conn).await.unwrap();
        assert!(products.is_empty());
    }

    #[tokio::test]
    async fn test_delete_product() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        ProductRepository::create(&conn, id, params("Test", "https://test.com"))
            .await
            .unwrap();

        let rows = ProductRepository::delete_by_id(&conn, id).await.unwrap();
        assert_eq!(rows, 1);

        let found = ProductRepository::find_by_id(&conn, id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_product() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        let created =
            ProductRepository::create(&conn, id, params("Original", "https://original.com"))
                .await
                .unwrap();

        let updated = ProductRepository::update(
            &conn,
            created,
            ProductUpdateInput {
                name: Some("Updated".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.url, Some("https://original.com".to_string()));
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let conn = setup_products_db().await;
        let fake_id = Uuid::new_v4();

        let found = ProductRepository::find_by_id(&conn, fake_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_non_existent_product() {
        let conn = setup_products_db().await;
        let fake_id = Uuid::new_v4();

        let rows = ProductRepository::delete_by_id(&conn, fake_id)
            .await
            .unwrap();
        assert_eq!(rows, 0);
    }

    #[tokio::test]
    async fn test_find_all_with_multiple_products() {
        let conn = setup_products_db().await;

        // Create 3 products
        for i in 1..=3 {
            ProductRepository::create(
                &conn,
                Uuid::new_v4(),
                params(&format!("Product {}", i), &format!("https://p{}.com", i)),
            )
            .await
            .unwrap();
        }

        let products = ProductRepository::find_all(&conn).await.unwrap();
        assert_eq!(products.len(), 3);
    }

    #[tokio::test]
    async fn test_create_with_all_fields() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        let created = ProductRepository::create(
            &conn,
            id,
            CreateProductRepoParams {
                name: "Full Product".to_string(),
                url: Some("https://full.com".to_string()),
                description: Some("A description".to_string()),
                notes: Some("Some notes".to_string()),
            },
        )
        .await
        .unwrap();

        assert_eq!(created.name, "Full Product");
        assert_eq!(created.url, Some("https://full.com".to_string()));
        assert_eq!(created.description, Some("A description".to_string()));
        assert_eq!(created.notes, Some("Some notes".to_string()));
    }

    #[tokio::test]
    async fn test_update_all_fields() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        let created =
            ProductRepository::create(&conn, id, params("Original", "https://original.com"))
                .await
                .unwrap();

        let updated = ProductRepository::update(
            &conn,
            created,
            ProductUpdateInput {
                name: Some("New Name".to_string()),
                url: Some(Some("https://new.com".to_string())),
                description: Some(Some("New description".to_string())),
                notes: Some(Some("New notes".to_string())),
                currency: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.url, Some("https://new.com".to_string()));
        assert_eq!(updated.description, Some("New description".to_string()));
        assert_eq!(updated.notes, Some("New notes".to_string()));
    }

    #[tokio::test]
    async fn test_update_clear_optional_fields() {
        let conn = setup_products_db().await;
        let id = Uuid::new_v4();

        let created = ProductRepository::create(
            &conn,
            id,
            CreateProductRepoParams {
                name: "Product".to_string(),
                url: Some("https://product.com".to_string()),
                description: Some("Has description".to_string()),
                notes: Some("Has notes".to_string()),
            },
        )
        .await
        .unwrap();

        // Clear description and notes by setting them to None
        let updated = ProductRepository::update(
            &conn,
            created,
            ProductUpdateInput {
                description: Some(None), // Clear description
                notes: Some(None),       // Clear notes
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert!(updated.description.is_none());
        assert!(updated.notes.is_none());
    }
}
