//! Column filtering for the `DataGrid<T>`.
//!
//! Phase 2 introduces interactive column-level filtering with four filter types:
//!
//! - **Text:** Case-insensitive substring match
//! - **Number:** Inclusive range filter with optional min/max
//! - **Date:** Inclusive date range filter with optional from/to
//! - **Select:** Multi-select from a predefined set of options
//!
//! Each column defines its filter type via [`FilterType`] (in `types.rs`), and
//! the active filter value is stored as a [`FilterValue`]. The function
//! [`apply_filters`] composes all active filters into the data pipeline.

use std::collections::HashMap;

use super::types::*;

// ---------------------------------------------------------------------------
// Filter Value
// ---------------------------------------------------------------------------

/// The active value of a column filter.
///
/// This differs from [`FilterType`] which only declares what kind of filter
/// is allowed. `FilterValue` stores the *current user input* for that filter.
#[derive(Clone, PartialEq, Debug)]
pub enum FilterValue {
    /// No filter active (default state).
    None,

    /// Text filter — match rows containing this substring (case-insensitive).
    Text {
        /// The substring to search for.
        query: String,
    },

    /// Number range filter — rows where value is in `[min, max]`.
    ///
    /// Unbounded sides are represented as `None`.
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },

    /// Date range filter — rows where date is in `[from, to]`.
    Date {
        /// Inclusive lower bound (ISO date string "YYYY-MM-DD").
        from: Option<String>,
        /// Inclusive upper bound (ISO date string "YYYY-MM-DD").
        to: Option<String>,
    },

    /// Select filter — rows where value matches one of the selected options.
    Select {
        /// The currently selected option values.
        selected: Vec<String>,
    },
}

impl FilterValue {
    /// Returns `true` if this is the `None` variant.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Returns `true` if this filter is active (has a meaningful value).
    pub fn is_active(&self) -> bool {
        match self {
            Self::None => false,
            Self::Text { query } => !query.is_empty(),
            Self::Number { min, max } => min.is_some() || max.is_some(),
            Self::Date { from, to } => from.is_some() || to.is_some(),
            Self::Select { selected } => !selected.is_empty(),
        }
    }

    /// Clear this filter (return to `None`).
    pub fn clear(&mut self) {
        *self = Self::None;
    }
}

// ---------------------------------------------------------------------------
// Filter application
// ---------------------------------------------------------------------------

/// Apply all active column filters to a list of indexed rows.
///
/// A row passes if *all* non-none filters match (logical AND across columns).
/// Within a column, the filter match logic depends on the [`FilterValue`]
/// variant and the column's [`FilterType`].
pub fn apply_filters<T: Clone>(
    rows: Vec<IndexedRow<T>>,
    columns: &[ColumnDef<T>],
    filters: &HashMap<&'static str, FilterValue>,
) -> Vec<IndexedRow<T>> {
    // Fast path: no active filters
    if filters.is_empty() || filters.values().all(|f| f.is_none()) {
        return rows;
    }

    rows.into_iter()
        .filter(|row| passes_all_filters(&row.data, columns, filters))
        .collect()
}

/// Check whether a single row passes all active filters.
fn passes_all_filters<T: Clone>(
    row: &T,
    columns: &[ColumnDef<T>],
    filters: &HashMap<&'static str, FilterValue>,
) -> bool {
    for col in columns {
        let filter = match filters.get(col.key) {
            Some(f) => f,
            None => continue,
        };

        if !filter.is_active() {
            continue;
        }

        let cell_value = (col.get_value)(row);

        if !matches_filter(&cell_value, col, filter) {
            return false;
        }
    }

    true
}

/// Check whether a single cell value matches a single filter.
fn matches_filter(value: &str, col: &ColumnDef<impl Clone>, filter: &FilterValue) -> bool {
    match filter {
        FilterValue::None => true,

        FilterValue::Text { query } => {
            if query.is_empty() {
                return true;
            }
            value.to_lowercase().contains(&query.to_lowercase())
        }

        FilterValue::Number { min, max } => {
            match parse_number(value, &col.renderer) {
                Some(num) => {
                    let above_min = min.map_or(true, |m| num >= m);
                    let below_max = max.map_or(true, |m| num <= m);
                    above_min && below_max
                }
                None => {
                    // Can't parse as number — only pass if no bounds are set
                    min.is_none() && max.is_none()
                }
            }
        }

        FilterValue::Date { from, to } => {
            let cell_date = normalize_date(value);
            match cell_date {
                Some(d) => {
                    let after_from = from.as_ref().map_or(true, |f| d >= *f);
                    let before_to = to.as_ref().map_or(true, |t| d <= *t);
                    after_from && before_to
                }
                None => {
                    // Can't parse as date — only pass if no bounds are set
                    from.is_none() && to.is_none()
                }
            }
        }

        FilterValue::Select { selected } => {
            if selected.is_empty() {
                return true;
            }
            selected.iter().any(|s| s == value)
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Try to parse a cell value as a number, respecting the column's renderer.
fn parse_number(value: &str, renderer: &CellRenderer) -> Option<f64> {
    // Strip common currency prefixes/suffixes
    let cleaned = match renderer {
        CellRenderer::Currency { code, .. } => {
            value.replace(code, "").trim().to_string()
        }
        CellRenderer::Number { prefix, .. } => {
            value.replace(prefix, "").trim().to_string()
        }
        CellRenderer::Percentage { .. } => {
            value.replace('%', "").trim().to_string()
        }
        _ => value.to_string(),
    };

    // Remove thousands separators
    let cleaned = cleaned.replace(',', "");

    cleaned.parse::<f64>().ok()
}

/// Normalize a date string to ISO `YYYY-MM-DD` format for comparison.
fn normalize_date(value: &str) -> Option<String> {
    let trimmed = value.trim();

    // Already ISO
    if trimmed.len() == 10 && trimmed.chars().filter(|c| *c == '-').count() == 2 {
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() == 3 && parts[0].len() == 4 {
            return Some(trimmed.to_string());
        }
    }

    // Try parsing DD-Mon-YYYY (e.g., "15-Jan-2024")
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d-%b-%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    // Try DD/MM/YYYY
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    // Try MM/DD/YYYY
    if let Ok(d) = chrono::NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }

    None
}

// ---------------------------------------------------------------------------
// Filter count (for showing active filter count badge)
// ---------------------------------------------------------------------------

/// Count the number of active filters.
pub fn count_active_filters(filters: &HashMap<&'static str, FilterValue>) -> usize {
    filters.values().filter(|f| f.is_active()).count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn text_col(key: &'static str) -> ColumnDef<&'static str> {
        ColumnDef::text(key, key, |s| s.to_string())
    }

    fn row(value: &'static str, index: usize) -> IndexedRow<&'static str> {
        IndexedRow { index, data: value }
    }

    #[test]
    fn test_text_filter_matches() {
        let col = text_col("name");
        let filter = FilterValue::Text { query: "apple".to_string() };
        assert!(matches_filter("Apple Pie", &col, &filter));
        assert!(matches_filter("apple", &col, &filter));
        assert!(!matches_filter("banana", &col, &filter));
    }

    #[test]
    fn test_text_filter_empty_query() {
        let col = text_col("name");
        let filter = FilterValue::Text { query: String::new() };
        assert!(matches_filter("anything", &col, &filter));
    }

    #[test]
    fn test_number_filter_range() {
        let col = text_col("price").with_renderer(CellRenderer::Number { prefix: "$", decimals: 2 });
        let filter = FilterValue::Number { min: Some(10.0), max: Some(100.0) };
        assert!(matches_filter("50", &col, &filter));
        assert!(matches_filter("10", &col, &filter));
        assert!(matches_filter("100", &col, &filter));
        assert!(!matches_filter("5", &col, &filter));
        assert!(!matches_filter("200", &col, &filter));
    }

    #[test]
    fn test_number_filter_min_only() {
        let col = text_col("price");
        let filter = FilterValue::Number { min: Some(50.0), max: None };
        assert!(matches_filter("100", &col, &filter));
        assert!(!matches_filter("25", &col, &filter));
    }

    #[test]
    fn test_select_filter() {
        let col = text_col("status");
        let filter = FilterValue::Select { selected: vec!["Active".to_string(), "Pending".to_string()] };
        assert!(matches_filter("Active", &col, &filter));
        assert!(matches_filter("Pending", &col, &filter));
        assert!(!matches_filter("Inactive", &col, &filter));
    }

    #[test]
    fn test_select_filter_empty() {
        let col = text_col("status");
        let filter = FilterValue::Select { selected: vec![] };
        assert!(matches_filter("Anything", &col, &filter));
    }

    #[test]
    fn test_date_filter_iso() {
        let col = text_col("date");
        let filter = FilterValue::Date {
            from: Some("2024-01-01".to_string()),
            to: Some("2024-12-31".to_string()),
        };
        assert!(matches_filter("2024-06-15", &col, &filter));
        assert!(!matches_filter("2023-12-31", &col, &filter));
        assert!(!matches_filter("2025-01-01", &col, &filter));
    }

    #[test]
    fn test_date_filter_formatted() {
        let col = text_col("date");
        let filter = FilterValue::Date {
            from: Some("2024-01-01".to_string()),
            to: Some("2024-12-31".to_string()),
        };
        assert!(matches_filter("15-Jan-2024", &col, &filter));
        assert!(!matches_filter("15-Jan-2023", &col, &filter));
    }

    #[test]
    fn test_apply_filters_all_pass() {
        let col = text_col("name");
        let rows = vec![row("apple", 0), row("banana", 1)];
        let filters: HashMap<&'static str, FilterValue> = HashMap::new();
        let result = apply_filters(rows, &[col], &filters);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_apply_filters_some_pass() {
        let col = text_col("name");
        let rows = vec![row("apple pie", 0), row("banana split", 1), row("appletini", 2)];
        let mut filters = HashMap::new();
        filters.insert("name", FilterValue::Text { query: "apple".to_string() });
        let result = apply_filters(rows, &[col], &filters);
        assert_eq!(result.len(), 2); // "apple pie" and "appletini"
        assert_eq!(result[0].index, 0);
        assert_eq!(result[1].index, 2);
    }

    #[test]
    fn test_count_active_filters() {
        let mut filters: HashMap<&'static str, FilterValue> = HashMap::new();
        assert_eq!(count_active_filters(&filters), 0);

        filters.insert("a", FilterValue::Text { query: "".to_string() });
        assert_eq!(count_active_filters(&filters), 0);

        filters.insert("a", FilterValue::Text { query: "hello".to_string() });
        assert_eq!(count_active_filters(&filters), 1);

        filters.insert("b", FilterValue::Number { min: Some(5.0), max: None });
        assert_eq!(count_active_filters(&filters), 2);
    }

    #[test]
    fn test_parse_number_with_currency() {
        let renderer = CellRenderer::Currency { code: "USD", decimals: 2 };
        assert_eq!(parse_number("USD 1,234.56", &renderer), Some(1234.56));
    }

    #[test]
    fn test_normalize_date_iso() {
        assert_eq!(normalize_date("2024-06-15"), Some("2024-06-15".to_string()));
    }

    #[test]
    fn test_normalize_date_formatted() {
        assert_eq!(normalize_date("15-Jan-2024"), Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_filter_value_is_active() {
        assert!(!FilterValue::None.is_active());
        assert!(!FilterValue::Text { query: String::new() }.is_active());
        assert!(FilterValue::Text { query: "a".to_string() }.is_active());
        assert!(!FilterValue::Select { selected: vec![] }.is_active());
        assert!(FilterValue::Select { selected: vec!["x".to_string()] }.is_active());
    }
}
