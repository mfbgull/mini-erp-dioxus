//! Client-side sorting logic for the `DataGrid<T>`.
//!
//! Supports multi-column sorting: each click adds or toggles the column in
//! the sort list. Shift+click appends a secondary sort column.

use std::cmp::Ordering;

use super::types::*;

// ---------------------------------------------------------------------------
// Sort state management
// ---------------------------------------------------------------------------

/// Handle a header click and produce the new sort state.
///
/// - **Normal click:** If the column is already the primary sort, toggle its
///   direction. Otherwise, replace the sort list with this column ascending.
/// - **Shift+click:** Append the column as a secondary sort. If already
///   present, toggle its direction.
pub fn handle_sort_click(
    column_key: &'static str,
    current_sort: &[SortColumn],
    is_shift: bool,
) -> Vec<SortColumn> {
    if is_shift {
        // Shift-click: add secondary sort
        let mut new_sort = current_sort.to_vec();
        if let Some(pos) = new_sort.iter().position(|s| s.key == column_key) {
            // Toggle direction for existing column
            new_sort[pos].direction = new_sort[pos].direction.toggle();
        } else {
            // Append new sort column
            new_sort.push(SortColumn::ascending(column_key));
        }
        new_sort
    } else {
        // Normal click: primary sort
        if let Some(existing) = current_sort.first() {
            if existing.key == column_key {
                // Toggle direction
                vec![SortColumn {
                    key: column_key,
                    direction: existing.direction.toggle(),
                }]
            } else {
                // Replace
                vec![SortColumn::ascending(column_key)]
            }
        } else {
            vec![SortColumn::ascending(column_key)]
        }
    }
}

// ---------------------------------------------------------------------------
// Sort application
// ---------------------------------------------------------------------------

/// Apply the active sort columns to a slice of indexed rows, returning a
/// sorted `Vec<IndexedRow<T>>`.
///
/// Multi-column sorting applies sort columns in order (primary, secondary, …),
/// only using the next column when the previous columns compare as equal.
pub fn apply_sort<T>(
    rows: Vec<IndexedRow<T>>,
    columns: &[ColumnDef<T>],
    sort: &[SortColumn],
) -> Vec<IndexedRow<T>>
where
    T: Clone,
{
    if sort.is_empty() {
        return rows;
    }

    // Build a lookup from column key → get_value closure for fast access
    let column_map: std::collections::HashMap<&'static str, &ColumnDef<T>> = columns
        .iter()
        .map(|c| (c.key, c))
        .collect();

    let mut sorted = rows;

    sorted.sort_by(|a, b| {
        let mut ordering = Ordering::Equal;

        for sort_col in sort {
            if ordering != Ordering::Equal {
                break; // primary sort already decided
            }

            if let Some(col) = column_map.get(sort_col.key) {
                let a_val = (col.get_value)(&a.data);
                let b_val = (col.get_value)(&b.data);

                ordering = compare_values(&a_val, &b_val);

                if sort_col.direction == SortDirection::Descending {
                    ordering = ordering.reverse();
                }
            }
        }

        ordering
    });

    sorted
}

/// Compare two string values for sorting. Handles numeric-aware comparison
/// where possible so "10" > "2" instead of lexicographic.
fn compare_values(a: &str, b: &str) -> Ordering {
    // Try numeric comparison first
    if let (Ok(a_num), Ok(b_num)) = (a.parse::<f64>(), b.parse::<f64>()) {
        return a_num.partial_cmp(&b_num).unwrap_or(Ordering::Equal);
    }

    // Try integer comparison
    if let (Ok(a_int), Ok(b_int)) = (a.parse::<i64>(), b.parse::<i64>()) {
        return a_int.cmp(&b_int);
    }

    // Fall back to case-insensitive lexicographic
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    a_lower.cmp(&b_lower)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_sort_click_toggle() {
        let sort = vec![SortColumn::ascending("name")];
        let result = handle_sort_click("name", &sort, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].direction, SortDirection::Descending);
    }

    #[test]
    fn test_handle_sort_click_replace() {
        let sort = vec![SortColumn::ascending("name")];
        let result = handle_sort_click("date", &sort, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key, "date");
        assert_eq!(result[0].direction, SortDirection::Ascending);
    }

    #[test]
    fn test_handle_sort_click_shift_append() {
        let sort = vec![SortColumn::ascending("name")];
        let result = handle_sort_click("date", &sort, true);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, "name");
        assert_eq!(result[1].key, "date");
    }

    #[test]
    fn test_compare_values_numeric() {
        assert_eq!(compare_values("10", "2"), Ordering::Greater);
        assert_eq!(compare_values("5", "5"), Ordering::Equal);
        assert_eq!(compare_values("1", "100"), Ordering::Less);
    }

    #[test]
    fn test_compare_values_text() {
        assert_eq!(compare_values("apple", "banana"), Ordering::Less);
        assert_eq!(compare_values("banana", "apple"), Ordering::Greater);
        assert_eq!(compare_values("cat", "cat"), Ordering::Equal);
    }

    #[test]
    fn test_apply_sort_empty() {
        let rows = vec![
            IndexedRow { index: 0, data: "b" },
            IndexedRow { index: 1, data: "a" },
        ];
        let sorted = apply_sort(rows.clone(), &[], &[]);
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].data, "b");
    }

    #[test]
    fn test_apply_sort_ascending() {
        let col = ColumnDef::<&str>::text("name", "Name", |s| s.to_string());
        let rows = vec![
            IndexedRow { index: 0, data: "zebra" },
            IndexedRow { index: 1, data: "apple" },
            IndexedRow { index: 2, data: "banana" },
        ];
        let sort = vec![SortColumn::ascending("name")];
        let sorted = apply_sort(rows, &[col], &sort);
        assert_eq!(sorted[0].data, "apple");
        assert_eq!(sorted[1].data, "banana");
        assert_eq!(sorted[2].data, "zebra");
    }
}
