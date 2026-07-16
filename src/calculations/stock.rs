//! Stock calculation functions.
//!
//! Pure functions for inventory costing (FIFO), stock valuation, and
//! reorder-level computations. These are the building blocks used by
//! `StockMovementService` and related business logic.

use crate::calculations::{round_money, BatchConsumption};

// ---------------------------------------------------------------------------
// FIFO Costing
// ---------------------------------------------------------------------------

/// A batch of inventory with tracking info (mirrors the `stock_batches` table).
#[derive(Debug, Clone)]
pub struct StockBatch {
    pub id: i64,
    pub quantity_remaining: f64,
    pub unit_cost: f64,
}

/// Result of a FIFO consumption across batches.
#[derive(Debug)]
pub struct FifoConsumptionResult {
    /// The individual batch slices consumed (oldest first).
    pub consumed: Vec<BatchConsumption>,
    /// The weighted-average unit cost of the total consumption.
    pub weighted_avg_cost: f64,
    /// Updated quantities for each input batch (same order as input).
    pub updated_batches: Vec<StockBatch>,
}

/// Consume a quantity from a list of batches using FIFO (oldest first).
///
/// Batches are assumed to already be sorted oldest-first (by received_date, then id).
/// Consumes from each batch in order until the requested quantity is fulfilled.
///
/// Returns the consumption records and updated batch quantities.
pub fn consume_fifo_batches(batches: &[StockBatch], qty_to_consume: f64) -> FifoConsumptionResult {
    let mut consumed = Vec::new();
    let mut updated_batches: Vec<StockBatch> = batches.to_vec();
    let mut remaining = qty_to_consume;

    for batch in &mut updated_batches {
        if remaining <= 0.0 {
            break;
        }
        let take = remaining.min(batch.quantity_remaining);
        if take > 0.0 {
            consumed.push(BatchConsumption {
                quantity: take,
                unit_cost: batch.unit_cost,
            });
            batch.quantity_remaining -= take;
            remaining -= take;
        }
    }

    let total_cost: f64 = consumed.iter().map(|c| c.cost()).sum();
    let total_qty: f64 = consumed.iter().map(|c| c.quantity).sum();
    let weighted_avg_cost = if total_qty > 0.0 { total_cost / total_qty } else { 0.0 };

    FifoConsumptionResult {
        consumed,
        weighted_avg_cost: round_money(weighted_avg_cost),
        updated_batches,
    }
}

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

    #[test]
    fn test_fifo_two_batches() {
        // Buy 1 at 100, buy 1 at 105. Sell 1 → should consume batch 1 (cost 100).
        let batches = vec![
            StockBatch { id: 1, quantity_remaining: 1.0, unit_cost: 100.0 },
            StockBatch { id: 2, quantity_remaining: 1.0, unit_cost: 105.0 },
        ];
        let result = consume_fifo_batches(&batches, 1.0);
        assert_eq!(result.consumed.len(), 1);
        assert!((result.consumed[0].quantity - 1.0).abs() < 0.01);
        assert!((result.consumed[0].unit_cost - 100.0).abs() < 0.01);
        assert!((result.weighted_avg_cost - 100.0).abs() < 0.01);
        // Batch 1 fully consumed, batch 2 untouched
        assert!((result.updated_batches[0].quantity_remaining).abs() < 0.01);
        assert!((result.updated_batches[1].quantity_remaining - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fifo_consume_across_batches() {
        // Batch 1: 10 units at 50. Batch 2: 5 units at 55. Consume 12.
        let batches = vec![
            StockBatch { id: 1, quantity_remaining: 10.0, unit_cost: 50.0 },
            StockBatch { id: 2, quantity_remaining: 5.0, unit_cost: 55.0 },
        ];
        let result = consume_fifo_batches(&batches, 12.0);
        assert_eq!(result.consumed.len(), 2);
        // First 10 from batch 1, next 2 from batch 2
        assert!((result.consumed[0].quantity - 10.0).abs() < 0.01);
        assert!((result.consumed[1].quantity - 2.0).abs() < 0.01);
        // Total cost = 500 + 110 = 610, avg = 610/12 = 50.83
        assert!((compute_fifo_cogs(&result.consumed) - 610.0).abs() < 0.01);
        assert!((result.weighted_avg_cost - 50.83).abs() < 0.01);
    }

    #[test]
    fn test_fifo_consume_more_than_available() {
        let batches = vec![
            StockBatch { id: 1, quantity_remaining: 3.0, unit_cost: 10.0 },
        ];
        let result = consume_fifo_batches(&batches, 5.0);
        // Only 3 available, consume all of them
        assert_eq!(result.consumed.len(), 1);
        assert!((result.consumed[0].quantity - 3.0).abs() < 0.01);
        assert!((result.updated_batches[0].quantity_remaining).abs() < 0.01);
    }

    #[test]
    fn test_fifo_no_batches() {
        let result = consume_fifo_batches(&[], 10.0);
        assert!(result.consumed.is_empty());
        assert!((result.weighted_avg_cost).abs() < 0.01);
    }
}
