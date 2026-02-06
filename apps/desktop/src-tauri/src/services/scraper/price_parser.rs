//! Price parsing utilities for extracting and normalizing prices from Schema.org data.

/// Price information extracted from Schema.org data
#[derive(Debug, Clone, Default)]
pub struct PriceInfo {
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
}

/// Parse a price string to cents (smallest currency unit)
/// Handles formats like "789.00", "1,234.56", "789", "789.9"
pub fn parse_price_to_cents(price_str: &str) -> Option<i64> {
    // Remove currency symbols, whitespace, and thousand separators
    let cleaned: String = price_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();

    if cleaned.is_empty() {
        return None;
    }

    // Parse as float and convert to cents
    let price: f64 = cleaned.parse().ok()?;
    Some((price * 100.0).round() as i64)
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

    let price_cents = raw_price.as_ref().and_then(|p| parse_price_to_cents(p));

    PriceInfo {
        price_cents,
        price_currency,
        raw_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price_to_cents_simple() {
        assert_eq!(parse_price_to_cents("789.00"), Some(78900));
        assert_eq!(parse_price_to_cents("99.99"), Some(9999));
        assert_eq!(parse_price_to_cents("49.99"), Some(4999));
    }

    #[test]
    fn test_parse_price_to_cents_with_thousands() {
        assert_eq!(parse_price_to_cents("1,234.56"), Some(123456));
        assert_eq!(parse_price_to_cents("10,000.00"), Some(1000000));
    }

    #[test]
    fn test_parse_price_to_cents_no_decimals() {
        assert_eq!(parse_price_to_cents("789"), Some(78900));
        assert_eq!(parse_price_to_cents("100"), Some(10000));
    }

    #[test]
    fn test_parse_price_to_cents_single_decimal() {
        assert_eq!(parse_price_to_cents("789.9"), Some(78990));
        assert_eq!(parse_price_to_cents("99.5"), Some(9950));
    }

    #[test]
    fn test_parse_price_to_cents_with_currency_symbol() {
        assert_eq!(parse_price_to_cents("$789.00"), Some(78900));
        assert_eq!(parse_price_to_cents("€99.99"), Some(9999));
    }

    #[test]
    fn test_parse_price_to_cents_empty() {
        assert_eq!(parse_price_to_cents(""), None);
        assert_eq!(parse_price_to_cents("   "), None);
    }

    #[test]
    fn test_get_price_from_offer_string_price() {
        let offer = serde_json::json!({
            "price": "789.00",
            "priceCurrency": "USD"
        });
        let price = get_price_from_offer(&offer);
        assert_eq!(price.price_cents, Some(78900));
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
        assert_eq!(price.price_cents, Some(4999));
        assert_eq!(price.price_currency, Some("EUR".to_string()));
        assert_eq!(price.raw_price, Some("49.99".to_string()));
    }

    #[test]
    fn test_get_price_from_offer_no_price() {
        let offer = serde_json::json!({
            "availability": "InStock"
        });
        let price = get_price_from_offer(&offer);
        assert_eq!(price.price_cents, None);
        assert_eq!(price.price_currency, None);
        assert_eq!(price.raw_price, None);
    }
}
