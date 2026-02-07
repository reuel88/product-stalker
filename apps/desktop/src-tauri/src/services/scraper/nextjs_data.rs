//! Next.js data extractor for parsing __NEXT_DATA__ script content.
//!
//! Many modern e-commerce sites use Next.js which embeds page data in a
//! `<script id="__NEXT_DATA__">` tag. This module provides utilities to
//! extract and parse this data as a fallback when Schema.org data is not available.

use scraper::{Html, Selector};
use serde_json::Value;

use crate::error::AppError;

/// Extract the __NEXT_DATA__ JSON from HTML content.
///
/// Next.js embeds page data in a script tag like:
/// ```html
/// <script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{...}}}</script>
/// ```
pub fn extract_next_data(html: &str) -> Result<Value, AppError> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"script#__NEXT_DATA__"#)
        .map_err(|e| AppError::Scraping(format!("Invalid selector: {:?}", e)))?;

    let script = document
        .select(&selector)
        .next()
        .ok_or_else(|| AppError::Scraping("No __NEXT_DATA__ script found".to_string()))?;

    let json_text = script.text().collect::<String>();

    serde_json::from_str(&json_text)
        .map_err(|e| AppError::Scraping(format!("Failed to parse __NEXT_DATA__ JSON: {}", e)))
}

/// Get the pageProps from Next.js data.
///
/// The standard Next.js structure is:
/// ```json
/// {
///   "props": {
///     "pageProps": {
///       // page-specific data
///     }
///   }
/// }
/// ```
pub fn get_page_props(next_data: &Value) -> Option<&Value> {
    next_data.get("props")?.get("pageProps")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_next_data_success() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script id="__NEXT_DATA__" type="application/json">
                {"props":{"pageProps":{"product":{"name":"Test Product"}}}}
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = extract_next_data(html).unwrap();
        assert!(result.get("props").is_some());
        assert!(result["props"]["pageProps"]["product"]["name"]
            .as_str()
            .unwrap()
            .contains("Test Product"));
    }

    #[test]
    fn test_extract_next_data_no_script() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test</title></head>
            <body></body>
            </html>
        "#;

        let result = extract_next_data(html);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Scraping(msg) => {
                assert!(msg.contains("No __NEXT_DATA__ script found"));
            }
            _ => panic!("Expected Scraping error"),
        }
    }

    #[test]
    fn test_extract_next_data_invalid_json() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script id="__NEXT_DATA__" type="application/json">
                not valid json
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = extract_next_data(html);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Scraping(msg) => {
                assert!(msg.contains("Failed to parse __NEXT_DATA__ JSON"));
            }
            _ => panic!("Expected Scraping error"),
        }
    }

    #[test]
    fn test_get_page_props_success() {
        let data = serde_json::json!({
            "props": {
                "pageProps": {
                    "product": {
                        "name": "Test Product",
                        "price": "23.99"
                    }
                }
            }
        });

        let page_props = get_page_props(&data).unwrap();
        assert!(page_props.get("product").is_some());
        assert_eq!(
            page_props["product"]["name"].as_str().unwrap(),
            "Test Product"
        );
    }

    #[test]
    fn test_get_page_props_missing_props() {
        let data = serde_json::json!({
            "other": "data"
        });

        assert!(get_page_props(&data).is_none());
    }

    #[test]
    fn test_get_page_props_missing_page_props() {
        let data = serde_json::json!({
            "props": {
                "other": "data"
            }
        });

        assert!(get_page_props(&data).is_none());
    }

    #[test]
    fn test_extract_next_data_with_complex_structure() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script id="__NEXT_DATA__" type="application/json">
                {
                    "props": {
                        "pageProps": {
                            "product": {
                                "name": "Curash Simply Water Wipes 6 x 80 Pack",
                                "sku": "2678514",
                                "price": "23.99",
                                "availability": "in-stock"
                            }
                        }
                    },
                    "buildId": "abc123"
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = extract_next_data(html).unwrap();
        let page_props = get_page_props(&result).unwrap();
        let product = page_props.get("product").unwrap();

        assert_eq!(
            product["name"].as_str().unwrap(),
            "Curash Simply Water Wipes 6 x 80 Pack"
        );
        assert_eq!(product["sku"].as_str().unwrap(), "2678514");
        assert_eq!(product["price"].as_str().unwrap(), "23.99");
        assert_eq!(product["availability"].as_str().unwrap(), "in-stock");
    }
}
