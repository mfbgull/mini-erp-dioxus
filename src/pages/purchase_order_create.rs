//! Purchase Order Create Page — Form to create a new purchase order.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use crate::models::{PurchaseOrderForm, PurchaseOrderItemForm};
use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

const PAGE_CSS: &str = r##"
.po-create-page { max-width: 1000px; margin: 0 auto; }
.po-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.po-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.po-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.po-back-link:hover { text-decoration: underline; }
.po-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.po-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.po-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.po-form-row > * { flex: 1; min-width: 180px; }
.po-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.po-items-table th { text-align: left; padding: 8px 6px; font-weight: 600; font-size: 12px; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.03em; border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.po-items-table td { padding: 6px; vertical-align: middle; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.po-item-num { text-align: center; font-weight: 600; color: var(--text-secondary); font-size: 13px; width: 30px; }
.po-item-cell-wide { min-width: 180px; }
.po-item-cell-narrow { min-width: 70px; }
.po-item-amount { text-align: right; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; padding-right: 10px; }
.po-item-actions { width: 40px; text-align: center; }
.po-remove-btn { border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 16px; padding: 4px; border-radius: 4px; transition: background 0.15s; line-height: 1; }
.po-remove-btn:hover { background: rgba(220, 53, 69, 0.1); }
.po-add-row { margin-top: 10px; }
.po-totals-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px; }
.po-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
@media (max-width: 768px) { .po-form-row { flex-direction: column; } .po-form-row > * { min-width: 100%; } .po-totals-grid { grid-template-columns: 1fr 1fr; } .po-action-bar { flex-direction: column; } }
"##;

static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
struct PoLineItem {
    id: u64,
    item_code: String,
    item_name: String,
    quantity: f64,
    rate: f64,
    discount_pct: f64,
    tax_rate: f64,
}

impl Default for PoLineItem {
    fn default() -> Self {
        Self {
            id: NEXT_LINE_ID.fetch_add(1, Ordering::Relaxed),
            item_code: String::new(), item_name: String::new(),
            quantity: 1.0, rate: 0.0, discount_pct: 0.0, tax_rate: 16.0,
        }
    }
}

impl PoLineItem {
    fn line_total(&self) -> f64 {
        let gross = self.quantity * self.rate;
        let disc_amt = gross * (self.discount_pct / 100.0);
        let after_disc = gross - disc_amt;
        after_disc + (after_disc * (self.tax_rate / 100.0))
    }
}

fn build_supplier_options(suppliers: &[crate::models::Supplier]) -> Vec<SelectOption> {
    suppliers.iter().map(|s| SelectOption { value: s.id.to_string(), label: format!("{} - {}", s.supplier_code, s.supplier_name) }).collect()
}

fn build_item_options(items: &[crate::models::Item]) -> Vec<SelectOption> {
    items.iter().map(|i| SelectOption { value: i.id.to_string(), label: format!("{} - {}", i.item_code, i.item_name) }).collect()
}

fn item_price_from_catalog(items: &[crate::models::Item], code: &str) -> f64 {
    items.iter().find(|i| i.id.to_string() == code).map(|i| i.standard_cost).unwrap_or(0.0)
}

fn item_name_from_catalog(items: &[crate::models::Item], code: &str) -> String {
    items.iter().find(|i| i.id.to_string() == code).map(|i| i.item_name.clone()).unwrap_or_default()
}

#[component]
pub fn PurchaseOrderCreatePage() -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let suppliers = client.list_suppliers().await.unwrap_or_default();
            let items = client.list_items_catalog().await.unwrap_or_default();
            (suppliers, items)
        }
    });

    let line_items = use_signal(|| { let mut v = Vec::new(); for _ in 0..3 { v.push(PoLineItem::default()); } v });
    let supplier_code = use_signal(String::new);
    let supplier_name = use_signal(String::new);
    let order_date = use_signal(|| chrono::Local::now().date_naive().to_string());
    let expected_date = use_signal(|| (chrono::Local::now().date_naive() + chrono::Duration::days(14)).to_string());
    let discount_pct = use_signal(|| String::from("0"));
    let tax_rate_str = use_signal(|| String::from("16"));
    let notes = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);

    let resource_data = resource.read().clone().unwrap_or_default();
    let supplier_opts = build_supplier_options(&resource_data.0);
    let item_opts = build_item_options(&resource_data.1);

    let subtotal: f64 = line_items.read().iter().map(|li| li.quantity * li.rate).sum();
    let discount_amt: f64 = line_items.read().iter().map(|li| (li.quantity * li.rate) * (li.discount_pct / 100.0)).sum();
    let taxable = subtotal - discount_amt;
    let disc_pct: f64 = discount_pct.read().parse().unwrap_or(0.0);
    let tax_rate: f64 = tax_rate_str.read().parse().unwrap_or(0.0);
    let header_disc = taxable * (disc_pct / 100.0);
    let after_header = taxable - header_disc;
    let total_tax = after_header * (tax_rate / 100.0);
    let grand_total = after_header + total_tax;

    let on_supplier_select = {
        let mut code = supplier_code.clone();
        let mut name = supplier_name.clone();
        let mut dirty = is_dirty.clone();
        let opts = supplier_opts.clone();
        move |v: String| { code.set(v.clone()); name.set(opts.iter().find(|o| o.value == v).map(|o| o.label.clone()).unwrap_or_default()); dirty.set(true); }
    };

    let add_item = {
        let mut its = line_items.clone();
        let mut dirty = is_dirty.clone();
        move |_| { its.write().push(PoLineItem::default()); dirty.set(true); }
    };

    let mut remove_item = {
        let mut its = line_items.clone();
        let mut dirty = is_dirty.clone();
        move |id: u64| { its.write().retain(|li| li.id != id); dirty.set(true); }
    };

    let save = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = supplier_code.clone();
        let c_name = supplier_name.clone();
        let its = line_items.clone();
        let o_date = order_date.clone();
        let nts = notes.clone();
        let mut dirty = is_dirty.clone();
        let nav = navigator.clone();
        let api = api.clone();
        move |_| {
            if c_code.read().is_empty() { toast.error("Validation Error", "Please select a supplier."); return; }
            let filled = its.read().iter().filter(|li| !li.item_code.is_empty()).count();
            if filled == 0 { toast.error("Validation Error", "Please add at least one item."); return; }
            saving.set(true);
            let mut toast = toast.clone();
            let nav = nav.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let supplier_id = c_code.read().parse::<i64>().unwrap_or(0);
            let po_date = o_date.read().clone();
            let notes_val = nts.read().clone();
            let order_items: Vec<PurchaseOrderItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| PurchaseOrderItemForm {
                    item_id: li.item_code.parse::<i64>().unwrap_or(0),
                    description: None,
                    quantity: li.quantity,
                    unit_price: li.rate,
                })
                .collect();
            spawn(async move {
                let form = PurchaseOrderForm { supplier_id, po_date, warehouse_id: None, notes: if notes_val.is_empty() { None } else { Some(notes_val) }, items: order_items };
                match api.read().create_purchase_order(&form).await {
                    Ok(body) => {
                        let po_no = body["data"]["po_no"].as_str().unwrap_or("N/A");
                        toast.success("PO Created", &format!("PO {} created.", po_no));
                        saving.set(false); dirty.set(false);
                        nav.push("/purchases/orders");
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create PO: {}", e));
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut c_code = supplier_code.clone();
        let c_name = supplier_name.clone();
        let mut its = line_items.clone();
        let o_date = order_date.clone();
        let nts = notes.clone();
        let mut dp = discount_pct.clone();
        let mut tr = tax_rate_str.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        move |_| {
            if c_code.read().is_empty() { toast.error("Validation Error", "Please select a supplier."); return; }
            let filled = its.read().iter().filter(|li| !li.item_code.is_empty()).count();
            if filled == 0 { toast.error("Validation Error", "Please add at least one item."); return; }
            saving.set(true);
            let mut toast = toast.clone();
            let api = api.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let supplier_id = c_code.read().parse::<i64>().unwrap_or(0);
            let po_date = o_date.read().clone();
            let notes_val = nts.read().clone();
            let order_items: Vec<PurchaseOrderItemForm> = its.read().iter()
                .filter(|li| !li.item_code.is_empty())
                .map(|li| PurchaseOrderItemForm {
                    item_id: li.item_code.parse::<i64>().unwrap_or(0),
                    description: None,
                    quantity: li.quantity,
                    unit_price: li.rate,
                })
                .collect();
            spawn(async move {
                let form = PurchaseOrderForm { supplier_id, po_date, warehouse_id: None, notes: if notes_val.is_empty() { None } else { Some(notes_val) }, items: order_items };
                match api.read().create_purchase_order(&form).await {
                    Ok(body) => {
                        let po_no = body["data"]["po_no"].as_str().unwrap_or("N/A");
                        toast.success("PO Created", &format!("PO {} created. Creating another…", po_no));
                        c_code.set(String::new());
                        its.write().clear();
                        for _ in 0..3 { its.write().push(PoLineItem::default()); }
                        dp.set(String::from("0")); tr.set(String::from("16"));
                        saving.set(false); dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create PO: {}", e));
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
        move |_| { if *dirty.read() { modal.set(true); } else { nav.push("/purchases/orders"); } }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/purchases/orders"); }
    };

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page po-create-page",
            div { class: "po-create-header",
                div {
                    a { class: "po-back-link", href: "/purchases/orders", "← Back to Purchase Orders" }
                    h1 { "New Purchase Order" }
                }
                if *is_dirty.read() { span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" } }
            }

            div { class: "po-section",
                h2 { "Order Details" }
                div { class: "po-form-row",
                    SearchableSelect { options: supplier_opts.clone(), selected_value: Some(supplier_code.read().clone()).filter(|s| !s.is_empty()), on_select: on_supplier_select, placeholder: "Select supplier…", searchable: true }
                    FormInput { label: Some("Order Date".to_string()), value: order_date.read().clone(), oninput: { let mut d = order_date.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } }, r#type: InputType::Date }
                    FormInput { label: Some("Expected Delivery".to_string()), value: expected_date.read().clone(), oninput: { let mut d = expected_date.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } }, r#type: InputType::Date }
                }
            }

            div { class: "po-section",
                h2 { "Line Items" }
                div { style: "overflow-x: auto;",
                    table { class: "po-items-table",
                        thead { tr {
                            th { style: "width: 30px;", "#" } th { style: "min-width: 180px;", "Item" } th { style: "width: 70px;", "Qty" } th { style: "width: 90px;", "Rate" } th { style: "width: 70px;", "Disc %" } th { style: "width: 60px;", "Tax %" } th { style: "text-align: right; width: 100px;", "Amount" } th { style: "width: 40px;" }
                        } }
                        tbody {
                            {line_items.read().iter().map(|li| {
                                let item = li.clone();
                                let idx = line_items.read().iter().position(|x| x.id == li.id).unwrap_or(0);
                                let amt = li.line_total();
                                rsx! {
                                    tr { key: "po-item-{li.id}",
                                        td { class: "po-item-num", "{idx + 1}" }
                                        td { class: "po-item-cell-wide",
                                            SearchableSelect { options: item_opts.clone(), selected_value: (!item.item_code.is_empty()).then(|| item.item_code.clone()), on_select: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let cat = resource_data.1.clone();
                                                move |v: String| { let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == item.id) { line.item_code = v.clone(); line.item_name = item_name_from_catalog(&cat, &v); line.rate = item_price_from_catalog(&cat, &v); } dirty.set(true); }
                                            }, placeholder: "Search item…", searchable: true }
                                        }
                                        td { class: "po-item-cell-narrow",
                                            FormInput { value: if item.quantity == 0.0 { String::new() } else { format!("{:.0}", item.quantity) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.quantity = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) }
                                        }
                                        td { class: "po-item-cell-narrow",
                                            FormInput { value: if item.rate == 0.0 { String::new() } else { format!("{:.2}", item.rate) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.rate = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(0.01) }
                                        }
                                        td { class: "po-item-cell-narrow",
                                            FormInput { value: if item.discount_pct == 0.0 { String::new() } else { format!("{:.0}", item.discount_pct) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.discount_pct = val.max(0.0).min(100.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(1.0) }
                                        }
                                        td { class: "po-item-cell-narrow",
                                            FormInput { value: if item.tax_rate == 0.0 { String::new() } else { format!("{:.0}", item.tax_rate) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.tax_rate = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) }
                                        }
                                        td { class: "po-item-amount",
                                            if !item.item_code.is_empty() { span { "PKR {amt:.2}" } }
                                            else { span { style: "color: var(--text-secondary); font-weight: 400;", "—" } }
                                        }
                                        td { class: "po-item-actions",
                                            button { class: "po-remove-btn", r#type: "button", onclick: move |_| remove_item(item.id), title: "Remove item", "×" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
                div { class: "po-add-row",
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon: Some("+".to_string()), onclick: add_item, disabled: *is_saving.read(), "Add Item" }
                }
            }

            div { class: "po-section",
                h2 { "Discount & Tax" }
                div { style: "display: flex; gap: 16px; flex-wrap: wrap;",
                    FormInput { label: Some("Header Discount (%)".to_string()), value: discount_pct.read().clone(), oninput: { let mut d = discount_pct.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5) }
                    FormInput { label: Some("Tax Rate (%)".to_string()), value: tax_rate_str.read().clone(), oninput: { let mut t = tax_rate_str.clone(); let mut dirty = is_dirty.clone(); move |v| { t.set(v); dirty.set(true); } }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5) }
                }
            }

            div { class: "po-section",
                h2 { "Totals" }
                div { class: "po-totals-grid",
                    StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.2}", subtotal), variant: StatCardVariant::Default }
                    StatCard { title: "Item Discount".to_string(), value: format!("PKR {:.2}", discount_amt), variant: if discount_amt > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: "Header Discount".to_string(), value: format!("PKR {:.2}", header_disc), variant: if header_disc > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: format!("Tax ({:.0}%)", tax_rate), value: format!("PKR {:.2}", total_tax), variant: StatCardVariant::Default }
                    StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.2}", grand_total), variant: StatCardVariant::Primary }
                }
            }

            div { class: "po-section",
                h2 { "Notes" }
                FormInput { value: notes.read().clone(), oninput: { let mut n = notes.clone(); let mut dirty = is_dirty.clone(); move |v| { n.set(v); dirty.set(true); } }, r#type: InputType::TextArea, placeholder: Some("Optional terms or notes…".to_string()) }
            }

            div { class: "po-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save PO" }
            }

            Modal { is_open: show_discard_modal, title: Some("Discard changes?".to_string()), size: ModalSize::Sm, close_on_backdrop: true, close_on_escape: true,
                footer: rsx! { Button { variant: ButtonVariant::Secondary, onclick: move |_| show_discard_modal.set(false), "Cancel" } Button { variant: ButtonVariant::Danger, onclick: confirm_discard, "Discard" } },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "You have unsaved changes. Discard this purchase order?" }
            }
        }
    }
}
