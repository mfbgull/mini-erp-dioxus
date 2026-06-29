//! Pure, side-effect-free calculation functions for MiniERP.
//!
//! This module contains ~40 synchronous functions ported from the original
//! TypeScript codebase. Each function is deterministic, has no I/O, and
//! operates only on its arguments.
//!
//! # Sub-modules
//!
//! | Module | Contents | Source (TS) |
//! |--------|----------|-------------|
//! | [`invoice`] | Discounts, taxes, totals, metrics | `invoiceCalculations.ts` |
//! | [`quotation`] | Quotation & Sales Order math | `quotationCalculations.ts` |
//! | [`customer`] | Ledger totals, AR aging, credit metrics | `customerCalculations.ts` |
//! | [`stock`] | FIFO COGS, stock valuation, reorder Qty | (inline) |
//! | [`formatting`] | Currency, date, percent, phone formatters | `formatters.ts` |

pub mod invoice;
pub mod quotation;
pub mod customer;
pub mod stock;
pub mod formatting;

/// Re-export the most commonly used functions at the `calculations` level
/// for ergonomic imports.
pub use formatting::{format_currency, format_date, format_percent};
pub use invoice::{calculate_total, compute_invoice_metrics};
pub use customer::calculate_credit_utilization;
pub use stock::compute_fifo_cogs;

use chrono::NaiveDate;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Round a monetary amount to 2 decimal places using banker's rounding.
pub(crate) fn round_money(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Round to `decimals` places.
pub(crate) fn round_to(v: f64, decimals: u8) -> f64 {
    let factor = 10_f64.powi(decimals as i32);
    (v * factor).round() / factor
}

/// Parse `"YYYY-MM-DD"` into `NaiveDate`, panicking on invalid input.
/// Production code should handle `Option` instead.
pub(crate) fn parse_date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .expect("calculations: invalid date format — expected YYYY-MM-DD")
}

// ---------------------------------------------------------------------------
// Shared data types
// ---------------------------------------------------------------------------

/// Discount configuration used by invoice and quotation calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Discount {
    pub scope: DiscountScope,
    pub r#type: DiscountType,
    pub value: f64,
}

impl Default for Discount {
    fn default() -> Self {
        Self {
            scope: DiscountScope::AfterTax,
            r#type: DiscountType::Percentage,
            value: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscountScope {
    /// Apply discount before computing tax.
    BeforeTax,
    /// Apply discount after computing tax.
    AfterTax,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiscountType {
    /// `value` is a percentage (0.0 – 100.0).
    Percentage,
    /// `value` is a flat amount.
    Flat,
}

/// Aggregated invoice/quote metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvoiceMetrics {
    pub subtotal: f64,
    pub discount_amount: f64,
    pub taxable_amount: f64,
    pub tax_amount: f64,
    pub total: f64,
}

/// AR aging buckets as-of a given date.
#[derive(Debug, Clone, PartialEq)]
pub struct AgingBuckets {
    pub current: f64,
    pub days_1_30: f64,
    pub days_31_60: f64,
    pub days_61_90: f64,
    pub days_90_plus: f64,
    pub total: f64,
}

impl AgingBuckets {
    pub fn empty() -> Self {
        Self {
            current: 0.0,
            days_1_30: 0.0,
            days_31_60: 0.0,
            days_61_90: 0.0,
            days_90_plus: 0.0,
            total: 0.0,
        }
    }
}

/// A single batch consumption record (from FIFO).
#[derive(Debug, Clone, Copy)]
pub struct BatchConsumption {
    pub quantity: f64,
    pub unit_cost: f64,
}

impl BatchConsumption {
    pub fn cost(&self) -> f64 {
        self.quantity * self.unit_cost
    }
}

/// Aggregated customer metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomerMetrics {
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub total_outstanding: f64,
    pub credit_limit: f64,
    pub credit_utilization: f64,
    pub overdue_invoices: usize,
    pub average_days_to_pay: f64,
    pub current_balance: f64,
}

/// Minimal invoice slice used by AR aging / customer calculations.
#[derive(Debug, Clone)]
pub struct InvoiceSummary {
    pub id: i64,
    pub due_date: NaiveDate,
    pub balance_amount: f64,
    pub total_amount: f64,
}

/// Minimal ledger entry for customer calculations.
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub debit: f64,
    pub credit: f64,
}

/// Minimal customer profile used by metrics computation.
#[derive(Debug, Clone)]
pub struct CustomerProfile {
    pub credit_limit: f64,
    pub current_balance: f64,
}
