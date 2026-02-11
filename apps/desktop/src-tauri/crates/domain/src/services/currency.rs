//! Currency exponent utilities for converting between major and minor units.

/// Zero-decimal currencies (no fractional unit)
const ZERO_DECIMAL_CURRENCIES: &[&str] = &["JPY", "KRW", "VND"];

/// Three-decimal currencies
const THREE_DECIMAL_CURRENCIES: &[&str] = &["KWD", "BHD", "OMR"];

/// Return the number of decimal places for an ISO 4217 currency code.
///
/// - 0 for JPY, KRW, VND (no fractional unit)
/// - 3 for KWD, BHD, OMR
/// - 2 for everything else (USD, EUR, AUD, GBP, etc.)
pub fn currency_exponent(code: &str) -> u32 {
    let upper = code.to_uppercase();

    if ZERO_DECIMAL_CURRENCIES.contains(&upper.as_str()) {
        return 0;
    }

    if THREE_DECIMAL_CURRENCIES.contains(&upper.as_str()) {
        return 3;
    }

    2
}

/// Return 10^exponent for the given currency code.
///
/// - 1 for JPY/KRW/VND
/// - 1000 for KWD/BHD/OMR
/// - 100 for everything else
pub fn minor_unit_multiplier(code: &str) -> i64 {
    10_i64.pow(currency_exponent(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_decimal_currencies() {
        assert_eq!(currency_exponent("JPY"), 0);
        assert_eq!(currency_exponent("KRW"), 0);
        assert_eq!(currency_exponent("VND"), 0);
    }

    #[test]
    fn test_three_decimal_currencies() {
        assert_eq!(currency_exponent("KWD"), 3);
        assert_eq!(currency_exponent("BHD"), 3);
        assert_eq!(currency_exponent("OMR"), 3);
    }

    #[test]
    fn test_two_decimal_currencies() {
        assert_eq!(currency_exponent("USD"), 2);
        assert_eq!(currency_exponent("EUR"), 2);
        assert_eq!(currency_exponent("AUD"), 2);
        assert_eq!(currency_exponent("GBP"), 2);
        assert_eq!(currency_exponent("CAD"), 2);
        assert_eq!(currency_exponent("NZD"), 2);
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(currency_exponent("jpy"), 0);
        assert_eq!(currency_exponent("Jpy"), 0);
        assert_eq!(currency_exponent("kwd"), 3);
        assert_eq!(currency_exponent("Kwd"), 3);
        assert_eq!(currency_exponent("usd"), 2);
        assert_eq!(currency_exponent("Usd"), 2);
    }

    #[test]
    fn test_unknown_currency_defaults_to_two() {
        assert_eq!(currency_exponent("XYZ"), 2);
        assert_eq!(currency_exponent("FOO"), 2);
    }

    #[test]
    fn test_minor_unit_multiplier_zero_decimal() {
        assert_eq!(minor_unit_multiplier("JPY"), 1);
        assert_eq!(minor_unit_multiplier("KRW"), 1);
    }

    #[test]
    fn test_minor_unit_multiplier_two_decimal() {
        assert_eq!(minor_unit_multiplier("USD"), 100);
        assert_eq!(minor_unit_multiplier("AUD"), 100);
    }

    #[test]
    fn test_minor_unit_multiplier_three_decimal() {
        assert_eq!(minor_unit_multiplier("KWD"), 1000);
        assert_eq!(minor_unit_multiplier("BHD"), 1000);
    }
}
