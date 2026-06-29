//! Customer calculation functions.
//!
//! Ported from the original `customerCalculations.ts` — pure functions for
//! customer ledger analysis, AR aging, credit metrics, and overdue detection.

use chrono::NaiveDate;

use crate::calculations::{
    round_money, AgingBuckets, CustomerMetrics, CustomerProfile, InvoiceSummary, LedgerEntry,
};

// ---------------------------------------------------------------------------
// Ledger calculations
// ---------------------------------------------------------------------------

/// Compute the total debit, total credit, and running balance from ledger entries.
///
/// The balance is computed as `sum(debits) - sum(credits)`.
///
/// # Returns
///
/// `(total_debit, total_credit, balance)`
pub fn calculate_ledger_totals(entries: &[LedgerEntry]) -> (f64, f64, f64) {
    let total_debit: f64 = entries.iter().map(|e| e.debit).sum();
    let total_credit: f64 = entries.iter().map(|e| e.credit).sum();
    let balance = round_money(total_debit - total_credit);
    (round_money(total_debit), round_money(total_credit), balance)
}

/// Calculate the current running balance from ledger entries.
///
/// Equivalent to `calculate_ledger_totals(entries).2`.
pub fn calculate_current_balance(entries: &[LedgerEntry]) -> f64 {
    calculate_ledger_totals(entries).2
}

/// Calculate the total invoiced amount from ledger entries.
///
/// This sums the debit amounts, which represent invoices raised.
pub fn calculate_total_invoiced(entries: &[LedgerEntry]) -> f64 {
    round_money(entries.iter().map(|e| e.debit).sum())
}

/// Calculate the total paid amount from ledger entries.
///
/// This sums the credit amounts, which represent payments received.
pub fn calculate_total_paid(entries: &[LedgerEntry]) -> f64 {
    round_money(entries.iter().map(|e| e.credit).sum())
}

// ---------------------------------------------------------------------------
// Outstanding & credit
// ---------------------------------------------------------------------------

/// Calculate total outstanding balance from a list of invoices.
///
/// Sums `balance_amount` for all invoices where `balance_amount > 0`.
pub fn calculate_total_outstanding(invoices: &[InvoiceSummary]) -> f64 {
    round_money(
        invoices
            .iter()
            .filter(|inv| inv.balance_amount > 0.01)
            .map(|inv| inv.balance_amount)
            .sum(),
    )
}

/// Calculate credit utilization as a ratio (0.0 – 1.0+).
///
/// Returns `0.0` if credit limit is zero or negative.
pub fn calculate_credit_utilization(balance: f64, credit_limit: f64) -> f64 {
    if credit_limit > 0.0 {
        round_money(balance / credit_limit)
    } else {
        0.0
    }
}

/// Calculate credit utilization as a percentage string.
pub fn calculate_credit_utilization_percent(balance: f64, credit_limit: f64) -> String {
    let ratio = calculate_credit_utilization(balance, credit_limit);
    format!("{:.1}%", ratio * 100.0)
}

// ---------------------------------------------------------------------------
// Overdue detection
// ---------------------------------------------------------------------------

/// Filter invoices that are overdue as of a given date.
///
/// An invoice is considered overdue if its `due_date` is before `as_of_date`
/// and its `balance_amount > 0`.
pub fn calculate_overdue_invoices(invoices: &[InvoiceSummary], as_of_date: NaiveDate) -> Vec<InvoiceSummary> {
    invoices
        .iter()
        .filter(|inv| inv.due_date < as_of_date && inv.balance_amount > 0.01)
        .cloned()
        .collect()
}

/// Calculate the weighted-average days to pay from ledger entries.
///
/// This approximates DPO by looking at the time between debit (invoice) and
/// subsequent credit (payment). Without date data on each entry, we return
/// a simplified estimate: assumes a 30-day average payment cycle.
///
/// For production, pass actual payment-against-invoice date pairs and compute
/// `(payment_date - invoice_date)` per invoice, then take the weighted average.
pub fn calculate_average_days_to_pay(invoices: &[InvoiceSummary]) -> f64 {
    let paid_count = invoices.iter().filter(|inv| inv.balance_amount < 0.01).count();
    if paid_count == 0 {
        return 0.0;
    }
    // TODO: Replace with actual date-based calculation using payment allocations
    // In production, compute: SUM(payment_date - invoice_date) / paid_count
    30.0
}

// ---------------------------------------------------------------------------
// AR Aging
// ---------------------------------------------------------------------------

/// Compute AR aging buckets from a list of invoices as-of a given date.
///
/// Buckets:
/// - **Current**: `due_date >= as_of_date` (not yet due)
/// - **1–30 days**: `due_date` 1–30 days before `as_of_date`
/// - **31–60 days**: 31–60 days overdue
/// - **61–90 days**: 61–90 days overdue
/// - **90+ days**: more than 90 days overdue
///
/// Only invoices with `balance_amount > 0` are included.
///
/// # Example
///
/// ```ignore
/// let invoices = vec![
///     InvoiceSummary { id: 1, due_date: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(), balance_amount: 500.0, total_amount: 500.0 },
///     InvoiceSummary { id: 2, due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), balance_amount: 300.0, total_amount: 300.0 },
/// ];
/// let as_of = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
/// let buckets = calculate_aging_buckets(&invoices, as_of);
/// assert!(buckets.current > 0.0);
/// assert!(buckets.days_31_60 > 0.0);
/// ```
pub fn calculate_aging_buckets(invoices: &[InvoiceSummary], as_of_date: NaiveDate) -> AgingBuckets {
    let mut buckets = AgingBuckets::empty();

    for inv in invoices {
        if inv.balance_amount <= 0.01 {
            continue;
        }
        let days_past_due = (as_of_date - inv.due_date).num_days();

        match days_past_due {
            d if d <= 0 => buckets.current += inv.balance_amount,
            1..=30 => buckets.days_1_30 += inv.balance_amount,
            31..=60 => buckets.days_31_60 += inv.balance_amount,
            61..=90 => buckets.days_61_90 += inv.balance_amount,
            _ => buckets.days_90_plus += inv.balance_amount,
        }
    }

    buckets.current = round_money(buckets.current);
    buckets.days_1_30 = round_money(buckets.days_1_30);
    buckets.days_31_60 = round_money(buckets.days_31_60);
    buckets.days_61_90 = round_money(buckets.days_61_90);
    buckets.days_90_plus = round_money(buckets.days_90_plus);
    buckets.total = round_money(
        buckets.current
            + buckets.days_1_30
            + buckets.days_31_60
            + buckets.days_61_90
            + buckets.days_90_plus,
    );

    buckets
}

/// Calculate Days Sales Outstanding (DSO).
///
/// Formula: `(receivables / credit_sales) × days`
///
/// Returns `0.0` if `credit_sales` is zero.
pub fn calculate_dso(receivables: f64, credit_sales: f64, days: i64) -> f64 {
    if credit_sales.abs() < 0.01 {
        return 0.0;
    }
    round_money((receivables / credit_sales) * days as f64)
}

// ---------------------------------------------------------------------------
// Computed metrics
// ---------------------------------------------------------------------------

/// Compute all customer-level metrics in one call.
///
/// Aggregates ledger entries, invoices, and payments into a single
/// [`CustomerMetrics`] struct for dashboard or detail-page display.
///
/// The caller provides `as_of_date` for AR aging calculations (avoids
/// implicit system-clock side effects in this pure function).
pub fn compute_customer_metrics(
    customer: &CustomerProfile,
    entries: &[LedgerEntry],
    invoices: &[InvoiceSummary],
    as_of_date: NaiveDate,
) -> CustomerMetrics {
    let (total_invoiced, total_paid, _balance) = calculate_ledger_totals(entries);
    let total_outstanding = calculate_total_outstanding(invoices);
    let overdue = calculate_overdue_invoices(invoices, as_of_date);
    let credit_utilization = calculate_credit_utilization(customer.current_balance, customer.credit_limit);
    let avg_days = calculate_average_days_to_pay(invoices);

    CustomerMetrics {
        total_invoiced,
        total_paid,
        total_outstanding,
        credit_limit: customer.credit_limit,
        credit_utilization,
        overdue_invoices: overdue.len(),
        average_days_to_pay: avg_days,
        current_balance: customer.current_balance,
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
    fn test_ledger_totals() {
        let entries = vec![
            LedgerEntry { debit: 1000.0, credit: 0.0 },
            LedgerEntry { debit: 0.0, credit: 500.0 },
            LedgerEntry { debit: 200.0, credit: 0.0 },
        ];
        let (d, c, b) = calculate_ledger_totals(&entries);
        assert!((d - 1200.0).abs() < 0.01);
        assert!((c - 500.0).abs() < 0.01);
        assert!((b - 700.0).abs() < 0.01);
    }

    #[test]
    fn test_current_balance() {
        let entries = vec![
            LedgerEntry { debit: 500.0, credit: 0.0 },
            LedgerEntry { debit: 0.0, credit: 300.0 },
        ];
        assert!((calculate_current_balance(&entries) - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_outstanding() {
        let invoices = vec![
            InvoiceSummary { id: 1, due_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(), balance_amount: 100.0, total_amount: 500.0 },
            InvoiceSummary { id: 2, due_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(), balance_amount: 0.0, total_amount: 300.0 },
            InvoiceSummary { id: 3, due_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(), balance_amount: 50.0, total_amount: 200.0 },
        ];
        assert!((calculate_total_outstanding(&invoices) - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_utilization() {
        assert!((calculate_credit_utilization(250_000.0, 500_000.0) - 0.5).abs() < 0.01);
        assert!((calculate_credit_utilization(500_000.0, 0.0) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_overdue_invoices() {
        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let invoices = vec![
            InvoiceSummary { id: 1, due_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(), balance_amount: 100.0, total_amount: 500.0 },
            InvoiceSummary { id: 2, due_date: NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(), balance_amount: 50.0, total_amount: 200.0 }, // not overdue
            InvoiceSummary { id: 3, due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), balance_amount: 0.0, total_amount: 300.0 }, // paid
        ];
        let overdue = calculate_overdue_invoices(&invoices, as_of);
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0].id, 1);
    }

    #[test]
    fn test_ledger_invoiced() {
        let entries = vec![
            LedgerEntry { debit: 1000.0, credit: 0.0 },
            LedgerEntry { debit: 500.0, credit: 0.0 },
            LedgerEntry { debit: 0.0, credit: 300.0 },
        ];
        assert!((calculate_total_invoiced(&entries) - 1500.0).abs() < 0.01);
    }

    #[test]
    fn test_ledger_paid() {
        let entries = vec![
            LedgerEntry { debit: 0.0, credit: 500.0 },
            LedgerEntry { debit: 0.0, credit: 200.0 },
        ];
        assert!((calculate_total_paid(&entries) - 700.0).abs() < 0.01);
    }

    #[test]
    fn test_credit_utilization_percent() {
        let pct = calculate_credit_utilization_percent(250_000.0, 500_000.0);
        assert_eq!(pct, "50.0%");
    }

    #[test]
    fn test_compute_customer_metrics() {
        let customer = CustomerProfile {
            credit_limit: 500_000.0,
            current_balance: 125_000.0,
        };
        let entries = vec![
            LedgerEntry { debit: 200_000.0, credit: 0.0 },
            LedgerEntry { debit: 0.0, credit: 75_000.0 },
        ];
        let invoices = vec![];
        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let m = compute_customer_metrics(&customer, &entries, &invoices, as_of);
        assert!((m.total_invoiced - 200_000.0).abs() < 0.01);
        assert!((m.total_paid - 75_000.0).abs() < 0.01);
    }

    #[test]
    fn test_aging_buckets() {
        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let invoices = vec![
            InvoiceSummary { id: 1, due_date: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(), balance_amount: 100.0, total_amount: 100.0 },  // current
            InvoiceSummary { id: 2, due_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(), balance_amount: 200.0, total_amount: 200.0 },  // 1-30 days
            InvoiceSummary { id: 3, due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(), balance_amount: 300.0, total_amount: 300.0 },   // 61-90 days
            InvoiceSummary { id: 4, due_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(), balance_amount: 400.0, total_amount: 400.0 },   // 90+ days
        ];
        let buckets = calculate_aging_buckets(&invoices, as_of);
        assert!((buckets.current - 100.0).abs() < 0.01);
        assert!((buckets.days_1_30 - 200.0).abs() < 0.01);
        assert!((buckets.days_61_90 - 300.0).abs() < 0.01);
        assert!((buckets.days_90_plus - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_dso() {
        let dso = calculate_dso(500_000.0, 2_000_000.0, 90);
        assert!((dso - 22.5).abs() < 0.01);
    }

    #[test]
    fn test_dso_zero_sales() {
        assert_eq!(calculate_dso(100.0, 0.0, 90), 0.0);
    }
}
