//! Formatting utility functions.
//!
//! Ported from the original `formatters.ts` — pure string formatters for
//! display of ERP data: currency amounts, dates, percentages, quantities,
//! phone numbers, and more.
//!
//! # Locale
//!
//! All formatters use PKR (`Rs.`) as the default currency and `en`-style
//! date formats. The `format_currency` function accepts an optional
//! code parameter for other currencies.

use chrono::{NaiveDate, NaiveDateTime};

// ---------------------------------------------------------------------------
// Currency formatting
// ---------------------------------------------------------------------------

/// Format a monetary amount as a currency string.
///
/// Formats with:
/// - Thousands separator (commas for `en` style).
/// - 2 decimal places by default.
/// - Currency prefix (e.g. `"Rs. "` for PKR).
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_currency(1234567.89), "Rs. 1,234,567.89");
/// assert_eq!(format_currency(0.0), "Rs. 0.00");
/// assert_eq!(format_currency(-500.50), "Rs. -500.50");
/// ```
pub fn format_currency(amount: f64) -> String {
    format_currency_with_code(amount, "Rs.", 2)
}

/// Format a monetary amount with a custom currency code and decimal places.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_currency_with_code(1234.5, "USD", 2), "USD 1,234.50");
/// assert_eq!(format_currency_with_code(50.0, "PKR", 0), "PKR 50");
/// ```
pub fn format_currency_with_code(amount: f64, code: &str, decimals: u8) -> String {
    let abs_amount = amount.abs();
    let int_part = abs_amount as i64;
    let dec_places = decimals as usize;

    // Format the integer part with thousands separators
    let int_str = int_part
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap_or_default()
        .join(",");

    let sign = if amount < 0.0 { "-" } else { "" };

    if dec_places > 0 {
        let frac = ((abs_amount - int_part as f64) * 10_f64.powi(dec_places as i32)).round() as i64;
        format!("{} {}{}.{:0width$}", code, sign, int_str, frac, width = dec_places)
    } else {
        format!("{} {}{}", code, sign, int_str)
    }
}

// ---------------------------------------------------------------------------
// Date & time formatting
// ---------------------------------------------------------------------------

/// Format a date using the default display format (e.g. `"2024-01-15"`).
///
/// Uses `chrono`'s built-in formatter with format `"%Y-%m-%d"`.
///
/// # Example
///
/// ```ignore
/// let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
/// assert_eq!(format_date(&date), "2024-01-15");
/// ```
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Format a date in a human-friendly format (e.g. `"Jan 15, 2024"`).
///
/// Uses format `"%b %d, %Y"`.
pub fn format_date_human(date: &NaiveDate) -> String {
    date.format("%b %d, %Y").to_string()
}

/// Format a date using an arbitrary `chrono` format string.
///
/// # Example
///
/// ```ignore
/// let date = NaiveDate::from_ymd_opt(2024, 6, 18).unwrap();
/// assert_eq!(format_date_custom(&date, "%d-%b-%Y"), "18-Jun-2024");
/// ```
pub fn format_date_custom(date: &NaiveDate, fmt: &str) -> String {
    date.format(fmt).to_string()
}

/// Parse and format a date string from `"YYYY-MM-DD"` to a custom format.
///
/// Returns `String::new()` if the input is empty or invalid.
pub fn parse_and_format_date(date_str: &str, fmt: &str) -> String {
    if date_str.is_empty() {
        return String::new();
    }
    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => date.format(fmt).to_string(),
        Err(_) => date_str.to_string(), // fallback: return original string
    }
}

// ---------------------------------------------------------------------------
// DateTime formatting
// ---------------------------------------------------------------------------

/// Format a `NaiveDateTime` for display (e.g. `"2024-01-15 14:30:00"`).
///
/// Uses format `"%Y-%m-%d %H:%M:%S"`.
pub fn format_datetime(dt: &NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a `NaiveDateTime` with a custom format string.
pub fn format_datetime_custom(dt: &NaiveDateTime, fmt: &str) -> String {
    dt.format(fmt).to_string()
}

// ---------------------------------------------------------------------------
// Percentage formatting
// ---------------------------------------------------------------------------

/// Format a ratio as a percentage string.
///
/// Multiplies by 100 and appends `"%"`. Handles negative values.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_percent(0.456), "45.6%");
/// assert_eq!(format_percent(1.0), "100.0%");
/// assert_eq!(format_percent(0.0), "0.0%");
/// ```
pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", value * 100.0)
}

/// Format a ratio with a custom number of decimal places.
pub fn format_percent_with_decimals(value: f64, decimals: u8) -> String {
    format!("{:.1$}%", value * 100.0, decimals as usize)
}

// ---------------------------------------------------------------------------
// Quantity formatting
// ---------------------------------------------------------------------------

/// Format a quantity with its unit of measure.
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_quantity(100.0, "Nos"), "100.00 Nos");
/// assert_eq!(format_quantity(0.5, "Kg"), "0.50 Kg");
/// ```
pub fn format_quantity(amount: f64, uom: &str) -> String {
    format!("{:.2} {}", amount, uom)
}

/// Format a quantity with custom decimal places.
pub fn format_quantity_custom(amount: f64, uom: &str, decimals: u8) -> String {
    format!("{:.*} {}", decimals as usize, amount, uom)
}

/// Format an integer quantity (no decimals).
pub fn format_quantity_int(amount: i64, uom: &str) -> String {
    format!("{} {}", amount, uom)
}

// ---------------------------------------------------------------------------
// Phone formatting
// ---------------------------------------------------------------------------

/// Format a phone number into a readable Pakistani-style format.
///
/// Handles:
/// - `+92 3XX XXXXXXX` (mobile)
/// - `+92 XX XXXXXXX` (landline)
/// - Raw numbers without country code (adds `+92` prefix)
///
/// # Examples
///
/// ```ignore
/// assert_eq!(format_phone("+923001112233"), "+92 300 1112233");
/// assert_eq!(format_phone("03001112233"), "+92 300 1112233");
/// ```
pub fn format_phone(phone: &str) -> String {
    let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    if cleaned.len() == 12 && cleaned.starts_with("92") {
        // +92 3XX XXXXXXX (mobile)
        if cleaned[2..3] == *"3" {
            format!(
                "+92 {} {}{}",
                &cleaned[2..5],
                &cleaned[5..8],
                &cleaned[8..12]
            )
        } else {
            // +92 XX XXXXXXX (landline)
            format!(
                "+92 {} {}",
                &cleaned[2..4],
                &cleaned[4..11]
            )
        }
    } else if cleaned.len() == 11 && cleaned.starts_with("92") {
        // 92421112233 → +92 42 1112233 (landline with country code, no +)
        format!("+92 {} {}", &cleaned[2..4], &cleaned[4..11])
    } else if cleaned.len() == 11 && cleaned.starts_with("0") {
        // 0300 1112233 → +92 300 1112233
        let without_zero = &cleaned[1..];
        format_phone(&format!("92{}", without_zero))
    } else if cleaned.len() == 10 && !cleaned.starts_with('0') {
        // 300 1112233 → +92 300 1112233
        format_phone(&format!("92{}", cleaned))
    } else {
        // Can't parse — return as-is
        phone.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_format_currency() {
        let s = format_currency(1234567.89);
        assert!(s.contains("Rs."));
        assert!(s.contains("1,234,567.89"));
    }

    #[test]
    fn test_format_currency_zero() {
        let s = format_currency(0.0);
        assert_eq!(s, "Rs. 0.00");
    }

    #[test]
    fn test_format_currency_negative() {
        let s = format_currency(-500.50);
        assert!(s.contains("-500.50"));
    }

    #[test]
    fn test_format_currency_with_code() {
        let s = format_currency_with_code(1234.5, "USD", 2);
        assert_eq!(s, "USD 1,234.50");
    }

    #[test]
    fn test_format_date_human() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(format_date_human(&date), "Jan 15, 2024");
    }

    #[test]
    fn test_format_date_custom() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 18).unwrap();
        assert_eq!(format_date_custom(&date, "%d-%b-%Y"), "18-Jun-2024");
    }

    #[test]
    fn test_parse_and_format() {
        let s = parse_and_format_date("2024-06-18", "%d-%b-%Y");
        assert_eq!(s, "18-Jun-2024");
    }

    #[test]
    fn test_parse_and_format_empty() {
        assert_eq!(parse_and_format_date("", "%d-%b-%Y"), "");
    }

    #[test]
    fn test_parse_and_format_invalid() {
        // Returns original string on parse failure
        let s = parse_and_format_date("NOT-A-DATE", "%d-%b-%Y");
        assert_eq!(s, "NOT-A-DATE");
    }

    #[test]
    fn test_format_percent() {
        assert_eq!(format_percent(0.456), "45.6%");
        assert_eq!(format_percent(1.0), "100.0%");
        assert_eq!(format_percent(0.0), "0.0%");
    }

    #[test]
    fn test_format_percent_decimals() {
        assert_eq!(format_percent_with_decimals(0.45678, 2), "45.68%");
    }

    #[test]
    fn test_format_quantity() {
        assert_eq!(format_quantity(100.0, "Nos"), "100.00 Nos");
        assert_eq!(format_quantity(0.5, "Kg"), "0.50 Kg");
    }

    #[test]
    fn test_format_phone_mobile() {
        assert_eq!(format_phone("+923001112233"), "+92 300 1112233");
    }

    #[test]
    fn test_format_phone_local() {
        assert_eq!(format_phone("03001112233"), "+92 300 1112233");
    }

    #[test]
    fn test_format_phone_short() {
        assert_eq!(format_phone("3001112233"), "+92 300 1112233");
    }

    #[test]
    fn test_format_phone_invalid() {
        assert_eq!(format_phone("abc"), "abc");
    }
}
