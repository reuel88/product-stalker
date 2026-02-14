use product_stalker_core::AppError;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::prelude::*;

/// Parameters for creating a product-retailer link
pub struct CreateProductRetailerParams {
    pub product_id: Uuid,
    pub url: String,
    pub label: Option<String>,
}

/// Repository for product-retailer junction data access
pub struct ProductRetailerRepository;

impl ProductRetailerRepository {
    /// Create a new product-retailer link
    pub async fn create(
        conn: &DatabaseConnection,
        id: Uuid,
        retailer_id: Uuid,
        params: CreateProductRetailerParams,
    ) -> Result<ProductRetailerModel, AppError> {
        let now = chrono::Utc::now();

        let active_model = ProductRetailerActiveModel {
            id: Set(id),
            product_id: Set(params.product_id),
            retailer_id: Set(retailer_id),
            url: Set(params.url),
            label: Set(params.label),
            created_at: Set(now),
        };

        let link = active_model.insert(conn).await?;
        Ok(link)
    }

    /// Find all product-retailer links for a product
    pub async fn find_by_product_id(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Vec<ProductRetailerModel>, AppError> {
        let links = ProductRetailer::find()
            .filter(ProductRetailerColumn::ProductId.eq(product_id))
            .all(conn)
            .await?;
        Ok(links)
    }

    /// Find a product-retailer link by ID
    pub async fn find_by_id(
        conn: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<ProductRetailerModel>, AppError> {
        let link = ProductRetailer::find_by_id(id).one(conn).await?;
        Ok(link)
    }

    /// Delete a product-retailer link by ID
    pub async fn delete_by_id(conn: &DatabaseConnection, id: Uuid) -> Result<u64, AppError> {
        let result = ProductRetailer::delete_by_id(id).exec(conn).await?;
        Ok(result.rows_affected)
    }

    /// Find all product-retailer links with their associated products (for bulk checks)
    pub async fn find_all_with_product(
        conn: &DatabaseConnection,
    ) -> Result<Vec<(ProductRetailerModel, Option<ProductModel>)>, AppError> {
        let results = ProductRetailer::find()
            .find_also_related(crate::entities::product::Entity)
            .all(conn)
            .await?;
        Ok(results)
    }

    /// Find a product-retailer link by ID with its retailer
    pub async fn find_by_id_with_retailer(
        conn: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<(ProductRetailerModel, Option<RetailerModel>)>, AppError> {
        let result = ProductRetailer::find_by_id(id)
            .find_also_related(crate::entities::retailer::Entity)
            .one(conn)
            .await?;
        Ok(result)
    }

    /// Count how many retailer links a product has
    pub async fn count_by_product_id(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<u64, AppError> {
        use sea_orm::PaginatorTrait;
        let count = ProductRetailer::find()
            .filter(ProductRetailerColumn::ProductId.eq(product_id))
            .count(conn)
            .await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::RetailerRepository;
    use crate::test_utils::setup_product_retailer_db;

    async fn create_test_data(
        conn: &DatabaseConnection,
    ) -> (ProductModel, RetailerModel, ProductRetailerModel) {
        use crate::repositories::{CreateProductRepoParams, ProductRepository};

        let product = ProductRepository::create(
            conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test Product".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let retailer = RetailerRepository::find_or_create_by_domain(conn, "amazon.com")
            .await
            .unwrap();

        let pr = ProductRetailerRepository::create(
            conn,
            Uuid::new_v4(),
            retailer.id,
            CreateProductRetailerParams {
                product_id: product.id,
                url: "https://amazon.com/dp/B123".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        (product, retailer, pr)
    }

    #[tokio::test]
    async fn test_create_and_find() {
        let conn = setup_product_retailer_db().await;
        let (product, _, pr) = create_test_data(&conn).await;

        let found = ProductRetailerRepository::find_by_id(&conn, pr.id)
            .await
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().product_id, product.id);
    }

    #[tokio::test]
    async fn test_find_by_product_id() {
        let conn = setup_product_retailer_db().await;
        let (product, _retailer, _) = create_test_data(&conn).await;

        // Add second retailer link
        let retailer2 = RetailerRepository::find_or_create_by_domain(&conn, "walmart.com")
            .await
            .unwrap();
        ProductRetailerRepository::create(
            &conn,
            Uuid::new_v4(),
            retailer2.id,
            CreateProductRetailerParams {
                product_id: product.id,
                url: "https://walmart.com/item/456".to_string(),
                label: Some("Walmart".to_string()),
            },
        )
        .await
        .unwrap();

        let links = ProductRetailerRepository::find_by_product_id(&conn, product.id)
            .await
            .unwrap();
        assert_eq!(links.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_by_id() {
        let conn = setup_product_retailer_db().await;
        let (_, _, pr) = create_test_data(&conn).await;

        let rows = ProductRetailerRepository::delete_by_id(&conn, pr.id)
            .await
            .unwrap();
        assert_eq!(rows, 1);

        let found = ProductRetailerRepository::find_by_id(&conn, pr.id)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_all_with_product() {
        let conn = setup_product_retailer_db().await;
        let _ = create_test_data(&conn).await;

        let results = ProductRetailerRepository::find_all_with_product(&conn)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_some());
    }

    #[tokio::test]
    async fn test_count_by_product_id() {
        let conn = setup_product_retailer_db().await;
        let (product, _retailer, _) = create_test_data(&conn).await;

        let count = ProductRetailerRepository::count_by_product_id(&conn, product.id)
            .await
            .unwrap();
        assert_eq!(count, 1);

        // Add another
        let retailer2 = RetailerRepository::find_or_create_by_domain(&conn, "bestbuy.com")
            .await
            .unwrap();
        ProductRetailerRepository::create(
            &conn,
            Uuid::new_v4(),
            retailer2.id,
            CreateProductRetailerParams {
                product_id: product.id,
                url: "https://bestbuy.com/product/789".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        let count = ProductRetailerRepository::count_by_product_id(&conn, product.id)
            .await
            .unwrap();
        assert_eq!(count, 2);
    }
}
