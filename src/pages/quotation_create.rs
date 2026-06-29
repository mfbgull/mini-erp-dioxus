//! Quotation Create Page — Form to create quotations with customer select,
//! line items, discount, tax, and totals.

use crate::calculations::{
    self,
    quotation::{calculate_item_discount, calculate_item_total},
    formatting,
    Discount, DiscountScope, DiscountType, InvoiceMetrics,
};
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use chrono::NaiveDate;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Constants & CSS
// ============================================================================

const DEFAULT_TAX_RATE: f64 = 16.0;
const MIN_ITEM_ROWS: usize = 3;
const VALIDITY_DAYS: i64 = 15;

const PAGE_CSS: &str = r##"
.quotation-create-page { max-width: 1000px; margin: 0 auto; }

.quotation-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.quotation-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.quotation-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.quotation-back-link:hover { text-decoration: underline; }

.quotation-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.quotation-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }

.quotation-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.quotation-form-row > * { flex: 1; min-width: 180px; }

.quotation-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.quotation-items-table th { text-align: left; padding: 8px 6px; font-weight: 600; font-size: 12px; color: var(--text-secondary, #6c757d); text-transform: uppercase; letter-spacing: 0.03em; border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.quotation-items-table td { padding: 6px; vertical-align: middle; border-bottom: 1px solid var(--border-color, #e0e0e0); }

.quotation-item-num { text-align: center; font-weight: 600; color: var(--text-secondary); font-size: 13px; width: 30px; }
.quotation-item-amount { text-align: right; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; padding-right: 10px; }
.quotation-item-actions { width: 40px; text-align: center; }

.quotation-remove-btn { border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 16px; padding: 4px; border-radius: 4px; transition: background 0.15s; line-height: 1; }
.quotation-remove-btn:hover { background: rgba(220, 53, 69, 0.1); }
.quotation-add-row { margin-top: 10px; }

.quotation-totals-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px; }
.quotation-discount-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.quotation-discount-row .cb-input-group { max-width: 120px; }

.quotation-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }

@media (max-width: 768px) {
    .quotation-form-row { flex-direction: column; }
    .quotation-form-row > * { min-width: 100%; }
    .quotation-totals-grid { grid-template-columns: 1fr 1fr; }
    .quotation-action-bar { flex-direction: column; }
}
"##;

// ============================================================================
// Data Types
// ============================================================================

static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
struct LineItem {
    id: u64,
    item_code: String,
    item_name: String,
    quantity: f64,
    unit_price: f64,
    discount_type: String,
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

// ============================================================================
// Sample Data
// ============================================================================

fn customer_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "CUST-001".to_string(), label: "Alpha Traders".to_string() },
        SelectOption { value: "CUST-002".to_string(), label: "Beta Industries".to_string() },
        SelectOption { value: "CUST-003".to_string(), label: "Gamma Supplies".to_string() },
        SelectOption { value: "CUST-004".to_string(), label: "Delta Corp".to_string() },
        SelectOption { value: "CUST-005".to_string(), label: "Epsilon LLC".to_string() },
        SelectOption { value: "CUST-006".to_string(), label: "Zeta Enterprises".to_string() },
        SelectOption { value: "CUST-007".to_string(), label: "Eta Manufacturing".to_string() },
        SelectOption { value: "CUST-008".to_string(), label: "Theta Retail".to_string() },
    ]
}

fn item_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "ITM-0001".to_string(), label: "Premium Widget Alpha".to_string() },
        SelectOption { value: "ITM-0002".to_string(), label: "Industrial Bolt M12".to_string() },
        SelectOption { value: "ITM-0003".to_string(), label: "Steel Rod 12mm x 6m".to_string() },
        SelectOption { value: "ITM-0004".to_string(), label: "Hydraulic Pump HPD-200".to_string() },
        SelectOption { value: "ITM-0005".to_string(), label: "Rubber Gasket Set".to_string() },
        SelectOption { value: "ITM-0006".to_string(), label: "Copper Wire 2.5mm (100m)".to_string() },
        SelectOption { value: "ITM-0007".to_string(), label: "LED Panel Light 24W".to_string() },
        SelectOption { value: "ITM-0008".to_string(), label: "Packaging Box 40x30x20cm".to_string() },
    ]
}

fn item_price(code: &str) -> f64 {
    match code {
        "ITM-0001" => 29.99,   "ITM-0002" => 0.45,   "ITM-0003" => 15.75,
        "ITM-0004" => 1250.00, "ITM-0005" => 8.99,   "ITM-0006" => 45.00,
        "ITM-0007" => 18.50,   "ITM-0008" => 1.20,   _ => 0.0,
    }
}

fn item_name(code: &str) -> String {
    item_options().into_iter().find(|o| o.value == code).map(|o| o.label).unwrap_or_default()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn QuotationCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    let items = use_signal(|| {
        let mut v: Vec<LineItem> = Vec::new();
        for _ in 0..MIN_ITEM_ROWS { v.push(LineItem::default()); }
        v
    });

    let customer_code = use_signal(String::new);
    let customer_name = use_signal(String::new);
    let today = chrono::Local::now().date_naive();
    let mut quote_date = use_signal(|| Some(today));
    let mut valid_until = use_signal(|| Some(today + chrono::Duration::days(VALIDITY_DAYS)));
    let discount_pct = use_signal(|| String::from("0"));
    let tax_rate_str = use_signal(|| format!("{}", DEFAULT_TAX_RATE));
    let notes = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let show_discard_modal = use_signal(|| false);

    let item_totals: Vec<f64> = items.read().iter().map(|li| li.net_amount()).collect();
    let discount_val = discount_pct.read().parse::<f64>().unwrap_or(0.0);
    let tax_rate_val = tax_rate_str.read().parse::<f64>().unwrap_or(0.0);

    let discount = Discount {
        scope: DiscountScope::BeforeTax,
        r#type: DiscountType::Percentage,
        value: discount_val,
    };

    // Use the invoice metrics compute function since quotation math is similar
    let metrics = if !item_totals.is_empty() {
        crate::calculations::invoice::compute_invoice_metrics(item_totals.clone(), &discount, tax_rate_val)
    } else {
        crate::calculations::InvoiceMetrics { subtotal: 0.0, discount_amount: 0.0, taxable_amount: 0.0, tax_amount: 0.0, total: 0.0 }
    };

    let filled_count = items.read().iter().filter(|li| !li.item_code.is_empty()).count();

    let on_customer_select = {
        let mut code = customer_code.clone();
        let mut name = customer_name.clone();
        let mut dirty = is_dirty.clone();
        move |value: String| {
            code.set(value.clone());
            name.set(customer_options().into_iter().find(|o| o.value == value).map(|o| o.label).unwrap_or_default());
            dirty.set(true);
        }
    };

    let add_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |_| { its.write().push(LineItem::default()); dirty.set(true); }
    };

    let remove_item = {
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        move |id: u64| { its.write().retain(|li| li.id != id); dirty.set(true); }
    };

    let save_quotation = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let c_code = customer_code.clone();
        let c_name = customer_name.clone();
        let mut its = items.clone();
        let mut dirty = is_dirty.clone();
        let navigator = navigator.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            if its.read().iter().filter(|li| !li.item_code.is_empty()).count() == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }

            saving.set(true);
            let customer = c_name.read().clone();
            let item_count = its.read().len();
            let mut toast = toast.clone();
            let nav = navigator.clone();

            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(800)).await;
                toast.success("Quotation Created", &format!("Quotation for {} with {} item(s).", customer, item_count));
                saving.set(false);
                dirty.set(false);
                nav.push("/sales/quotations");
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = customer_code.clone();
        let c_name = customer_name.clone();
        let mut its = items.clone();
        let mut inv_date = quote_date.clone();
        let mut v_until = valid_until.clone();
        let mut nts = notes.clone();
        let mut disc_pct = discount_pct.clone();
        let mut tax_str = tax_rate_str.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            if its.read().iter().filter(|li| !li.item_code.is_empty()).count() == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }

            saving.set(true);
            let c_name = c_name.clone();
            let item_count = its.read().len();
            let mut toast = toast.clone();

            spawn(async move {
                crate::utils::sleep(std::time::Duration::from_millis(800)).await;
                toast.success("Quotation Created", &format!("Quotation for {} with {} item(s). Creating another…", c_name.read(), item_count));
                c_code.set(String::new());
                its.write().clear();
                for _ in 0..MIN_ITEM_ROWS { its.write().push(LineItem::default()); }
                let t = chrono::Local::now().date_naive();
                inv_date.set(Some(t));
                v_until.set(Some(t + chrono::Duration::days(VALIDITY_DAYS)));
                nts.set(String::new());
                disc_pct.set(String::from("0"));
                tax_str.set(format!("{}", DEFAULT_TAX_RATE));
                saving.set(false);
                dirty.set(false);
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let dirty = is_dirty.clone();
        let nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/sales/quotations"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/sales/quotations"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page quotation-create-page",

            div { class: "quotation-create-header",
                div {
                    a { class: "quotation-back-link", href: "/sales/quotations", "← Back to Quotations" }
                    h1 { "New Quotation" }
                }
                if *is_dirty.read() {
                    span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" }
                }
            }

            // Section: Quotation Details
            div { class: "quotation-section",
                h2 { "Quotation Details" }
                div { class: "quotation-form-row",
                    div {
                        SearchableSelect {
                            options: customer_options(),
                            selected_value: Some(customer_code.read().clone()),
                            on_select: on_customer_select,
                            placeholder: "Select customer…",
                            searchable: true,
                            class: "cb-input-group",
                        }
                    }
                    div {
                        FormInput {
                            label: "Date".to_string(),
                            value: (*quote_date.read()).map(|d| d.to_string()).unwrap_or_default(),
                            r#type: InputType::Date,
                            oninput: move |v: String| {
                                if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                    quote_date.set(Some(d));
                                    is_dirty.set(true);
                                }
                            },
                        }
                    }
                    div {
                        FormInput {
                            label: "Valid Until".to_string(),
                            value: (*valid_until.read()).map(|d| d.to_string()).unwrap_or_default(),
                            r#type: InputType::Date,
                            oninput: move |v: String| {
                                if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                    valid_until.set(Some(d));
                                    is_dirty.set(true);
                                }
                            },
                        }
                    }
                }
            }

            // Section: Items
            div { class: "quotation-section",
                h2 { "Items" }
                div { style: "overflow-x: auto;",
                    table { class: "quotation-items-table",
                        thead { tr {
                            th { style: "width: 30px;", "#" }
                            th { style: "min-width: 180px;", "Item" }
                            th { style: "width: 70px;", "Qty" }
                            th { style: "width: 90px;", "Rate" }
                            th { style: "width: 70px;", "Disc %" }
                            th { style: "width: 60px;", "Tax %" }
                            th { style: "text-align: right; width: 100px;", "Amount" }
                            th { style: "width: 40px;" }
                        }}
                        tbody {
                            {items.read().iter().map(|li| {
                                let item_data = li.clone();
                                let idx = items.read().iter().position(|x| x.id == li.id).unwrap_or(0);
                                let net_amt = li.net_amount();
                                let net_amt_str = format!("PKR {net_amt:.2}");

                                let on_item_select = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    move |value: String| {
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.item_code = value.clone();
                                            line.item_name = item_name(&value);
                                            line.unit_price = item_price(&value);
                                        }
                                        dirty.set(true);
                                    }
                                };

                                let on_qty = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    move |v: String| {
                                        let q = v.parse::<f64>().unwrap_or(0.0).max(0.0);
                                        if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.quantity = q; }
                                        dirty.set(true);
                                    }
                                };

                                let on_price = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    move |v: String| {
                                        let p = v.parse::<f64>().unwrap_or(0.0).max(0.0);
                                        if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.unit_price = p; }
                                        dirty.set(true);
                                    }
                                };

                                let on_disc = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    move |v: String| {
                                        let d = v.parse::<f64>().unwrap_or(0.0).max(0.0).min(100.0);
                                        if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.discount_value = d; line.discount_type = "Percentage".to_string(); }
                                        dirty.set(true);
                                    }
                                };

                                let on_tax = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    move |v: String| {
                                        let t = v.parse::<f64>().unwrap_or(0.0).max(0.0);
                                        if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.tax_rate = t; }
                                        dirty.set(true);
                                    }
                                };

                                let mut on_rem = remove_item.clone();
                                let rem_id = item_data.id;

                                rsx! {
                                    tr { key: "item-{item_data.id}",
                                        td { class: "quotation-item-num", "{idx + 1}" }
                                        td {
                                            SearchableSelect {
                                                options: item_options(),
                                                selected_value: (!item_data.item_code.is_empty()).then(|| item_data.item_code.clone()),
                                                on_select: on_item_select,
                                                placeholder: "Search item…",
                                                searchable: true,
                                            }
                                        }
                                        td {
                                            FormInput {
                                                value: if item_data.quantity == 0.0 { String::new() } else { format!("{:.0}", item_data.quantity) },
                                                oninput: on_qty,
                                                r#type: InputType::Number,
                                                min: Some(0.0), step: Some(1.0),
                                            }
                                        }
                                        td {
                                            FormInput {
                                                value: if item_data.unit_price == 0.0 { String::new() } else { format!("{:.2}", item_data.unit_price) },
                                                oninput: on_price,
                                                r#type: InputType::Number,
                                                min: Some(0.0), step: Some(0.01),
                                            }
                                        }
                                        td {
                                            FormInput {
                                                value: if item_data.discount_value == 0.0 { String::new() } else { format!("{:.0}", item_data.discount_value) },
                                                oninput: on_disc,
                                                r#type: InputType::Number,
                                                min: Some(0.0), max: Some(100.0), step: Some(1.0),
                                            }
                                        }
                                        td {
                                            FormInput {
                                                value: if item_data.tax_rate == 0.0 { String::new() } else { format!("{:.0}", item_data.tax_rate) },
                                                oninput: on_tax,
                                                r#type: InputType::Number,
                                                min: Some(0.0), step: Some(1.0),
                                            }
                                        }
                                        td { class: "quotation-item-amount", style: "width: 100px;",
                                            if !item_data.item_code.is_empty() {
                                                span { "{net_amt_str}" }
                                            } else {
                                                span { style: "color: var(--text-secondary); font-weight: 400;", "—" }
                                            }
                                        }
                                        td { class: "quotation-item-actions",
                                            button { class: "quotation-remove-btn", r#type: "button", onclick: move |_| on_rem(rem_id), title: "Remove item", "×" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
                div { class: "quotation-add-row",
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon: Some("+".to_string()), onclick: add_item, disabled: *is_saving.read(), "Add Item" }
                }
            }

            // Section: Discount & Tax
            div { class: "quotation-section",
                h2 { "Discount & Tax" }
                div { class: "quotation-discount-row",
                    FormInput {
                        label: "Header Discount (%)".to_string(),
                        value: discount_pct.read().clone(),
                        oninput: { let mut d = discount_pct.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } },
                        r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5),
                    }
                    FormInput {
                        label: "Tax Rate (%)".to_string(),
                        value: tax_rate_str.read().clone(),
                        oninput: { let mut t = tax_rate_str.clone(); let mut dirty = is_dirty.clone(); move |v| { t.set(v); dirty.set(true); } },
                        r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5),
                    }
                }
            }

            // Section: Totals
            div { class: "quotation-section",
                h2 { "Totals" }
                div { class: "quotation-totals-grid",
                    StatCard { title: "Subtotal".to_string(), value: formatting::format_currency(metrics.subtotal), variant: StatCardVariant::Default }
                    StatCard { title: "Discount".to_string(), value: formatting::format_currency(metrics.discount_amount), variant: if metrics.discount_amount > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: format!("Tax ({:.0}%)", tax_rate_val), value: formatting::format_currency(metrics.tax_amount), variant: StatCardVariant::Default }
                    StatCard { title: "Grand Total".to_string(), value: formatting::format_currency(metrics.total), variant: StatCardVariant::Primary }
                }
            }

            // Section: Notes
            div { class: "quotation-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: { let mut n = notes.clone(); let mut dirty = is_dirty.clone(); move |v| { n.set(v); dirty.set(true); } },
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes or terms…".to_string()),
                    hint: Some("These notes will appear on the printed quotation.".to_string()),
                }
            }

            // Action Bar
            div { class: "quotation-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save_quotation, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Quotation" }
            }

            // Discard Modal
            Modal {
                is_open: show_discard_modal,
                title: Some("Discard changes?".to_string()),
                size: ModalSize::Sm,
                close_on_backdrop: true,
                close_on_escape: true,
                footer: rsx! {
                    Button { variant: ButtonVariant::Secondary, onclick: cancel_discard, "Cancel" }
                    Button { variant: ButtonVariant::Danger, onclick: confirm_discard, "Discard" }
                },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;",
                    "You have unsaved changes. Are you sure you want to discard this quotation?"
                }
            }
        }
    }
}
