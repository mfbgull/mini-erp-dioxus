//! Invoice Create Page — A full-featured invoice creation form using the
//! common UI component library (Button, FormInput, SearchableSelect,
//! DateRangePicker, StatCard, Toast, Modal).
//!
//! This page is the canonical example of how all common components work
//! together in a complex data-entry form. It demonstrates:
//!
//! - Customer selection via `SearchableSelect`
//! - Date entry via `DateRangePicker` (invoice + due date with range validation)
//! - Dynamic line-item grid with inline use of `SearchableSelect` + `FormInput`
//! - Header-level discount with scope toggle
//! - Auto-computed totals using `calculations::invoice::compute_invoice_metrics`
//! - Totals summary using `StatCard`
//! - Discard confirmation via `Modal`
//! - Save / Save & New / Cancel actions with toast feedback
//! - Form dirty-state detection

use crate::calculations::{
    invoice::calculate_item_discount,
    invoice::calculate_item_total,
    invoice::compute_invoice_metrics,
    Discount, DiscountScope, DiscountType, InvoiceMetrics,
};
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, DateRangePicker, FormInput, InputType, Modal,
    ModalSize, SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use crate::auth::use_auth;
use crate::models::{Customer, Item, InvoiceForm, InvoiceItemForm};
use chrono::NaiveDate;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Constants & CSS
// ============================================================================

/// Default tax rate used for new invoices.
const DEFAULT_TAX_RATE: f64 = 16.0;

/// Minimum number of empty item rows to show.
const MIN_ITEM_ROWS: usize = 3;

/// Inline page-level styles (complements COMMON_CSS).
const PAGE_CSS: &str = r##"
.invoice-create-page {
    max-width: 1000px;
    margin: 0 auto;
}

.invoice-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.invoice-create-header h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
}

.invoice-back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
    margin-bottom: 16px;
}

.invoice-back-link:hover { text-decoration: underline; }

/* ── Section Card ── */
.invoice-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.invoice-section h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

/* ── Form Row Layout ── */
.invoice-form-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.invoice-form-row > * {
    flex: 1;
    min-width: 180px;
}

/* ── Items Table ── */
.invoice-items-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.invoice-items-table th {
    text-align: left;
    padding: 8px 6px;
    font-weight: 600;
    font-size: 12px;
    color: var(--text-secondary, #6c757d);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    border-bottom: 2px solid var(--border-color, #e0e0e0);
    white-space: nowrap;
}

.invoice-items-table td {
    padding: 6px;
    vertical-align: middle;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.invoice-item-num {
    text-align: center;
    font-weight: 600;
    color: var(--text-secondary);
    font-size: 13px;
    width: 30px;
}

.invoice-item-cell-wide {
    min-width: 180px;
}

.invoice-item-cell-narrow {
    min-width: 70px;
}

.invoice-item-amount {
    text-align: right;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    padding-right: 10px;
}

.invoice-item-actions {
    width: 40px;
    text-align: center;
}

.invoice-remove-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--danger, #dc3545);
    font-size: 16px;
    padding: 4px;
    border-radius: 4px;
    transition: background 0.15s;
    line-height: 1;
}

.invoice-remove-btn:hover {
    background: rgba(220, 53, 69, 0.1);
}

.invoice-add-row {
    margin-top: 10px;
}

/* ── Totals Section ── */
.invoice-totals-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 16px;
}

/* ── Discount Section ── */
.invoice-discount-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.invoice-discount-row .cb-input-group {
    max-width: 120px;
}

.invoice-scope-btn {
    padding: 6px 14px;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius-sm, 4px);
    background: #fff;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    transition: all 0.15s;
}

.invoice-scope-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
}

.invoice-scope-btn-active {
    background: var(--accent, #4a90d9);
    color: #fff;
    border-color: var(--accent, #4a90d9);
}

.invoice-scope-btn-active:hover {
    background: #357abd;
    color: #fff;
}

/* ── Payment Section ── */
.invoice-payment-toggle {
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    user-select: none;
}

.invoice-payment-toggle input[type="checkbox"] {
    accent-color: var(--accent, #4a90d9);
    width: 16px;
    height: 16px;
}

.invoice-payment-fields {
    display: flex;
    gap: 16px;
    margin-top: 16px;
    flex-wrap: wrap;
}

.invoice-payment-fields > * {
    flex: 1;
    min-width: 180px;
}

/* ── Action Bar ── */
.invoice-action-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}

/* ── Responsive ── */
@media (max-width: 768px) {
    .invoice-form-row { flex-direction: column; }
    .invoice-form-row > * { min-width: 100%; }

    .invoice-items-table th:nth-child(n+4),
    .invoice-items-table td:nth-child(n+4) { display: none; }

    .invoice-totals-grid {
        grid-template-columns: 1fr 1fr;
    }

    .invoice-action-bar {
        flex-direction: column;
    }

    .invoice-discount-row { flex-direction: column; align-items: stretch; }
    .invoice-discount-row .cb-input-group { max-width: 100%; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

/// ID counter for stable line-item keys.
static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

/// A single line item in the invoice.
#[derive(Clone, Debug)]
struct LineItem {
    id: u64,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    discount_type: String,  // "None" | "Percentage" | "Flat"
    discount_value: f64,
    tax_rate: f64,
}

impl Default for LineItem {
    fn default() -> Self {
        Self {
            id: NEXT_LINE_ID.fetch_add(1, Ordering::Relaxed),
            item_code: String::new(),
            item_name: String::new(),
            quantity: 1.0,
            unit_price: 0.0,
            discount_type: "None".to_string(),
            discount_value: 0.0,
            tax_rate: DEFAULT_TAX_RATE,
        }
    }
}

impl LineItem {
    fn net_amount(&self) -> f64 {
        let disc = if self.discount_type == "Percentage" {
            calculate_item_discount(self.quantity, self.unit_price, "percentage", self.discount_value)
        } else if self.discount_type == "Flat" {
            calculate_item_discount(self.quantity, self.unit_price, "flat", self.discount_value)
        } else {
            0.0
        };
        calculate_item_total(self.quantity, self.unit_price, disc)
    }
}


fn source_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "Direct".to_string(), label: "Direct".to_string() },
        SelectOption { value: "Sales Order".to_string(), label: "Sales Order".to_string() },
        SelectOption { value: "POS".to_string(), label: "POS".to_string() },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn InvoiceCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    // ── State ──
    let items = use_signal(|| {
        let mut v: Vec<LineItem> = Vec::new();
        for _ in 0..MIN_ITEM_ROWS {
            v.push(LineItem::default());
        }
        v
    });

    let customer_code = use_signal(String::new);
    let customer_name = use_signal(String::new);
    let source_type = use_signal(|| "Direct".to_string());

    // Invoice date / due date as NaiveDate options for DateRangePicker
    let today = chrono::Local::now().date_naive();
    let invoice_date = use_signal(|| Some(today));
    let due_date = use_signal(|| Some(today + chrono::Duration::days(30)));

    let discount_pct = use_signal(|| String::from("0"));
    let mut discount_scope = use_signal(|| DiscountScope::BeforeTax);
    let tax_rate_str = use_signal(|| format!("{}", DEFAULT_TAX_RATE));
    let notes = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let is_dirty = use_signal(|| false);
    let show_discard_modal = use_signal(|| false);

    // ── Payment state ──
    let mut record_payment = use_signal(|| false);
    let mut payment_amount = use_signal(String::new);
    let mut payment_method = use_signal(|| "Cash".to_string());

    // ── API-loaded data ──
    let customer_map = use_signal(HashMap::<String, Customer>::new);
    let item_map = use_signal(HashMap::<String, Item>::new);
    let customer_options = use_signal(Vec::<SelectOption>::new);
    let item_options_signal = use_signal(Vec::<SelectOption>::new);

    // ── Load customers and items from API ──
    let auth_api = use_auth().api;
    let mut cust_map = customer_map.clone();
    let mut cust_opts = customer_options.clone();
    let mut it_map = item_map.clone();
    let mut it_opts = item_options_signal.clone();
    use_effect(move || {
        let client = auth_api.with(|c| c.clone());
        spawn(async move {
            if let Ok(customers) = client.list_customers().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for c in &customers {
                    opts.push(SelectOption {
                        value: c.customer_code.clone(),
                        label: format!("{} ({})", c.customer_name, c.customer_code),
                    });
                    map.insert(c.customer_code.clone(), c.clone());
                }
                cust_map.set(map);
                cust_opts.set(opts);
            }
            if let Ok(items) = client.list_items_catalog().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for i in items {
                    let price = i.selling_price;
                    opts.push(SelectOption {
                        value: i.item_code.clone(),
                        label: format!("{} (PKR {:.2})", i.item_name, price),
                    });
                    map.insert(i.item_code.clone(), i);
                }
                it_map.set(map);
                it_opts.set(opts);
            }
        });
    });

    // ── Computed totals ──
    let item_totals: Vec<f64> = items.read().iter().map(|li| li.net_amount()).collect();

    let discount_val = discount_pct.read().parse::<f64>().unwrap_or(0.0);
    let tax_rate_val = tax_rate_str.read().parse::<f64>().unwrap_or(0.0);

    let discount = Discount {
        scope: *discount_scope.read(),
        r#type: DiscountType::Percentage,
        value: discount_val,
    };

    let metrics = if !item_totals.is_empty() {
        compute_invoice_metrics(item_totals.clone(), &discount, tax_rate_val)
    } else {
        InvoiceMetrics {
            subtotal: 0.0,
            discount_amount: 0.0,
            taxable_amount: 0.0,
            tax_amount: 0.0,
            total: 0.0,
        }
    };

    // ── Derive items count ──
    let filled_count = items.read().iter().filter(|li| !li.item_code.is_empty()).count();

    // ── Handlers ──

    let on_customer_select = {
        let mut code = customer_code.clone();
        let mut name = customer_name.clone();
        let mut dirty = is_dirty.clone();
        let cust_map = customer_map.clone();
        move |value: String| {
            code.set(value.clone());
            let label = cust_map.read()
                .get(&value)
                .map(|c| c.customer_name.clone())
                .unwrap_or_default();
            name.set(label);
            dirty.set(true);
        }
    };

    let on_source_change = {
        let mut st = source_type.clone();
        let mut dirty = is_dirty.clone();
        move |value: String| {
            st.set(value);
            dirty.set(true);
        }
    };

    let on_date_change = {
        let mut inv = invoice_date.clone();
        let mut due = due_date.clone();
        let mut dirty = is_dirty.clone();
        move |(start, end): (Option<NaiveDate>, Option<NaiveDate>)| {
            inv.set(start);
            due.set(end);
            dirty.set(true);
        }
    };

    let add_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |_| {
            its.write().push(LineItem::default());
            dirty.set(true);
        }
    };

    let remove_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |id: u64| {
            its.write().retain(|li| li.id != id);
            dirty.set(true);
        }
    };

    // Save: create invoice and navigate to list
    let save_invoice = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = customer_code.clone();
        let c_name = customer_name.clone();
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        let navigator = navigator.clone();
        let source_type = source_type.clone();
        let inv_date = invoice_date.clone();
        let d_date = due_date.clone();
        let nts = notes.clone();
        let disc_pct = discount_pct.clone();
        let tax_str = tax_rate_str.clone();
        let api = auth_api.clone();
        let cust_map = customer_map.clone();
        let it_map = item_map.clone();
        let rec_pay = record_payment.clone();
        let pay_amt = payment_amount.clone();
        let pay_meth = payment_method.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            let filled = its.read().iter().filter(|li| !li.item_code.is_empty()).count();
            if filled == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }

            let cust_id = cust_map.read().get(&c_code.read().clone()).map(|c| c.id).unwrap_or(0);
            let form_items: Vec<InvoiceItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| {
                    let item_id = it_map.read().get(&li.item_code).map(|i| i.id).unwrap_or(0);
                    InvoiceItemForm {
                        item_id,
                        description: Some(li.item_name.clone()),
                        quantity: li.quantity,
                        unit_price: li.unit_price,
                        tax_rate: Some(li.tax_rate),
                        discount_type: if li.discount_value > 0.0 { Some("percentage".to_string()) } else { None },
                        discount_value: if li.discount_value > 0.0 { Some(li.discount_value) } else { None },
                    }
                }).collect();

            let form = InvoiceForm {
                customer_id: cust_id,
                invoice_date: inv_date.read().as_ref().map(|d| d.to_string()).unwrap_or_default(),
                due_date: d_date.read().as_ref().map(|d| d.to_string()),
                source_type: Some(source_type.read().clone()),
                warehouse_id: None,
                discount_scope: Some("BeforeTax".to_string()),
                discount_type: Some("percentage".to_string()),
                discount_value: Some(disc_pct.read().parse::<f64>().unwrap_or(0.0)),
                tax_rate: Some(tax_str.read().parse::<f64>().unwrap_or(0.0)),
                notes: Some(nts.read().clone()),
                items: form_items,
                record_payment: Some(*rec_pay.read()),
                payment_amount: if *rec_pay.read() { pay_amt.read().parse::<f64>().ok() } else { None },
                payment_method: if *rec_pay.read() { Some(pay_meth.read().clone()) } else { None },
                deleted_payment_ids: None,
            };

            saving.set(true);
            let mut toast = toast.clone();
            let nav = navigator.clone();
            let customer = c_name.read().clone();
            let client = api.with(|c| c.clone());

            spawn(async move {
                match client.create_invoice(&form).await {
                    Ok(_inv) => {
                        toast.success("Invoice Created", &format!("Invoice for {}.", customer));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/sales/invoices");
                    }
                    Err(e) => {
                        toast.error("Failed", &format!("Could not create invoice: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Save & New: create invoice, then reset form
    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = customer_code.clone();
        let mut its = items.clone();
        let c_name = customer_name.clone();
        let mut inv_date = invoice_date.clone();
        let mut d_date = due_date.clone();
        let mut nts = notes.clone();
        let mut disc_pct = discount_pct.clone();
        let mut tax_str = tax_rate_str.clone();
        let mut dirty = is_dirty.clone();
        let source_type = source_type.clone();
        let api = auth_api.clone();
        let cust_map = customer_map.clone();
        let it_map = item_map.clone();
        let mut rec_pay = record_payment.clone();
        let mut pay_amt = payment_amount.clone();
        let mut pay_meth = payment_method.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            let filled = its.read().iter().filter(|li| !li.item_code.is_empty()).count();
            if filled == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }

            let cust_id = cust_map.read().get(&c_code.read().clone()).map(|c| c.id).unwrap_or(0);
            let form_items: Vec<InvoiceItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| {
                    let item_id = it_map.read().get(&li.item_code).map(|i| i.id).unwrap_or(0);
                    InvoiceItemForm {
                        item_id,
                        description: Some(li.item_name.clone()),
                        quantity: li.quantity,
                        unit_price: li.unit_price,
                        tax_rate: Some(li.tax_rate),
                        discount_type: if li.discount_value > 0.0 { Some("percentage".to_string()) } else { None },
                        discount_value: if li.discount_value > 0.0 { Some(li.discount_value) } else { None },
                    }
                }).collect();

            let do_record = *rec_pay.read();
            let form = InvoiceForm {
                customer_id: cust_id,
                invoice_date: inv_date.read().as_ref().map(|d| d.to_string()).unwrap_or_default(),
                due_date: d_date.read().as_ref().map(|d| d.to_string()),
                source_type: Some(source_type.read().clone()),
                warehouse_id: None,
                discount_scope: Some("BeforeTax".to_string()),
                discount_type: Some("percentage".to_string()),
                discount_value: Some(disc_pct.read().parse::<f64>().unwrap_or(0.0)),
                tax_rate: Some(tax_str.read().parse::<f64>().unwrap_or(0.0)),
                notes: Some(nts.read().clone()),
                items: form_items,
                record_payment: Some(do_record),
                payment_amount: if do_record { pay_amt.read().parse::<f64>().ok() } else { None },
                payment_method: if do_record { Some(pay_meth.read().clone()) } else { None },
                deleted_payment_ids: None,
            };

            let item_count = its.read().iter().filter(|li| !li.item_code.is_empty()).count() as i32;
            saving.set(true);
            let mut toast = toast.clone();
            let client = api.with(|c| c.clone());

            spawn(async move {
                match client.create_invoice(&form).await {
                    Ok(_inv) => {
                        toast.success(
                            "Invoice Created",
                            &format!("Invoice for {}. Creating another…", c_name.read()),
                        );

                        // Reset form
                        c_code.set(String::new());
                        its.write().clear();
                        for _ in 0..MIN_ITEM_ROWS {
                            its.write().push(LineItem::default());
                        }
                        let t = chrono::Local::now().date_naive();
                        inv_date.set(Some(t));
                        d_date.set(Some(t + chrono::Duration::days(30)));
                        nts.set(String::new());
                        disc_pct.set(String::from("0"));
                        tax_str.set(format!("{}", DEFAULT_TAX_RATE));
                        rec_pay.set(false);
                        pay_amt.set(String::new());
                        pay_meth.set("Cash".to_string());
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Failed", &format!("Could not create invoice: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Discard: confirm via modal if dirty, else navigate immediately
    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let dirty = is_dirty.clone();
        let nav = navigator.clone();
        move |_| {
            if *dirty.read() {
                modal.set(true);
            } else {
                nav.push("/sales/invoices");
            }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| {
            modal.set(false);
            nav.push("/sales/invoices");
        }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    let discount_scope_val = *discount_scope.read();
    let scope_btn_class = if matches!(discount_scope_val, DiscountScope::BeforeTax) { "invoice-scope-btn invoice-scope-btn-active" } else { "invoice-scope-btn" };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page invoice-create-page",

            // ── Header ──
            div { class: "invoice-create-header",
                div {
                    a {
                        class: "invoice-back-link",
                        href: "/sales/invoices",
                        "← Back to Invoices"
                    }
                    h1 { "New Invoice" }
                }
                if *is_dirty.read() {
                    span {
                        style: "font-size: 12px; color: var(--warning); font-weight: 500;",
                        "⚠ Unsaved changes"
                    }
                }
            }

            // ── Section: Header Info ──
            div { class: "invoice-section",
                h2 { "Invoice Details" }

                // Row 1: Customer + Source
                div { class: "invoice-form-row",
                    div {
                        SearchableSelect {
                            options: customer_options(),
                            selected_value: Some(customer_code.read().clone()),
                            on_select: on_customer_select.clone(),
                            placeholder: "Select customer…",
                            searchable: true,
                            class: "cb-input-group",
                        }
                    }
                    div {
                        SearchableSelect {
                            options: source_type_options(),
                            selected_value: Some(source_type.read().clone()),
                            on_select: on_source_change.clone(),
                            placeholder: "Source type…",
                            searchable: false,
                            class: "cb-input-group",
                        }
                    }
                }

                // Row 2: Date range (DateRangePicker with range validation)
                div { class: "invoice-form-row", style: "margin-top: 12px;",
                    DateRangePicker {
                        start: *invoice_date.read(),
                        end: *due_date.read(),
                        on_change: on_date_change,
                        start_label: "Invoice Date".to_string(),
                        end_label: "Due Date".to_string(),
                    }
                }
            }

            // ── Section: Items ──
            div { class: "invoice-section",
                h2 { "Items" }

                div { style: "overflow-x: auto;",
                    table { class: "invoice-items-table",
                        thead {
                            tr {
                                th { style: "width: 30px;", "#" }
                                th { style: "min-width: 180px;", "Item" }
                                th { style: "width: 70px;", "Qty" }
                                th { style: "width: 90px;", "Rate" }
                                th { style: "width: 70px;", "Disc %" }
                                th { style: "width: 60px;", "Tax %" }
                                th { style: "text-align: right; width: 100px;", "Amount" }
                                th { style: "width: 40px;" }
                            }
                        }
                        tbody {
                            {items.read().iter().map(|li| {
                                let item = li.clone();
                                let idx_key = format!("item-{}", li.id);
                                let idx = items.read().iter().position(|x| x.id == li.id).unwrap_or(0);
                                let net_amt = li.net_amount();
                                let net_amt_str = format!("PKR {net_amt:.2}");

                                // Closures for this row
                                let on_item_select = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let it_map = item_map.clone();
                                    move |value: String| {
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == item.id) {
                                            line.item_code = value.clone();
                                            line.item_name = it_map.read().get(&value).map(|i| i.item_name.clone()).unwrap_or_default();
                                            line.unit_price = it_map.read().get(&value).map(|i| i.selling_price).unwrap_or(0.0);
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let on_qty_change = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = li.id;
                                    move |v: String| {
                                        let val = v.parse::<f64>().unwrap_or(0.0);
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.quantity = val.max(0.0);
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let on_price_change = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = li.id;
                                    move |v: String| {
                                        let val = v.parse::<f64>().unwrap_or(0.0);
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.unit_price = val.max(0.0);
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let on_disc_change = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = li.id;
                                    move |v: String| {
                                        let val = v.parse::<f64>().unwrap_or(0.0);
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.discount_value = val.max(0.0).min(100.0);
                                            line.discount_type = "Percentage".to_string();
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let on_tax_change = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = li.id;
                                    move |v: String| {
                                        let val = v.parse::<f64>().unwrap_or(0.0);
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.tax_rate = val.max(0.0);
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let mut on_remove = remove_item.clone();
                                let remove_id = li.id;

                                rsx! {
                                    tr { key: "{idx_key}",
                                        td { class: "invoice-item-num", "{idx + 1}" }
                                        td { class: "invoice-item-cell-wide",
                                            SearchableSelect {
                                                options: item_options_signal(),
                                                selected_value: (!item.item_code.is_empty()).then(|| item.item_code.clone()),
                                                on_select: on_item_select,
                                                placeholder: "Search item…",
                                                searchable: true,
                                            }
                                        }
                                        td { class: "invoice-item-cell-narrow",
                                            FormInput {
                                                value: if item.quantity == 0.0 { String::new() } else { format!("{:.0}", item.quantity) },
                                                oninput: on_qty_change,
                                                r#type: InputType::Number,
                                                min: Some(0.0),
                                                step: Some(1.0),
                                            }
                                        }
                                        td { class: "invoice-item-cell-narrow",
                                            FormInput {
                                                value: if item.unit_price == 0.0 { String::new() } else { format!("{:.2}", item.unit_price) },
                                                oninput: on_price_change,
                                                r#type: InputType::Number,
                                                min: Some(0.0),
                                                step: Some(0.01),
                                            }
                                        }
                                        td { class: "invoice-item-cell-narrow",
                                            FormInput {
                                                value: if item.discount_value == 0.0 { String::new() } else { format!("{:.0}", item.discount_value) },
                                                oninput: on_disc_change,
                                                r#type: InputType::Number,
                                                min: Some(0.0),
                                                max: Some(100.0),
                                                step: Some(1.0),
                                            }
                                        }
                                        td { class: "invoice-item-cell-narrow",
                                            FormInput {
                                                value: if item.tax_rate == 0.0 { String::new() } else { format!("{:.0}", item.tax_rate) },
                                                oninput: on_tax_change,
                                                r#type: InputType::Number,
                                                min: Some(0.0),
                                                step: Some(1.0),
                                            }
                                        }
                                        td {
                                            class: "invoice-item-amount",
                                            style: "width: 100px;",
                                            if !item.item_code.is_empty() {
                                                span { "{net_amt_str}" }
                                            } else {
                                                span { style: "color: var(--text-secondary); font-weight: 400;", "—" }
                                            }
                                        }
                                        td { class: "invoice-item-actions",
                                            button {
                                                class: "invoice-remove-btn",
                                                r#type: "button",
                                                onclick: move |_| on_remove(remove_id),
                                                title: "Remove item",
                                                "×"
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }

                // Add item button
                div { class: "invoice-add-row",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        icon: Some("+".to_string()),
                        onclick: add_item,
                        disabled: *is_saving.read(),
                        "Add Item"
                    }
                }
            }

            // ── Section: Discount & Tax ──
            div { class: "invoice-section",
                h2 { "Discount & Tax" }
                div { class: "invoice-discount-row",
                    FormInput {
                        label: "Header Discount (%)".to_string(),
                        value: discount_pct.read().clone(),
                        oninput: {
                            let mut d = discount_pct.clone();
                            let mut dirty = is_dirty.clone();
                            move |v| { d.set(v); dirty.set(true); }
                        },
                        r#type: InputType::Number,
                        min: Some(0.0),
                        max: Some(100.0),
                        step: Some(0.5),
                    }
                    FormInput {
                        label: "Tax Rate (%)".to_string(),
                        value: tax_rate_str.read().clone(),
                        oninput: {
                            let mut t = tax_rate_str.clone();
                            let mut dirty = is_dirty.clone();
                            move |v| { t.set(v); dirty.set(true); }
                        },
                        r#type: InputType::Number,
                        min: Some(0.0),
                        max: Some(100.0),
                        step: Some(0.5),
                    }
                    div { style: "display: flex; align-items: flex-end; gap: 6px; padding-bottom: 4px;",
                        span { style: "font-size: 12px; font-weight: 600; color: var(--text-secondary); white-space: nowrap;",
                            "Discount scope:"
                        }
                        button {
                            class: "{scope_btn_class}",
                            r#type: "button",
                            onclick: move |_| {
                                let next = match *discount_scope.read() {
                                    DiscountScope::BeforeTax => DiscountScope::AfterTax,
                                    DiscountScope::AfterTax => DiscountScope::BeforeTax,
                                };
                                discount_scope.set(next);
                            },
                            if matches!(*discount_scope.read(), DiscountScope::BeforeTax) { "Before Tax" } else { "After Tax" }
                        }
                    }
                }
            }

            // ── Section: Totals ──
            div { class: "invoice-section",
                h2 { "Totals" }
                div { class: "invoice-totals-grid",
                    StatCard {
                        title: format!("Subtotal"),
                        value: crate::calculations::formatting::format_currency(metrics.subtotal),
                        variant: StatCardVariant::Default,
                    }
                    StatCard {
                        title: format!("Discount"),
                        value: crate::calculations::formatting::format_currency(metrics.discount_amount),
                        variant: if metrics.discount_amount > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default },
                    }
                    StatCard {
                        title: format!("Tax ({:.0}%)", tax_rate_val),
                        value: crate::calculations::formatting::format_currency(metrics.tax_amount),
                        variant: StatCardVariant::Default,
                    }
                    StatCard {
                        title: format!("Grand Total"),
                        value: crate::calculations::formatting::format_currency(metrics.total),
                        variant: StatCardVariant::Primary,
                    }
                }
            }

            // ── Section: Notes ──
            div { class: "invoice-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: {
                        let mut n = notes.clone();
                        let mut dirty = is_dirty.clone();
                        move |v| { n.set(v); dirty.set(true); }
                    },
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes or payment terms…".to_string()),
                    hint: Some("These notes will appear on the printed invoice.".to_string()),
                }
            }

            // ── Section: Payment ──
            div { class: "invoice-section",
                h2 { "Payment" }
                label { class: "invoice-payment-toggle",
                    input {
                        r#type: "checkbox",
                        checked: *record_payment.read(),
                        onchange: {
                            let mut rp = record_payment.clone();
                            let mut pa = payment_amount.clone();
                            let mut pm = payment_method.clone();
                            let mut dirty = is_dirty.clone();
                            let total = metrics.total;
                            move |_| {
                                let new_val = !*rp.read();
                                rp.set(new_val);
                                if new_val {
                                    pa.set(format!("{:.2}", total));
                                    pm.set("Cash".to_string());
                                } else {
                                    pa.set(String::new());
                                    pm.set("Cash".to_string());
                                }
                                dirty.set(true);
                            }
                        },
                    }
                    span { "Record payment with this invoice" }
                }
                if *record_payment.read() {
                    div { class: "invoice-payment-fields",
                        FormInput {
                            label: "Payment Amount".to_string(),
                            value: payment_amount.read().clone(),
                            oninput: {
                                let mut pa = payment_amount.clone();
                                let mut dirty = is_dirty.clone();
                                move |v| { pa.set(v); dirty.set(true); }
                            },
                            r#type: InputType::Number,
                            min: Some(0.01),
                            step: Some(0.01),
                        }
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-bottom: 4px;", "Payment Method" }
                            SearchableSelect {
                                options: vec![
                                    SelectOption { value: "Cash".to_string(), label: "Cash".to_string() },
                                    SelectOption { value: "Bank Transfer".to_string(), label: "Bank Transfer".to_string() },
                                    SelectOption { value: "Credit Card".to_string(), label: "Credit Card".to_string() },
                                    SelectOption { value: "Cheque".to_string(), label: "Cheque".to_string() },
                                ],
                                selected_value: Some(payment_method.read().clone()),
                                on_select: {
                                    let mut pm = payment_method.clone();
                                    let mut dirty = is_dirty.clone();
                                    move |v: String| { pm.set(v); dirty.set(true); }
                                },
                                placeholder: "Select method…",
                                searchable: false,
                                class: Some("cb-input-group".to_string()),
                            }
                        }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "invoice-action-bar",
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: open_discard,
                    disabled: *is_saving.read(),
                    "Discard"
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    onclick: save_and_new,
                    loading: *is_saving.read(),
                    icon: Some("💾".to_string()),
                    "Save & New"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: save_invoice,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Invoice"
                }
            }

            // ── Discard Confirmation Modal ──
            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: cancel_discard,
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Danger,
                        onclick: confirm_discard,
                        "Discard"
                    }
                },
                p {
                    style: "margin: 0; color: var(--text-secondary); font-size: 14px;",
                    "You have unsaved changes. Are you sure you want to discard this invoice?"
                }
            }
        }
    }
}
