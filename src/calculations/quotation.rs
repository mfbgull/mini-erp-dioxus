//! Quotation and Sales Order calculation functions.
//!
//! Ported from the original `quotationCalculations.ts` — pure math for quotes,
//! SOs, and reusable item filtering helpers.

use crate::calculations::{round_money, Discount, DiscountType};

// ---------------------------------------------------------------------------
// Discount & total helpers (quotation-specific handling)
// ---------------------------------------------------------------------------

/// Calculate the per-line discount amount for a quotation / SO item.
///
/// The quotation discount logic closely mirrors invoice logic:
/// - `"flat"`: fixed amount (capped at line gross).
/// - `"percentage"`: percentage of `(quantity × unit_price)`.
pub fn calculate_item_discount(quantity: f64, unit_price: f64, discount_type: &str, discount_value: f64) -> f64 {
    let gross = quantity * unit_price;
    match discount_type {
        "flat" => round_money(discount_value.min(gross)),
        "percentage" => round_money(gross * discount_value / 100.0),
        _ => 0.0,
    }
}

/// Calculate the net line total for a quotation / SO item (after per-item discount).
pub fn calculate_item_total(quantity: f64, unit_price: f64, discount_amount: f64) -> f64 {
    round_money(quantity * unit_price - discount_amount)
}

/// Calculate the subtotal of all line items (net of per-item discounts).
pub fn calculate_subtotal(items: &[f64]) -> f64 {
    round_money(items.iter().sum())
}

/// Calculate the header-level discount amount (same as invoice logic).
pub fn calculate_discount(subtotal: f64, discount_type: &str, discount_value: f64) -> f64 {
    match discount_type {
        "flat" => round_money(discount_value.min(subtotal)),
        "percentage" => round_money(subtotal * discount_value / 100.0),
        _ => 0.0,
    }
}

/// Calculate tax amount on the given base.
pub fn calculate_tax(taxable_amount: f64, tax_rate: f64) -> f64 {
    round_money(taxable_amount * tax_rate / 100.0)
}

/// Calculate the grand total for a quotation / SO.
///
/// Applies discount before or after tax per `discount.scope`.
pub fn calculate_total(
    item_totals: &[f64],
    discount: &Discount,
    tax_rate: f64,
) -> f64 {
    let subtotal = calculate_subtotal(item_totals);

    let disc_amt = match discount.r#type {
        DiscountType::Flat => discount.value.min(subtotal),
        DiscountType::Percentage => subtotal * discount.value / 100.0,
    };
    let disc_amt = round_money(disc_amt);

    // Tax is always computed on subtotal (before discount) for quotations.
    let tax = calculate_tax(subtotal, tax_rate);

    match discount.scope {
        crate::calculations::DiscountScope::BeforeTax => {
            round_money(subtotal - disc_amt + tax)
        }
        crate::calculations::DiscountScope::AfterTax => {
            round_money(subtotal + tax - disc_amt)
        }
    }
}

// ---------------------------------------------------------------------------
// Item helpers
// ---------------------------------------------------------------------------

/// Filter items that can be sold (finished goods or purchased items).
///
/// A "sellable" item is one that is either a finished good (`is_finished_good`)
/// or a purchased item (`is_purchased`). Raw materials are typically not
/// sold directly.
pub fn filter_sellable_items(
    items: &[ItemFlags],
) -> Vec<ItemFlags> {
    items
        .iter()
        .filter(|i| i.is_finished_good || i.is_purchased)
        .copied()
        .collect()
}

/// Filter out empty/incomplete form rows (where item_id or name is empty).
///
/// A row is considered "filled" if its `is_filled` flag is true.
pub fn filter_filled_items<T: HasFilled>(items: &[T]) -> Vec<&T> {
    items.iter().filter(|i| i.is_filled()).collect()
}

/// Minimal item flags used by `filter_sellable_items`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemFlags {
    pub is_raw_material: bool,
    pub is_finished_good: bool,
    pub is_purchased: bool,
    pub is_manufactured: bool,
}

/// Trait for items that can report whether they are "filled" (i.e., have data).
pub trait HasFilled {
    fn is_filled(&self) -> bool;
}

impl HasFilled for ItemFlags {
    fn is_filled(&self) -> bool {
        // All item flags are "filled" — this trait is for form rows.
        true
    }
}

/// Create an empty quotation item row (default zeroed).
#[derive(Debug, Clone, Default)]
pub struct QuotationFormItem {
    pub item_id: Option<i64>,
    pub item_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_type: String,
    pub discount_value: f64,
    pub tax_rate: f64,
}

impl HasFilled for QuotationFormItem {
    fn is_filled(&self) -> bool {
        self.item_id.is_some() && !self.item_name.is_empty()
    }
}

/// Create a new empty quotation form row.
pub fn create_empty_item_row() -> QuotationFormItem {
    QuotationFormItem::default()
}

/// Pad a vector to at least `min` items by appending default rows.
pub fn pad_items_to_minimum(items: &mut Vec<QuotationFormItem>, min: usize) {
    while items.len() < min {
        items.push(QuotationFormItem::default());
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
    fn test_quotation_item_discount_percentage() {
        let d = calculate_item_discount(10.0, 50.0, "percentage", 10.0);
        assert!((d - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_quotation_item_discount_flat() {
        let d = calculate_item_discount(10.0, 50.0, "flat", 25.0);
        assert!((d - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_quotation_item_total() {
        let t = calculate_item_total(10.0, 50.0, 50.0);
        assert!((t - 450.0).abs() < 0.01);
    }

    #[test]
    fn test_quotation_subtotal() {
        let s = calculate_subtotal(&[200.0, 300.0]);
        assert!((s - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_quotation_total_before_tax() {
        let discount = Discount {
            scope: DS::BeforeTax,
            r#type: DT::Percentage,
            value: 10.0,
        };
        let total = calculate_total(&[400.0, 600.0], &discount, 16.0);
        // subtotal=1000, discount=100, taxable=900, tax=144, total=900+144=1044
        assert!((total - 1044.0).abs() < 0.01);
    }

    #[test]
    fn test_quotation_total_after_tax() {
        let discount = Discount {
            scope: DS::AfterTax,
            r#type: DT::Flat,
            value: 50.0,
        };
        let total = calculate_total(&[400.0, 600.0], &discount, 16.0);
        // subtotal=1000, tax=160, discount=50, total=1000+160-50=1110
        assert!((total - 1110.0).abs() < 0.01);
    }

    #[test]
    fn test_filter_sellable_items() {
        let items = vec![
            ItemFlags { is_raw_material: true, is_finished_good: false, is_purchased: false, is_manufactured: false },
            ItemFlags { is_raw_material: false, is_finished_good: true, is_purchased: false, is_manufactured: false },
            ItemFlags { is_raw_material: false, is_finished_good: false, is_purchased: true, is_manufactured: false },
        ];
        let sellable = filter_sellable_items(&items);
        assert_eq!(sellable.len(), 2);
    }

    #[test]
    fn test_filter_filled_items() {
        let items = vec![
            QuotationFormItem { item_id: Some(1), item_name: "Widget".to_string(), ..Default::default() },
            QuotationFormItem { item_id: None, item_name: "".to_string(), ..Default::default() },
        ];
        let filled = filter_filled_items(&items);
        assert_eq!(filled.len(), 1);
    }

    #[test]
    fn test_pad_items() {
        let mut items: Vec<QuotationFormItem> = vec![];
        pad_items_to_minimum(&mut items, 3);
        assert_eq!(items.len(), 3);
    }
}
