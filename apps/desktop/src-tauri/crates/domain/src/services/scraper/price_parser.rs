//! Price parsing utilities for extracting and normalizing prices from Schema.org data.

use rust_decimal::Decimal;
use std::str::FromStr;

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

/// Extract price info from an offer object
pub fn get_price_from_offer(offer: &serde_json::Value) -> PriceInfo {
    let raw_price = offer.get("price").and_then(|p| match p {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    });

    let price_currency = offer
        .get("priceCurrency")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());

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
        let price = get_price_from_offer(&offer);
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
        let price = get_price_from_offer(&offer);
        assert_eq!(price.price_minor_units, Some(4999));
        assert_eq!(price.price_currency, Some("EUR".to_string()));
        assert_eq!(price.raw_price, Some("49.99".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_no_price() {
        let offer = serde_json::json!({
            "availability": "InStock"
        });
        let price = get_price_from_offer(&offer);
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
        let price = get_price_from_offer(&offer);
        assert_eq!(price.price_minor_units, Some(1500));
        assert_eq!(price.price_currency, Some("JPY".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_kwd() {
        let offer = serde_json::json!({
            "price": "29.990",
            "priceCurrency": "KWD"
        });
        let price = get_price_from_offer(&offer);
        assert_eq!(price.price_minor_units, Some(29990));
        assert_eq!(price.price_currency, Some("KWD".to_string()));
    }
}
