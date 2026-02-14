//! Service layer for product-retailer links.

use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::prelude::ProductRetailerModel;
use crate::repositories::{
    CreateProductRetailerParams, ProductRetailerRepository, RetailerRepository,
};
use product_stalker_core::AppError;

/// Parameters for adding a retailer to a product
pub struct AddRetailerParams {
    pub product_id: Uuid,
    pub url: String,
    pub label: Option<String>,
}

/// Parameters for reordering retailers
pub struct ReorderRetailersParams {
    pub updates: Vec<(Uuid, i32)>,
}

/// Service layer for product-retailer business logic
pub struct ProductRetailerService;

impl ProductRetailerService {
    /// Add a retailer URL to a product.
    ///
    /// Validates the URL, extracts the domain, finds or creates the retailer,
    /// and creates the product-retailer link.
    pub async fn add_retailer(
        conn: &DatabaseConnection,
        params: AddRetailerParams,
    ) -> Result<ProductRetailerModel, AppError> {
        Self::validate_url(&params.url)?;
        let domain = Self::extract_domain(&params.url)?;

        let retailer = RetailerRepository::find_or_create_by_domain(conn, &domain).await?;

        let id = Uuid::new_v4();
        ProductRetailerRepository::create(
            conn,
            id,
            retailer.id,
            CreateProductRetailerParams {
                product_id: params.product_id,
                url: params.url,
                label: params.label,
            },
        )
        .await
    }

    /// Get all retailer links for a product
    pub async fn get_retailers_for_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Vec<ProductRetailerModel>, AppError> {
        ProductRetailerRepository::find_by_product_id(conn, product_id).await
    }

    /// Reorder retailer links
    pub async fn reorder(
        conn: &DatabaseConnection,
        params: ReorderRetailersParams,
    ) -> Result<(), AppError> {
        for &(_, sort_order) in &params.updates {
            if sort_order < 0 {
                return Err(AppError::Validation(
                    "sort_order must be non-negative".to_string(),
                ));
            }
        }

        ProductRetailerRepository::update_sort_orders(conn, params.updates).await
    }

    /// Remove a retailer link
    pub async fn remove_retailer(
        conn: &DatabaseConnection,
        product_retailer_id: Uuid,
    ) -> Result<(), AppError> {
        let rows = ProductRetailerRepository::delete_by_id(conn, product_retailer_id).await?;
        if rows == 0 {
            return Err(AppError::NotFound(format!(
                "Product retailer not found: {}",
                product_retailer_id
            )));
        }
        Ok(())
    }

    /// Extract domain from a URL
    pub fn extract_domain(url_str: &str) -> Result<String, AppError> {
        let parsed = url::Url::parse(url_str)
            .map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;
        parsed
            .host_str()
            .map(|h| h.to_string())
            .ok_or_else(|| AppError::Validation("URL has no host".to_string()))
    }

    fn validate_url(url: &str) -> Result<(), AppError> {
        if url.trim().is_empty() {
            return Err(AppError::Validation("URL cannot be empty".to_string()));
        }
        url::Url::parse(url)
            .map_err(|e| AppError::Validation(format!("Invalid URL format: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_https() {
        let domain =
            ProductRetailerService::extract_domain("https://www.amazon.com/dp/B123").unwrap();
        assert_eq!(domain, "www.amazon.com");
    }

    #[test]
    fn test_extract_domain_http() {
        let domain = ProductRetailerService::extract_domain("http://walmart.com/item/456").unwrap();
        assert_eq!(domain, "walmart.com");
    }

    #[test]
    fn test_extract_domain_invalid() {
        let result = ProductRetailerService::extract_domain("not-a-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(ProductRetailerService::validate_url("").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(ProductRetailerService::validate_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(ProductRetailerService::validate_url("not-a-url").is_err());
    }

    #[tokio::test]
    async fn test_reorder_validates_negative_sort_order() {
        // No DB needed — validation happens before any DB call
        let params = ReorderRetailersParams {
            updates: vec![(Uuid::new_v4(), -1)],
        };
        // We need a dummy connection, but since validation fails first, we can use any
        // Actually, reorder takes &DatabaseConnection, so we need one. Let's test the validation
        // logic directly by checking the error type.
        // Use an in-memory DB just to satisfy the signature.
        let conn = crate::test_utils::setup_product_retailer_db().await;
        let result = ProductRetailerService::reorder(&conn, params).await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::repositories::{CreateProductRepoParams, ProductRepository};
    use crate::test_utils::setup_product_retailer_db;

    #[tokio::test]
    async fn test_add_retailer() {
        let conn = setup_product_retailer_db().await;
        let product = ProductRepository::create(
            &conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let pr = ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://amazon.com/dp/B123".to_string(),
                label: Some("Amazon".to_string()),
            },
        )
        .await
        .unwrap();

        assert_eq!(pr.product_id, product.id);
        assert_eq!(pr.url, "https://amazon.com/dp/B123");
        assert_eq!(pr.label, Some("Amazon".to_string()));
    }

    #[tokio::test]
    async fn test_add_retailer_creates_retailer() {
        let conn = setup_product_retailer_db().await;
        let product = ProductRepository::create(
            &conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://bestbuy.com/product/789".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        let retailer = RetailerRepository::find_by_domain(&conn, "bestbuy.com")
            .await
            .unwrap();
        assert!(retailer.is_some());
        assert_eq!(retailer.unwrap().domain, "bestbuy.com");
    }

    #[tokio::test]
    async fn test_get_retailers_for_product() {
        let conn = setup_product_retailer_db().await;
        let product = ProductRepository::create(
            &conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://amazon.com/dp/B123".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://walmart.com/item/456".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        let retailers = ProductRetailerService::get_retailers_for_product(&conn, product.id)
            .await
            .unwrap();
        assert_eq!(retailers.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_retailer() {
        let conn = setup_product_retailer_db().await;
        let product = ProductRepository::create(
            &conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let pr = ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://amazon.com/dp/B123".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        ProductRetailerService::remove_retailer(&conn, pr.id)
            .await
            .unwrap();

        let retailers = ProductRetailerService::get_retailers_for_product(&conn, product.id)
            .await
            .unwrap();
        assert!(retailers.is_empty());
    }

    #[tokio::test]
    async fn test_remove_retailer_not_found() {
        let conn = setup_product_retailer_db().await;
        let result = ProductRetailerService::remove_retailer(&conn, Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_reorder_retailers() {
        let conn = setup_product_retailer_db().await;
        let product = ProductRepository::create(
            &conn,
            Uuid::new_v4(),
            CreateProductRepoParams {
                name: "Test".to_string(),
                url: None,
                description: None,
                notes: None,
            },
        )
        .await
        .unwrap();

        let pr1 = ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://amazon.com/dp/B123".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        let pr2 = ProductRetailerService::add_retailer(
            &conn,
            AddRetailerParams {
                product_id: product.id,
                url: "https://walmart.com/item/456".to_string(),
                label: None,
            },
        )
        .await
        .unwrap();

        // Swap order
        ProductRetailerService::reorder(
            &conn,
            super::ReorderRetailersParams {
                updates: vec![(pr2.id, 0), (pr1.id, 1)],
            },
        )
        .await
        .unwrap();

        let retailers = ProductRetailerService::get_retailers_for_product(&conn, product.id)
            .await
            .unwrap();
        assert_eq!(retailers[0].id, pr2.id);
        assert_eq!(retailers[1].id, pr1.id);
    }
}
