//! Price parsing utilities for extracting and normalizing prices from Schema.org data.

use rust_decimal::Decimal;
use std::str::FromStr;
use url::Url;

use super::super::currency;

/// Price information extracted from Schema.org data
#[derive(Debug, Clone, Default)]
pub struct PriceInfo {
    pub price_minor_units: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
}

/// Parse a price string to minor units (smallest currency unit) using exact decimal arithmetic.
///
/// Handles formats like "789.00", "1,234.56", "789", "789.9"
/// Uses `rust_decimal` to avoid floating-point rounding errors.
/// Multiplies by the correct factor for the given currency (100 for USD, 1 for JPY, 1000 for KWD).
pub fn parse_price_to_minor_units(price_str: &str, currency_code: Option<&str>) -> Option<i64> {
    // Remove currency symbols, whitespace, and thousand separators
    let cleaned: String = price_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    let price = Decimal::from_str(&cleaned).ok()?;
    let multiplier = match currency_code {
        Some(code) => Decimal::from(currency::minor_unit_multiplier(code)),
        None => Decimal::from(100), // default to factor 100
    };

    let minor_units = price * multiplier;
    // to_string then parse avoids depending on the `rust_decimal` i64 conversion trait
    minor_units.round().to_string().parse::<i64>().ok()
}

/// Domain suffix to currency code mappings
/// Within each inner slice, more specific suffixes (e.g., ".com.au") must come before
/// generic ones (e.g., ".au") so that `ends_with` matches the longest suffix first.
pub const DOMAIN_CURRENCY_MAP: &[(&[&str], &str)] = &[
    (&[".com.au", ".au"], "AUD"),
    (&[".co.uk", ".uk"], "GBP"),
    (&[".ca"], "CAD"),
    (&[".co.nz", ".nz"], "NZD"),
    (&[".eu"], "EUR"),
    (&[".com", ".us"], "USD"),
];

/// Path locale patterns to currency mappings
/// Format: (locale_pattern, currency_code)
pub const PATH_LOCALE_CURRENCY_MAP: &[(&str, &str)] = &[
    ("en-au", "AUD"), // Australian English
    ("en-nz", "NZD"), // New Zealand English
    ("en-gb", "GBP"), // British English
    ("en-uk", "GBP"), // Alternative UK pattern
    ("en-ca", "CAD"), // Canadian English
    ("en-us", "USD"), // US English
    ("fr-ca", "CAD"), // French Canadian
    ("en-eu", "EUR"), // EU English (some stores)
                      // Add more as needed
];

/// Infer currency from a store's domain TLD
///
/// Returns a currency code based on the domain's top-level domain.
/// Returns None if the domain doesn't match a known pattern.
pub fn infer_currency_from_domain(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    DOMAIN_CURRENCY_MAP
        .iter()
        .find(|(suffixes, _)| suffixes.iter().any(|s| host.ends_with(s)))
        .map(|(_, currency)| (*currency).to_string())
}

/// Infer currency from URL path locale patterns
///
/// Many multi-locale Shopify stores use paths like:
/// - /en-au/products/... → AUD
/// - /en-gb/products/... → GBP
/// - /en-us/products/... → USD
///
/// This is more reliable than domain TLDs for generic .com domains
/// with multi-currency support.
pub fn infer_currency_from_path(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let path = parsed.path().to_lowercase();

    PATH_LOCALE_CURRENCY_MAP
        .iter()
        .find(|(locale, _)| {
            // Match /locale/ or /locale- patterns
            path.contains(&format!("/{}/", locale))
                || path.contains(&format!("/{}-", locale))
                || path.starts_with(&format!("/{}/", locale))
        })
        .map(|(_, currency)| (*currency).to_string())
}

/// Check if a URL contains a path-based locale pattern
///
/// This is a convenience wrapper around `infer_currency_from_path()` for callers
/// that only need to know if a path locale exists, not which currency it maps to.
///
/// # Examples
/// ```
/// use product_stalker_domain::services::scraper::has_path_locale;
///
/// assert!(has_path_locale("https://reyllen.com/en-au/products/item"));
/// assert!(has_path_locale("https://store.com/en-gb/collections/all"));
/// assert!(!has_path_locale("https://example-en-au.com/products/item")); // domain, not path
/// ```
pub fn has_path_locale(url: &str) -> bool {
    infer_currency_from_path(url).is_some()
}

/// Extract price info from an offer object
///
/// Currency is determined in order of precedence:
/// 1. Path-based locale (e.g., /en-au/ → AUD) - most reliable for multi-locale stores
/// 2. Inferred from the store's domain TLD (e.g., .com.au → AUD)
/// 3. Currency from the offer data (API default, may not match locale)
/// 4. None if none of the above are available
pub fn get_price_from_offer(offer: &serde_json::Value, url: &str) -> PriceInfo {
    let raw_price = offer.get("price").and_then(|p| match p {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    });

    let api_currency = offer
        .get("priceCurrency")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());

    // Apply priority system: path locale > domain > API default
    let price_currency = if raw_price.is_some() {
        infer_currency_from_path(url)
            .or_else(|| infer_currency_from_domain(url))
            .or(api_currency)
    } else {
        None
    };

    let price_minor_units = raw_price
        .as_ref()
        .and_then(|p| parse_price_to_minor_units(p, price_currency.as_deref()));

    PriceInfo {
        price_minor_units,
        price_currency,
        raw_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_simple() {
        assert_eq!(
            parse_price_to_minor_units("789.00", Some("USD")),
            Some(78900)
        );
        assert_eq!(parse_price_to_minor_units("99.99", Some("USD")), Some(9999));
        assert_eq!(parse_price_to_minor_units("49.99", Some("AUD")), Some(4999));
    }

    #[test]
    fn test_parse_price_with_thousands() {
        assert_eq!(
            parse_price_to_minor_units("1,234.56", Some("USD")),
            Some(123456)
        );
        assert_eq!(
            parse_price_to_minor_units("10,000.00", Some("USD")),
            Some(1000000)
        );
    }

    #[test]
    fn test_parse_price_no_decimals() {
        assert_eq!(parse_price_to_minor_units("789", Some("USD")), Some(78900));
        assert_eq!(parse_price_to_minor_units("100", Some("USD")), Some(10000));
    }

    #[test]
    fn test_parse_price_single_decimal() {
        assert_eq!(
            parse_price_to_minor_units("789.9", Some("USD")),
            Some(78990)
        );
        assert_eq!(parse_price_to_minor_units("99.5", Some("USD")), Some(9950));
    }

    #[test]
    fn test_parse_price_with_currency_symbol() {
        assert_eq!(
            parse_price_to_minor_units("$789.00", Some("USD")),
            Some(78900)
        );
        assert_eq!(
            parse_price_to_minor_units("€99.99", Some("EUR")),
            Some(9999)
        );
    }

    #[test]
    fn test_parse_price_empty() {
        assert_eq!(parse_price_to_minor_units("", Some("USD")), None);
        assert_eq!(parse_price_to_minor_units("   ", Some("USD")), None);
    }

    #[test]
    fn test_parse_price_none_currency_defaults_to_100() {
        assert_eq!(parse_price_to_minor_units("99.99", None), Some(9999));
        assert_eq!(parse_price_to_minor_units("789", None), Some(78900));
    }

    #[test]
    fn test_parse_price_jpy_zero_decimals() {
        // JPY: factor = 1 (no fractional units)
        assert_eq!(parse_price_to_minor_units("1500", Some("JPY")), Some(1500));
        assert_eq!(parse_price_to_minor_units("2980", Some("JPY")), Some(2980));
    }

    #[test]
    fn test_parse_price_kwd_three_decimals() {
        // KWD: factor = 1000 (3 decimal places)
        assert_eq!(
            parse_price_to_minor_units("29.990", Some("KWD")),
            Some(29990)
        );
        assert_eq!(parse_price_to_minor_units("5.500", Some("KWD")), Some(5500));
        assert_eq!(parse_price_to_minor_units("100", Some("KWD")), Some(100000));
    }

    #[test]
    fn test_get_price_from_offer_string_price() {
        let offer = serde_json::json!({
            "price": "789.00",
            "priceCurrency": "USD"
        });
        let price = get_price_from_offer(&offer, "https://example.com/product");
        assert_eq!(price.price_minor_units, Some(78900));
        assert_eq!(price.price_currency, Some("USD".to_string()));
        assert_eq!(price.raw_price, Some("789.00".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_numeric_price() {
        let offer = serde_json::json!({
            "price": 49.99,
            "priceCurrency": "EUR"
        });
        let price = get_price_from_offer(&offer, "https://example.eu/product");
        assert_eq!(price.price_minor_units, Some(4999));
        assert_eq!(price.price_currency, Some("EUR".to_string()));
        assert_eq!(price.raw_price, Some("49.99".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_no_price() {
        let offer = serde_json::json!({
            "availability": "InStock"
        });
        let price = get_price_from_offer(&offer, "https://example.com/product");
        assert_eq!(price.price_minor_units, None);
        assert_eq!(price.price_currency, None);
        assert_eq!(price.raw_price, None);
    }

    #[test]
    fn test_get_price_from_offer_jpy() {
        let offer = serde_json::json!({
            "price": "1500",
            "priceCurrency": "JPY"
        });
        let price = get_price_from_offer(&offer, "https://example.jp/product");
        assert_eq!(price.price_minor_units, Some(1500));
        assert_eq!(price.price_currency, Some("JPY".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_kwd() {
        let offer = serde_json::json!({
            "price": "29.990",
            "priceCurrency": "KWD"
        });
        let price = get_price_from_offer(&offer, "https://example.kw/product");
        assert_eq!(price.price_minor_units, Some(29990));
        assert_eq!(price.price_currency, Some("KWD".to_string()));
    }

    #[test]
    fn test_infer_currency_from_domain() {
        assert_eq!(
            infer_currency_from_domain("https://store.com.au/products/test"),
            Some("AUD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://shop.co.uk/products/test"),
            Some("GBP".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://example.ca/products/test"),
            Some("CAD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.com/products/test"),
            Some("USD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://unknown.xyz/products/test"),
            None
        );
    }

    #[test]
    fn test_infer_currency_from_path() {
        // Test all 8 locales defined in PATH_LOCALE_CURRENCY_MAP
        assert_eq!(
            infer_currency_from_path("https://store.com/en-au/products/test"),
            Some("AUD".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://shop.com/en-nz/products/test"),
            Some("NZD".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://shop.com/en-gb/products/test"),
            Some("GBP".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://example.com/en-uk/products/test"),
            Some("GBP".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://store.com/en-ca/products/test"),
            Some("CAD".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://example.com/en-us/products/test"),
            Some("USD".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://shop.com/fr-ca/products/test"),
            Some("CAD".to_string())
        );
        assert_eq!(
            infer_currency_from_path("https://example.com/en-eu/products/test"),
            Some("EUR".to_string())
        );
        // No path locale
        assert_eq!(
            infer_currency_from_path("https://store.com/products/test"),
            None
        );
    }

    #[test]
    fn test_get_price_from_offer_path_locale_takes_precedence() {
        // Path locale (en-au) should override API currency (GBP) and domain (.com)
        let offer = serde_json::json!({
            "price": "99.99",
            "priceCurrency": "GBP"
        });
        let price = get_price_from_offer(&offer, "https://reyllen.com/en-au/products/test");
        assert_eq!(price.price_minor_units, Some(9999));
        assert_eq!(price.price_currency, Some("AUD".to_string())); // Should be AUD, not GBP
        assert_eq!(price.raw_price, Some("99.99".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_domain_fallback() {
        // No path locale, should use domain (.com.au)
        let offer = serde_json::json!({
            "price": "49.99",
            "priceCurrency": "USD"
        });
        let price = get_price_from_offer(&offer, "https://store.com.au/products/test");
        assert_eq!(price.price_minor_units, Some(4999));
        assert_eq!(price.price_currency, Some("AUD".to_string())); // Should use domain
    }

    #[test]
    fn test_get_price_from_offer_api_fallback() {
        // No path locale or recognized domain, should use API currency
        let offer = serde_json::json!({
            "price": "29.99",
            "priceCurrency": "EUR"
        });
        let price = get_price_from_offer(&offer, "https://unknown.xyz/products/test");
        assert_eq!(price.price_minor_units, Some(2999));
        assert_eq!(price.price_currency, Some("EUR".to_string())); // Should use API
    }

    #[test]
    fn test_has_path_locale_en_au_with_slashes() {
        assert!(has_path_locale(
            "https://reyllen.com/en-au/products/backpack"
        ));
    }

    #[test]
    fn test_has_path_locale_en_au_mixed_case() {
        assert!(has_path_locale("https://example.com/EN-AU/products/item"));
    }

    #[test]
    fn test_has_path_locale_en_nz() {
        assert!(has_path_locale("https://example.com/en-nz/products/item"));
    }

    #[test]
    fn test_has_path_locale_en_gb() {
        assert!(has_path_locale("https://example.com/en-gb/products/item"));
    }

    #[test]
    fn test_has_path_locale_en_us() {
        assert!(has_path_locale("https://example.com/en-us/products/item"));
    }

    #[test]
    fn test_has_path_locale_en_ca() {
        assert!(has_path_locale("https://example.com/en-ca/products/item"));
    }

    #[test]
    fn test_has_path_locale_fr_ca() {
        assert!(has_path_locale("https://example.com/fr-ca/products/item"));
    }

    #[test]
    fn test_has_path_locale_no_path_locale() {
        assert!(!has_path_locale("https://example.com/products/item"));
    }

    #[test]
    fn test_has_path_locale_partial_match_no_slashes() {
        // Should not match "en-au" without slashes in the right context
        assert!(!has_path_locale("https://example-en-au.com/products/item"));
    }

    #[test]
    fn test_has_path_locale_query_param_locale() {
        // Query params like ?locale=en-au should not match (needs path locale)
        assert!(!has_path_locale(
            "https://example.com/products/item?locale=en-au"
        ));
    }
}
