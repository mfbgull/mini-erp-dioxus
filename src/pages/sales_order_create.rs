//! Sales Order Create Page — Form to create sales orders with customer select,
//! delivery date, line items, discount, tax, and totals.

use crate::auth::use_auth;
use crate::calculations::{
    invoice::{calculate_item_discount, calculate_item_total, compute_invoice_metrics},
    Discount, DiscountScope, DiscountType, InvoiceMetrics,
};
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use crate::models::{Customer, Item, SalesOrderForm, SalesOrderItemForm};
use chrono::NaiveDate;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// Constants & CSS
// ============================================================================

const DEFAULT_TAX_RATE: f64 = 16.0;
const MIN_ITEM_ROWS: usize = 3;
const LEAD_TIME_DAYS: i64 = 14;

const PAGE_CSS: &str = r##"
.socreate-page { max-width: 1000px; margin: 0 auto; }

.socreate-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.socreate-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.socreate-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.socreate-back-link:hover { text-decoration: underline; }

.socreate-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.socreate-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }

.socreate-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.socreate-form-row > * { flex: 1; min-width: 180px; }

.socreate-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.socreate-items-table th { text-align: left; padding: 8px 6px; font-weight: 600; font-size: 12px; color: var(--text-secondary, #6c757d); text-transform: uppercase; letter-spacing: 0.03em; border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.socreate-items-table td { padding: 6px; vertical-align: middle; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.socreate-item-num { text-align: center; font-weight: 600; color: var(--text-secondary); font-size: 13px; width: 30px; }
.socreate-item-amount { text-align: right; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; padding-right: 10px; }
.socreate-item-actions { width: 40px; text-align: center; }
.socreate-remove-btn { border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 16px; padding: 4px; border-radius: 4px; transition: background 0.15s; line-height: 1; }
.socreate-remove-btn:hover { background: rgba(220, 53, 69, 0.1); }
.socreate-add-row { margin-top: 10px; }

.socreate-totals-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px; }
.socreate-discount-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.socreate-discount-row .cb-input-group { max-width: 120px; }

.socreate-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }

@media (max-width: 768px) {
    .socreate-form-row { flex-direction: column; }
    .socreate-form-row > * { min-width: 100%; }
    .socreate-totals-grid { grid-template-columns: 1fr 1fr; }
    .socreate-action-bar { flex-direction: column; }
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
        } else { 0.0 };
        calculate_item_total(self.quantity, self.unit_price, disc)
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesOrderCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let items = use_signal(|| {
        let mut v: Vec<LineItem> = Vec::new();
        for _ in 0..MIN_ITEM_ROWS { v.push(LineItem::default()); }
        v
    });

    let customer_code = use_signal(String::new);
    let customer_name = use_signal(String::new);
    let today = chrono::Local::now().date_naive();
    let mut order_date = use_signal(|| Some(today));
    let mut delivery_date = use_signal(|| Some(today + chrono::Duration::days(LEAD_TIME_DAYS)));
    let discount_pct = use_signal(|| String::from("0"));
    let tax_rate_str = use_signal(|| format!("{}", DEFAULT_TAX_RATE));
    let notes = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let show_discard_modal = use_signal(|| false);

    // ── API-loaded data (real ids, not hardcoded codes) ──
    let customer_map = use_signal(HashMap::<String, Customer>::new);
    let item_map = use_signal(HashMap::<String, Item>::new);
    let customer_options = use_signal(Vec::<SelectOption>::new);
    let item_options_signal = use_signal(Vec::<SelectOption>::new);
    {
        let auth_api = api;
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
                        opts.push(SelectOption { value: c.customer_code.clone(), label: format!("{} ({})", c.customer_name, c.customer_code) });
                        map.insert(c.customer_code.clone(), c.clone());
                    }
                    cust_map.set(map);
                    cust_opts.set(opts);
                }
                if let Ok(items) = client.list_items_catalog().await {
                    let mut map = HashMap::new();
                    let mut opts = Vec::new();
                    for i in items {
                        opts.push(SelectOption { value: i.item_code.clone(), label: format!("{} ({})", i.item_name, i.item_code) });
                        map.insert(i.item_code.clone(), i);
                    }
                    it_map.set(map);
                    it_opts.set(opts);
                }
            });
        });
    }

    let item_totals: Vec<f64> = items.read().iter().map(|li| li.net_amount()).collect();
    let discount_val = discount_pct.read().parse::<f64>().unwrap_or(0.0);
    let tax_rate_val = tax_rate_str.read().parse::<f64>().unwrap_or(0.0);

    let discount = Discount { scope: DiscountScope::BeforeTax, r#type: DiscountType::Percentage, value: discount_val };

    let metrics = if !item_totals.is_empty() {
        compute_invoice_metrics(item_totals.clone(), &discount, tax_rate_val)
    } else {
        InvoiceMetrics { subtotal: 0.0, discount_amount: 0.0, taxable_amount: 0.0, tax_amount: 0.0, total: 0.0 }
    };

    let on_customer_select = {
        let mut code = customer_code.clone();
        let mut name = customer_name.clone();
        let mut dirty = is_dirty.clone();
        let cust_map = customer_map.clone();
        move |value: String| {
            name.set(cust_map.read().get(&value).map(|c| c.customer_name.clone()).unwrap_or_default());
            code.set(value);
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

    let save_order = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let c_code = customer_code.clone();
        let c_name = customer_name.clone();
        let its = items.clone();
        let o_date = order_date.clone();
        let nts = notes.clone();
        let mut dirty = is_dirty.clone();
        let navigator = navigator.clone();
        let api = api.clone();
        let cust_map = customer_map.clone();
        let it_map = item_map.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            if its.read().iter().filter(|li| !li.item_code.is_empty()).count() == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }
            let customer_id = cust_map.read().get(&*c_code.read()).map(|c| c.id).unwrap_or(0);
            if customer_id == 0 {
                toast.error("Validation Error", "Selected customer is not valid.");
                return;
            }
            saving.set(true);
            let mut toast = toast.clone();
            let nav = navigator.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let so_date = (*o_date.read()).map(|d| d.to_string()).unwrap_or_default();
            let notes_val = nts.read().clone();
            let order_items: Vec<SalesOrderItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| SalesOrderItemForm {
                    item_id: it_map.read().get(&li.item_code).map(|i| i.id).unwrap_or(0),
                    description: None,
                    quantity: li.quantity,
                    unit_price: li.unit_price,
                })
                .collect();
            spawn(async move {
                let form = SalesOrderForm { customer_id, so_date, warehouse_id: None, notes: if notes_val.is_empty() { None } else { Some(notes_val) }, items: order_items };
                match api.read().create_sales_order(&form).await {
                    Ok(so) => {
                        toast.success("Sales Order Created", &format!("SO {} created.", so.so_no));
                        saving.set(false); dirty.set(false);
                        nav.push("/sales/orders");
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create sales order: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = customer_code.clone();
        let mut its = items.clone();
        let mut o_date = order_date.clone();
        let mut d_date = delivery_date.clone();
        let mut nts = notes.clone();
        let mut disc_pct = discount_pct.clone();
        let mut tax_str = tax_rate_str.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        let cust_map = customer_map.clone();
        let it_map = item_map.clone();

        move |_| {
            if c_code.read().is_empty() {
                toast.error("Validation Error", "Please select a customer.");
                return;
            }
            if its.read().iter().filter(|li| !li.item_code.is_empty()).count() == 0 {
                toast.error("Validation Error", "Please add at least one item.");
                return;
            }
            let customer_id = cust_map.read().get(&*c_code.read()).map(|c| c.id).unwrap_or(0);
            if customer_id == 0 {
                toast.error("Validation Error", "Selected customer is not valid.");
                return;
            }
            saving.set(true);
            let mut toast = toast.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let so_date = (*o_date.read()).map(|d| d.to_string()).unwrap_or_default();
            let notes_val = nts.read().clone();
            let order_items: Vec<SalesOrderItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| SalesOrderItemForm {
                    item_id: it_map.read().get(&li.item_code).map(|i| i.id).unwrap_or(0),
                    description: None,
                    quantity: li.quantity,
                    unit_price: li.unit_price,
                })
                .collect();
            spawn(async move {
                let form = SalesOrderForm { customer_id, so_date, warehouse_id: None, notes: if notes_val.is_empty() { None } else { Some(notes_val) }, items: order_items };
                match api.read().create_sales_order(&form).await {
                    Ok(so) => {
                        toast.success("Sales Order Created", &format!("SO {} created. Creating another…", so.so_no));
                        c_code.set(String::new());
                        its.write().clear();
                        for _ in 0..MIN_ITEM_ROWS { its.write().push(LineItem::default()); }
                        let t = chrono::Local::now().date_naive();
                        o_date.set(Some(t));
                        d_date.set(Some(t + chrono::Duration::days(LEAD_TIME_DAYS)));
                        nts.set(String::new());
                        disc_pct.set(String::from("0"));
                        tax_str.set(format!("{}", DEFAULT_TAX_RATE));
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create sales order: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let dirty = is_dirty.clone();
        let nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/sales/orders"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/sales/orders"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page socreate-page",

            div { class: "socreate-header",
                div {
                    a { class: "socreate-back-link", href: "/sales/orders", "← Back to Sales Orders" }
                    h1 { "New Sales Order" }
                }
                if *is_dirty.read() {
                    span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" }
                }
            }

            // Section: Order Details
            div { class: "socreate-section",
                h2 { "Order Details" }
                div { class: "socreate-form-row",
                    div {
                        SearchableSelect {
                            options: customer_options.read().clone(),
                            selected_value: Some(customer_code.read().clone()),
                            on_select: on_customer_select,
                            placeholder: "Select customer…",
                            searchable: true,
                            class: "cb-input-group",
                        }
                    }
                    div {
                        FormInput {
                            label: "Order Date".to_string(),
                            value: (*order_date.read()).map(|d| d.to_string()).unwrap_or_default(),
                            r#type: InputType::Date,
                            oninput: move |v: String| {
                                if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                    order_date.set(Some(d));
                                    is_dirty.set(true);
                                }
                            },
                        }
                    }
                    div {
                        FormInput {
                            label: "Delivery Date".to_string(),
                            value: (*delivery_date.read()).map(|d| d.to_string()).unwrap_or_default(),
                            r#type: InputType::Date,
                            oninput: move |v: String| {
                                if let Ok(d) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
                                    delivery_date.set(Some(d));
                                    is_dirty.set(true);
                                }
                            },
                        }
                    }
                }
            }

            // Section: Items
            div { class: "socreate-section",
                h2 { "Items" }
                div { style: "overflow-x: auto;",
                    table { class: "socreate-items-table",
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

                                let on_item_select = {
                                    let mut its = items.clone();
                                    let mut dirty = is_dirty.clone();
                                    let id = item_data.id;
                                    let it_map = item_map.clone();
                                    move |value: String| {
                                        let looked_up = it_map.read().get(&value).map(|i| (i.item_name.clone(), i.selling_price));
                                        let mut w = its.write();
                                        if let Some(line) = w.iter_mut().find(|x| x.id == id) {
                                            line.item_code = value.clone();
                                            if let Some((name, price)) = looked_up {
                                                line.item_name = name;
                                                line.unit_price = price;
                                            }
                                        }
                                        dirty.set(true);
                                    }
                                };
                                let on_qty = {
                                    let mut its = items.clone(); let mut dirty = is_dirty.clone(); let id = item_data.id;
                                    move |v: String| { let q = v.parse::<f64>().unwrap_or(0.0).max(0.0); if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.quantity = q; } dirty.set(true); }
                                };
                                let on_price = {
                                    let mut its = items.clone(); let mut dirty = is_dirty.clone(); let id = item_data.id;
                                    move |v: String| { let p = v.parse::<f64>().unwrap_or(0.0).max(0.0); if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.unit_price = p; } dirty.set(true); }
                                };
                                let on_disc = {
                                    let mut its = items.clone(); let mut dirty = is_dirty.clone(); let id = item_data.id;
                                    move |v: String| { let d = v.parse::<f64>().unwrap_or(0.0).max(0.0).min(100.0); if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.discount_value = d; line.discount_type = "Percentage".to_string(); } dirty.set(true); }
                                };
                                let on_tax = {
                                    let mut its = items.clone(); let mut dirty = is_dirty.clone(); let id = item_data.id;
                                    move |v: String| { let t = v.parse::<f64>().unwrap_or(0.0).max(0.0); if let Some(line) = its.write().iter_mut().find(|x| x.id == id) { line.tax_rate = t; } dirty.set(true); }
                                };
                                let mut on_rem = remove_item.clone();
                                let rem_id = item_data.id;

                                rsx! {
                                    tr { key: "soitem-{item_data.id}",
                                        td { class: "socreate-item-num", "{idx + 1}" }
                                        td {
                                            SearchableSelect {
                                                options: item_options_signal.read().clone(),
                                                selected_value: (!item_data.item_code.is_empty()).then(|| item_data.item_code.clone()),
                                                on_select: on_item_select,
                                                placeholder: "Search item…",
                                                searchable: true,
                                            }
                                        }
                                        td { FormInput { value: if item_data.quantity == 0.0 { String::new() } else { format!("{:.0}", item_data.quantity) }, oninput: on_qty, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) } }
                                        td { FormInput { value: if item_data.unit_price == 0.0 { String::new() } else { format!("{:.2}", item_data.unit_price) }, oninput: on_price, r#type: InputType::Number, min: Some(0.0), step: Some(0.01) } }
                                        td { FormInput { value: if item_data.discount_value == 0.0 { String::new() } else { format!("{:.0}", item_data.discount_value) }, oninput: on_disc, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(1.0) } }
                                        td { FormInput { value: if item_data.tax_rate == 0.0 { String::new() } else { format!("{:.0}", item_data.tax_rate) }, oninput: on_tax, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) } }
                                        td { class: "socreate-item-amount", style: "width: 100px;",
                                            if !item_data.item_code.is_empty() { span { "PKR {net_amt:.2}" } } else { span { style: "color: var(--text-secondary); font-weight: 400;", "—" } }
                                        }
                                        td { class: "socreate-item-actions",
                                            button { class: "socreate-remove-btn", r#type: "button", onclick: move |_| on_rem(rem_id), title: "Remove item", "×" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
                div { class: "socreate-add-row",
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon: Some("+".to_string()), onclick: add_item, disabled: *is_saving.read(), "Add Item" }
                }
            }

            // Section: Discount & Tax
            div { class: "socreate-section",
                h2 { "Discount & Tax" }
                div { class: "socreate-discount-row",
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
            div { class: "socreate-section",
                h2 { "Totals" }
                div { class: "socreate-totals-grid",
                    StatCard { title: "Subtotal".to_string(), value: crate::calculations::formatting::format_currency(metrics.subtotal), variant: StatCardVariant::Default }
                    StatCard { title: "Discount".to_string(), value: crate::calculations::formatting::format_currency(metrics.discount_amount), variant: if metrics.discount_amount > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: format!("Tax ({:.0}%)", tax_rate_val), value: crate::calculations::formatting::format_currency(metrics.tax_amount), variant: StatCardVariant::Default }
                    StatCard { title: "Grand Total".to_string(), value: crate::calculations::formatting::format_currency(metrics.total), variant: StatCardVariant::Primary }
                }
            }

            // Section: Notes
            div { class: "socreate-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: { let mut n = notes.clone(); let mut dirty = is_dirty.clone(); move |v| { n.set(v); dirty.set(true); } },
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes or delivery instructions…".to_string()),
                }
            }

            // Action Bar
            div { class: "socreate-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save_order, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Order" }
            }

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
                    "You have unsaved changes. Are you sure you want to discard this sales order?"
                }
            }
        }
    }
}
