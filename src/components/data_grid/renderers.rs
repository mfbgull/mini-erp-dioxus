//! Cell rendering functions for the `DataGrid<T>`.
//!
//! Each renderer takes a raw string value (extracted by `ColumnDef::get_value`)
//! and produces an `Element` node for that cell.

use dioxus::prelude::*;

use super::types::*;

// ---------------------------------------------------------------------------
// Main render dispatch
// ---------------------------------------------------------------------------

/// Render a cell value according to the column's renderer configuration.
pub fn render_cell(value: &str, renderer: &CellRenderer) -> Element {
    match renderer {
        CellRenderer::Text => render_text(value),

        CellRenderer::Number { prefix, decimals } => {
            render_number(value, prefix, *decimals)
        }

        CellRenderer::Currency { code, decimals } => {
            render_currency(value, code, *decimals)
        }

        CellRenderer::Date { format } => {
            render_date(value, format)
        }

        CellRenderer::DateTime { format } => {
            render_datetime(value, format)
        }

        CellRenderer::Badge {
            color_map,
            default_color,
        } => render_badge(value, color_map, default_color),

        CellRenderer::Percentage { decimals } => {
            render_percentage(value, *decimals)
        }

        CellRenderer::Custom(f) => f(value),
    }
}

// ---------------------------------------------------------------------------
// Individual renderers
// ---------------------------------------------------------------------------

/// Plain text — simple span with the value.
fn render_text(value: &str) -> Element {
    rsx! {
        span { class: "dg-cell-text", "{value}" }
    }
}

/// Number with optional prefix and decimal formatting.
///
/// E.g. prefix="Rs. ", decimals=2 → "Rs. 1,234.57"
fn render_number(value: &str, prefix: &str, decimals: u8) -> Element {
    let formatted = if let Ok(num) = value.parse::<f64>() {
        format_number(num, decimals)
    } else {
        value.to_string()
    };

    rsx! {
        span { class: "dg-cell-number", "{prefix}{formatted}" }
    }
}

/// Currency — same as number but with the currency code indicator.
fn render_currency(value: &str, code: &str, decimals: u8) -> Element {
    let formatted = if let Ok(num) = value.parse::<f64>() {
        format_number(num, decimals)
    } else {
        value.to_string()
    };

    rsx! {
        span { class: "dg-cell-currency",
            span { class: "dg-cell-currency-code", "{code} " }
            span { class: "dg-cell-currency-value", "{formatted}" }
        }
    }
}

/// Date rendering using `chrono` format string.
fn render_date(value: &str, format: &str) -> Element {
    let display = format_date_str(value, format);
    rsx! {
        span { class: "dg-cell-date", "{display}" }
    }
}

/// DateTime rendering.
fn render_datetime(value: &str, format: &str) -> Element {
    let display = format_date_str(value, format);
    rsx! {
        span { class: "dg-cell-datetime", "{display}" }
    }
}

/// Badge / status pill with color mapping.
pub fn render_badge(
    value: &str,
    color_map: &[(&str, BadgeColor)],
    default_color: &BadgeColor,
) -> Element {
    let color = color_map
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(_, c)| *c)
        .unwrap_or(*default_color);

    let color_class = match color {
        BadgeColor::Gray => "dg-badge-gray",
        BadgeColor::Green => "dg-badge-green",
        BadgeColor::Red => "dg-badge-red",
        BadgeColor::Yellow => "dg-badge-yellow",
        BadgeColor::Blue => "dg-badge-blue",
        BadgeColor::Purple => "dg-badge-purple",
        BadgeColor::Cyan => "dg-badge-cyan",
    };

    rsx! {
        span { class: "dg-badge {color_class}", "{value}" }
    }
}

/// Percentage — multiply by 100, format, append "%".
fn render_percentage(value: &str, decimals: u8) -> Element {
    let display = if let Ok(num) = value.parse::<f64>() {
        let pct = num * 100.0;
        format!("{:.1$}%", pct, decimals as usize)
    } else {
        format!("{}%", value)
    };

    rsx! {
        span { class: "dg-cell-percentage", "{display}" }
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Format a number with thousands separator and fixed decimals.
fn format_number(num: f64, decimals: u8) -> String {
    let precision = decimals as usize;
    let rounded = format!("{:.prec$}", num, prec = precision);

    // Add thousands separator
    let mut parts: Vec<&str> = rounded.split('.').collect();
    let int_part = parts[0];
    let mut formatted = String::new();

    let chars: Vec<char> = int_part.chars().collect();
    let len = chars.len();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(*ch);
    }

    if parts.len() > 1 {
        formatted.push('.');
        formatted.push_str(parts[1]);
    }

    formatted
}

/// Attempt to parse and reformat a date string using the given `chrono` format.
fn format_date_str(value: &str, format: &str) -> String {
    // Try parsing as ISO date first (most common for API responses)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return date.format(format).to_string();
    }
    // Try ISO datetime
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return dt.format(format).to_string();
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return dt.format(format).to_string();
    }
    // Fallback: return as-is
    value.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234.5678, 2), "1,234.57");
        assert_eq!(format_number(1000.0, 0), "1,000");
        assert_eq!(format_number(0.0, 2), "0.00");
        assert_eq!(format_number(999.99, 2), "999.99");
        assert_eq!(format_number(1234567.89, 2), "1,234,567.89");
    }

    #[test]
    fn test_format_date_iso() {
        let result = format_date_str("2024-01-15", "%d-%b-%Y");
        assert_eq!(result, "15-Jan-2024");
    }

    #[test]
    fn test_format_date_fallback() {
        let result = format_date_str("not-a-date", "%Y-%m-%d");
        assert_eq!(result, "not-a-date");
    }

    #[test]
    fn test_render_badge_color_selection() {
        let map = vec![
            ("Paid", BadgeColor::Green),
            ("Unpaid", BadgeColor::Red),
        ];
        // We can't easily test RSX equality, but we can test the color logic:
        let color = map
            .iter()
            .find(|(key, _)| *key == "Paid")
            .map(|(_, c)| *c)
            .unwrap_or(BadgeColor::Gray);
        assert_eq!(color, BadgeColor::Green);

        let missing = map
            .iter()
            .find(|(key, _)| *key == "Unknown")
            .map(|(_, c)| *c)
            .unwrap_or(BadgeColor::Gray);
        assert_eq!(missing, BadgeColor::Gray);
    }
}
