//! Invoice calculation functions.
//!
//! Ported from the original `invoiceCalculations.ts` — all pure, synchronous,
//! side-effect-free math for line-item discounts, subtotals, taxes, and
//! payment status logic.

use crate::calculations::{round_money, Discount, DiscountScope, DiscountType, InvoiceMetrics};

// ---------------------------------------------------------------------------
// Line-item helpers
// ---------------------------------------------------------------------------

/// Calculate the discount amount for a single line item.
///
/// - `"flat"`: `discount_value` is a fixed amount subtracted directly.
/// - `"percentage"`: `discount_value` is a percentage of `(quantity × unit_price)`.
/// - Any other type returns 0.0.
///
/// # Examples
///
/// ```ignore
/// // Percentage discount: 10% of (5 × 100) = 50
/// assert_eq!(calculate_item_discount(5.0, 100.0, "percentage", 10.0), 50.0);
///
/// // Flat discount: subtract 20 directly
/// assert_eq!(calculate_item_discount(5.0, 100.0, "flat", 20.0), 20.0);
/// ```
pub fn calculate_item_discount(quantity: f64, unit_price: f64, discount_type: &str, discount_value: f64) -> f64 {
    let gross = quantity * unit_price;
    match discount_type {
        "flat" => round_money(discount_value.min(gross)),
        "percentage" => round_money(gross * discount_value / 100.0),
        _ => 0.0,
    }
}

/// Calculate the net total for a single line item (after per-item discount).
///
/// Equivalent to `(quantity × unit_price) - item_discount`.
pub fn calculate_item_total(quantity: f64, unit_price: f64, discount: f64) -> f64 {
    round_money(quantity * unit_price - discount)
}

// ---------------------------------------------------------------------------
// Document-level totals
// ---------------------------------------------------------------------------

/// Calculate the sum of line-item totals before any header-level discount or tax.
///
/// Each item is expected to already have its per-item discount applied.
pub fn calculate_subtotal(items: &[f64]) -> f64 {
    round_money(items.iter().sum())
}

/// Calculate the tax amount on the taxable subtotal.
///
/// `tax_rate` is a percentage (e.g. 16.0 for 16%).
pub fn calculate_tax(taxable_amount: f64, tax_rate: f64) -> f64 {
    round_money(taxable_amount * tax_rate / 100.0)
}

/// Calculate the header-level discount amount.
///
/// - `"flat"`: fixed amount (capped at `subtotal`).
/// - `"percentage"`: percentage of `subtotal`.
/// - Any other type returns 0.0.
pub fn calculate_discount(subtotal: f64, discount_type: &str, discount_value: f64) -> f64 {
    match discount_type {
        "flat" => round_money(discount_value.min(subtotal)),
        "percentage" => round_money(subtotal * discount_value / 100.0),
        _ => 0.0,
    }
}

/// Calculate the grand total given subtotal, discount, and tax.
///
/// The `discount.scope` controls whether the discount is subtracted before
/// or after tax is computed.
pub fn calculate_total(subtotal: f64, discount: &Discount, tax: f64) -> f64 {
    let disc_amt = match discount.r#type {
        DiscountType::Flat => discount.value.min(subtotal),
        DiscountType::Percentage => subtotal * discount.value / 100.0,
    };
    let disc_amt = round_money(disc_amt);

    match discount.scope {
        DiscountScope::BeforeTax => {
            let after_discount = subtotal - disc_amt;
            round_money(after_discount + tax)
        }
        DiscountScope::AfterTax => {
            round_money(subtotal - disc_amt + tax)
        }
    }
}

// ---------------------------------------------------------------------------
// Invoice status
// ---------------------------------------------------------------------------

/// Determine the expected payment status based on amounts.
///
/// # Logic
///
/// | Condition | Status |
/// |-----------|--------|
/// | `paid ≈ 0` | `"Unpaid"` |
/// | `paid ≥ total` | `"Paid"` |
/// | otherwise | `"Partially Paid"` |
pub fn get_expected_status(total: f64, paid: f64, balance: f64) -> &'static str {
    if paid.abs() < 0.01 {
        "Unpaid"
    } else if paid >= total - 0.01 {
        "Paid"
    } else if balance.abs() < 0.01 {
        "Paid"
    } else {
        "Partially Paid"
    }
}

// ---------------------------------------------------------------------------
// Computed metrics
// ---------------------------------------------------------------------------

/// Compute all invoice-level metrics from line-item amounts and header discount.
///
/// This is the primary entry point for invoice calculations. It processes an
/// iterator of line-item totals (net of per-item discount), applies the
/// header-level discount according to its scope, applies tax, and returns the
/// full [`InvoiceMetrics`] struct.
///
/// # Arguments
///
/// * `item_totals` — Iterator of each line item's net amount (after per-item discount).
/// * `discount` — Header-level discount configuration.
/// * `tax_rate` — Tax percentage (e.g. 16.0 for 16% GST).
///
/// # Examples
///
/// ```ignore
/// let items = vec![100.0, 200.0, 150.0];  // after line-item discounts
/// let discount = Discount { scope: BeforeTax, r#type: Percentage, value: 10.0 };
/// let metrics = compute_invoice_metrics(items, &discount, 16.0);
///
/// // subtotal = 450.0
/// // discount = 45.0
/// // taxable  = 405.0
/// // tax      = 64.8
/// // total    = 469.8
/// ```
pub fn compute_invoice_metrics(
    item_totals: impl IntoIterator<Item = f64>,
    discount: &Discount,
    tax_rate: f64,
) -> InvoiceMetrics {
    let items: Vec<f64> = item_totals.into_iter().collect();
    let subtotal = calculate_subtotal(&items);

    // Header discount
    let discount_amount = match discount.r#type {
        DiscountType::Flat => discount.value.min(subtotal),
        DiscountType::Percentage => subtotal * discount.value / 100.0,
    };
    let discount_amount = round_money(discount_amount);

    let taxable_amount = match discount.scope {
        DiscountScope::BeforeTax => round_money(subtotal - discount_amount),
        DiscountScope::AfterTax => subtotal,
    };

    let tax_amount = calculate_tax(taxable_amount, tax_rate);
    let total = calculate_total(subtotal, discount, tax_amount);

    InvoiceMetrics {
        subtotal,
        discount_amount,
        taxable_amount,
        tax_amount,
        total,
    }
}

// ---------------------------------------------------------------------------
// Form helpers
// ---------------------------------------------------------------------------

/// Pad the items vector to at least `min` rows by appending default rows.
pub fn pad_items_to_minimum<T: Default>(items: &mut Vec<T>, min: usize) {
    while items.len() < min {
        items.push(T::default());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculations::{DiscountScope as DS, DiscountType as DT};

    #[test]
    fn test_item_discount_percentage() {
        assert!((calculate_item_discount(5.0, 100.0, "percentage", 10.0) - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_item_discount_flat() {
        assert!((calculate_item_discount(5.0, 100.0, "flat", 30.0) - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_item_discount_capped() {
        // Discount can't exceed line total
        assert!((calculate_item_discount(2.0, 50.0, "flat", 200.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_item_discount_unknown_type() {
        assert_eq!(calculate_item_discount(5.0, 100.0, "invalid", 10.0), 0.0);
    }

    #[test]
    fn test_item_total() {
        assert!((calculate_item_total(5.0, 100.0, 50.0) - 450.0).abs() < 0.01);
    }

    #[test]
    fn test_subtotal() {
        let items = vec![100.0, 200.0, 50.0];
        assert!((calculate_subtotal(&items) - 350.0).abs() < 0.01);
    }

    #[test]
    fn test_tax() {
        assert!((calculate_tax(100.0, 16.0) - 16.0).abs() < 0.01);
    }

    #[test]
    fn test_discount_percentage() {
        assert!((calculate_discount(1000.0, "percentage", 10.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_discount_flat() {
        assert!((calculate_discount(1000.0, "flat", 150.0) - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_discount_capped() {
        assert!((calculate_discount(100.0, "flat", 200.0) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_total_before_tax_discount() {
        let discount = Discount {
            scope: DS::BeforeTax,
            r#type: DT::Percentage,
            value: 10.0,
        };
        // subtotal=1000, discount=100, taxable=900, tax=144 (16%), total=900+144=1044
        let total = calculate_total(1000.0, &discount, 144.0);
        assert!((total - 1044.0).abs() < 0.01);
    }

    #[test]
    fn test_total_after_tax_discount() {
        let discount = Discount {
            scope: DS::AfterTax,
            r#type: DT::Flat,
            value: 50.0,
        };
        // subtotal=1000, tax=160, discount=50, total=1000+160-50=1110
        let total = calculate_total(1000.0, &discount, 160.0);
        assert!((total - 1110.0).abs() < 0.01);
    }

    #[test]
    fn test_get_expected_status() {
        assert_eq!(get_expected_status(100.0, 0.0, 100.0), "Unpaid");
        assert_eq!(get_expected_status(100.0, 100.0, 0.0), "Paid");
        assert_eq!(get_expected_status(100.0, 50.0, 50.0), "Partially Paid");
        assert_eq!(get_expected_status(100.0, 99.99, 0.01), "Partially Paid");
    }

    #[test]
    fn test_compute_invoice_metrics_before_tax() {
        let discount = Discount {
            scope: DS::BeforeTax,
            r#type: DT::Percentage,
            value: 10.0,
        };
        let items = vec![200.0, 300.0, 500.0]; // subtotal = 1000
        let m = compute_invoice_metrics(items, &discount, 16.0);

        assert!((m.subtotal - 1000.0).abs() < 0.01);
        assert!((m.discount_amount - 100.0).abs() < 0.01);
        assert!((m.taxable_amount - 900.0).abs() < 0.01);
        assert!((m.tax_amount - 144.0).abs() < 0.01);
        assert!((m.total - 1044.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_invoice_metrics_after_tax() {
        let discount = Discount {
            scope: DS::AfterTax,
            r#type: DT::Flat,
            value: 50.0,
        };
        let items = vec![400.0, 600.0]; // subtotal = 1000
        let m = compute_invoice_metrics(items, &discount, 16.0);

        assert!((m.subtotal - 1000.0).abs() < 0.01);
        assert!((m.discount_amount - 50.0).abs() < 0.01);
        assert!((m.taxable_amount - 1000.0).abs() < 0.01); // after-tax: taxable = subtotal
        assert!((m.tax_amount - 160.0).abs() < 0.01);
        // total = subtotal + tax - discount = 1000 + 160 - 50
        assert!((m.total - 1110.0).abs() < 0.01);
    }
}
