//! Stock calculation functions.
//!
//! Pure functions for inventory costing (FIFO), stock valuation, and
//! reorder-level computations. These are the building blocks used by
//! `StockMovementService` and related business logic.

use crate::calculations::{round_money, BatchConsumption};

// ---------------------------------------------------------------------------
// FIFO Costing
// ---------------------------------------------------------------------------

/// Compute the total cost of goods sold from a set of batch consumption records.
///
/// Each [`BatchConsumption`] represents a quantity taken from an oldest-first
/// FIFO batch. This function simply sums `SUM(quantity × unit_cost)`.
///
/// # Examples
///
/// ```ignore
/// let consumed = vec![
///     BatchConsumption { quantity: 10.0, unit_cost: 50.0 },   // cost = 500
///     BatchConsumption { quantity: 5.0, unit_cost: 55.0 },    // cost = 275
/// ];
/// assert_eq!(compute_fifo_cogs(&consumed), 775.0);
/// ```
pub fn compute_fifo_cogs(consumptions: &[BatchConsumption]) -> f64 {
    round_money(consumptions.iter().map(|c| c.cost()).sum())
}

/// Calculate the total stock value across multiple items.
///
/// For each item, the value is `balance.quantity × cost[item_id]`.
///
/// # Arguments
///
/// * `balances` — Slice of `(item_id, quantity)` tuples representing current stock levels.
/// * `costs` — A function that returns the unit cost for a given item ID.
///
/// # Example
///
/// ```ignore
/// let balances = vec![(1, 100.0), (2, 50.0)];
/// let costs = |id: i64| match id { 1 => 10.0, 2 => 20.0, _ => 0.0 };
/// let value = compute_stock_value_fn(&balances, &costs);
/// // Item 1: 100 × 10 = 1000
/// // Item 2: 50  × 20 = 1000
/// // Total: 2000
/// ```
pub fn compute_stock_value_fn(balances: &[(i64, f64)], costs: &dyn Fn(i64) -> f64) -> f64 {
    round_money(balances.iter().map(|(id, qty)| qty * costs(*id)).sum())
}

/// Calculate the total stock value using a pre-built cost map.
///
/// Convenience wrapper around [`compute_stock_value_fn`] when costs are
/// already collected into a `HashMap`.
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// let balances = vec![(1, 100.0), (2, 50.0)];
/// let costs = HashMap::from([(1, 10.0), (2, 20.0)]);
/// assert_eq!(compute_stock_value(&balances, &costs), 2000.0);
/// ```
pub fn compute_stock_value(balances: &[(i64, f64)], costs: &std::collections::HashMap<i64, f64>) -> f64 {
    compute_stock_value_fn(balances, &|id| costs.get(&id).copied().unwrap_or(0.0))
}

// ---------------------------------------------------------------------------
// Reorder calculations
// ---------------------------------------------------------------------------

/// Calculate the recommended reorder quantity.
///
/// Formula: `(daily_usage × lead_time_days) + safety_stock - current_stock`
///
/// If the result is negative (current stock is sufficient), returns `0.0`.
///
/// # Arguments
///
/// * `daily_usage` — Average units consumed per day.
/// * `lead_time_days` — Number of days from order placement to receipt.
/// * `safety_stock` — Buffer stock to maintain.
/// * `current_stock` — Current on-hand quantity.
///
/// # Examples
///
/// ```ignore
/// // Usage: 10/day, lead time: 7 days, safety: 20, current: 50
/// let reorder = calculate_reorder_quantity(10.0, 7.0, 20.0, 50.0);
/// // = (10 × 7) + 20 - 50 = 40
/// assert_eq!(reorder, 40.0);
///
/// // Sufficient stock: should return 0
/// let reorder = calculate_reorder_quantity(10.0, 7.0, 20.0, 100.0);
/// // = 70 + 20 - 100 = -10 → 0
/// assert_eq!(reorder, 0.0);
/// ```
pub fn calculate_reorder_quantity(
    daily_usage: f64,
    lead_time_days: f64,
    safety_stock: f64,
    current_stock: f64,
) -> f64 {
    let reorder = (daily_usage * lead_time_days) + safety_stock - current_stock;
    reorder.max(0.0)
}

/// Determine if an item needs reordering (below or at reorder level).
///
/// Returns `true` if `current_stock <= reorder_level`.
pub fn is_low_stock(current_stock: f64, reorder_level: f64) -> bool {
    current_stock <= reorder_level
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fifo_cogs() {
        let consumed = vec![
            BatchConsumption { quantity: 10.0, unit_cost: 50.0 },
            BatchConsumption { quantity: 5.0, unit_cost: 55.0 },
        ];
        assert!((compute_fifo_cogs(&consumed) - 775.0).abs() < 0.01);
    }

    #[test]
    fn test_fifo_cogs_empty() {
        assert_eq!(compute_fifo_cogs(&[]), 0.0);
    }

    #[test]
    fn test_stock_value_fn() {
        let balances = vec![(1, 100.0), (2, 50.0)];
        let costs = |id: i64| match id {
            1 => 10.0,
            2 => 20.0,
            _ => 0.0,
        };
        assert!((compute_stock_value_fn(&balances, &costs) - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_stock_value_map() {
        let balances = vec![(1, 100.0), (2, 50.0)];
        let costs = HashMap::from([(1, 10.0), (2, 20.0)]);
        assert!((compute_stock_value(&balances, &costs) - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_stock_value_missing_item() {
        let balances = vec![(1, 100.0), (99, 50.0)]; // item 99 not in map
        let costs = HashMap::from([(1, 10.0)]);
        assert!((compute_stock_value(&balances, &costs) - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_reorder_quantity() {
        let qty = calculate_reorder_quantity(10.0, 7.0, 20.0, 50.0);
        assert!((qty - 40.0).abs() < 0.01);
    }

    #[test]
    fn test_reorder_quantity_sufficient() {
        let qty = calculate_reorder_quantity(10.0, 7.0, 20.0, 100.0);
        assert_eq!(qty, 0.0);
    }

    #[test]
    fn test_is_low_stock() {
        assert!(is_low_stock(5.0, 10.0));
        assert!(!is_low_stock(15.0, 10.0));
        assert!(is_low_stock(10.0, 10.0)); // exactly at reorder level
    }
}
