//! Direct Purchase Create Page — Form to create a direct purchase without a PO.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use crate::models::DirectPurchaseForm;
use dioxus::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

const PAGE_CSS: &str = r##"
.dp-create-page { max-width: 1000px; margin: 0 auto; }
.dp-create-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.dp-create-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.dp-back-link { display: inline-flex; align-items: center; gap: 4px; font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px; }
.dp-back-link:hover { text-decoration: underline; }
.dp-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.dp-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.dp-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.dp-form-row > * { flex: 1; min-width: 180px; }
.dp-items-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.dp-items-table th { text-align: left; padding: 8px 6px; font-weight: 600; font-size: 12px; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.03em; border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.dp-items-table td { padding: 6px; vertical-align: middle; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.dp-item-num { text-align: center; font-weight: 600; color: var(--text-secondary); font-size: 13px; width: 30px; }
.dp-item-cell-wide { min-width: 180px; }
.dp-item-cell-narrow { min-width: 70px; }
.dp-item-amount { text-align: right; font-weight: 600; font-variant-numeric: tabular-nums; white-space: nowrap; padding-right: 10px; }
.dp-item-actions { width: 40px; text-align: center; }
.dp-remove-btn { border: none; background: transparent; cursor: pointer; color: var(--danger, #dc3545); font-size: 16px; padding: 4px; border-radius: 4px; transition: background 0.15s; line-height: 1; }
.dp-remove-btn:hover { background: rgba(220, 53, 69, 0.1); }
.dp-add-row { margin-top: 10px; }
.dp-totals-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 16px; }
.dp-action-bar { display: flex; justify-content: flex-end; align-items: center; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
@media (max-width: 768px) { .dp-form-row { flex-direction: column; } .dp-form-row > * { min-width: 100%; } .dp-totals-grid { grid-template-columns: 1fr 1fr; } .dp-action-bar { flex-direction: column; } }
"##;

static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
struct LineItem {
    id: u64,
    item_code: String,
    item_name: String,
    quantity: f64,
    rate: f64,
    discount_pct: f64,
    tax_rate: f64,
}

impl Default for LineItem {
    fn default() -> Self {
        Self {
            id: NEXT_LINE_ID.fetch_add(1, Ordering::Relaxed),
            item_code: String::new(),
            item_name: String::new(),
            quantity: 1.0,
            rate: 0.0,
            discount_pct: 0.0,
            tax_rate: 16.0,
        }
    }
}

impl LineItem {
    fn line_total(&self) -> f64 {
        let gross = self.quantity * self.rate;
        let disc_amt = gross * (self.discount_pct / 100.0);
        let after_disc = gross - disc_amt;
        let tax_amt = after_disc * (self.tax_rate / 100.0);
        after_disc + tax_amt
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
pub fn DirectPurchaseCreatePage() -> Element {
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

    let line_items = use_signal(|| { let mut v = Vec::new(); for _ in 0..3 { v.push(LineItem::default()); } v });
    let supplier_code = use_signal(String::new);
    let supplier_name = use_signal(String::new);
    let purchase_date = use_signal(|| chrono::Local::now().date_naive().to_string());
    let discount_pct = use_signal(|| String::from("0"));
    let tax_rate_str = use_signal(|| String::from("16"));
    let notes = use_signal(String::new);
    let is_saving = use_signal(|| false);
    let is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);

    let resource_data = resource.read().clone().unwrap_or_default();
    let supplier_opts = build_supplier_options(&resource_data.0);
    let item_opts = build_item_options(&resource_data.1);

    fn compute_totals(items: &[LineItem]) -> (f64, f64, f64, f64) {
        let subtotal: f64 = items.iter().map(|li| li.quantity * li.rate).sum();
        let discount_amount: f64 = items.iter().map(|li| (li.quantity * li.rate) * (li.discount_pct / 100.0)).sum();
        let taxable = subtotal - discount_amount;
        (subtotal, discount_amount, taxable, taxable)
    }

    let (subtotal, discount_amt, taxable, _grand) = compute_totals(&line_items.read());
    let disc_pct: f64 = discount_pct.read().parse().unwrap_or(0.0);
    let tax_rate: f64 = tax_rate_str.read().parse().unwrap_or(0.0);
    let header_disc = subtotal * (disc_pct / 100.0);
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

    let on_date_change = {
        let mut d = purchase_date.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { d.set(v); dirty.set(true); }
    };

    let add_item = {
        let mut its = line_items.clone();
        let mut dirty = is_dirty.clone();
        move |_| { its.write().push(LineItem::default()); dirty.set(true); }
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
        let p_date = purchase_date.clone();
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
            let first_item = its.read().iter().find(|li| !li.item_code.is_empty()).cloned().unwrap_or_default();
            let supplier_name_val = c_name.read().clone();
            let purchase_date_val = p_date.read().clone();
            let notes_val = nts.read().clone();
            let item_id = first_item.item_code.parse::<i64>().unwrap_or(0);
            spawn(async move {
                let form = DirectPurchaseForm { item_id, warehouse_id: 1, quantity: first_item.quantity, unit_cost: first_item.rate, supplier_name: supplier_name_val, purchase_date: purchase_date_val, notes: if notes_val.is_empty() { None } else { Some(notes_val) } };
                match api.read().create_direct_purchase(&form).await {
                    Ok(body) => {
                        let pno = body["data"]["purchase_no"].as_str().unwrap_or("N/A");
                        toast.success("Direct Purchase Created", &format!("Purchase {} created.", pno));
                        saving.set(false); dirty.set(false);
                        nav.push("/purchases/direct");
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create purchase: {}", e));
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
        let p_date = purchase_date.clone();
        let nts = notes.clone();
        let mut disc_pct = discount_pct.clone();
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
            let first_item = its.read().iter().find(|li| !li.item_code.is_empty()).cloned().unwrap_or_default();
            let supplier_name_val = c_name.read().clone();
            let purchase_date_val = p_date.read().clone();
            let notes_val = nts.read().clone();
            let item_id = first_item.item_code.parse::<i64>().unwrap_or(0);
            spawn(async move {
                let form = DirectPurchaseForm { item_id, warehouse_id: 1, quantity: first_item.quantity, unit_cost: first_item.rate, supplier_name: supplier_name_val, purchase_date: purchase_date_val, notes: if notes_val.is_empty() { None } else { Some(notes_val) } };
                match api.read().create_direct_purchase(&form).await {
                    Ok(body) => {
                        let pno = body["data"]["purchase_no"].as_str().unwrap_or("N/A");
                        toast.success("Direct Purchase Created", &format!("Purchase {} created. Creating another…", pno));
                        c_code.set(String::new());
                        its.write().clear();
                        for _ in 0..3 { its.write().push(LineItem::default()); }
                        disc_pct.set(String::from("0"));
                        tr.set(String::from("16"));
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &format!("Failed to create purchase: {}", e));
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
        move |_| { if *dirty.read() { modal.set(true); } else { nav.push("/purchases/direct"); } }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/purchases/direct"); }
    };

    let cancel_discard = move |_| show_discard_modal.set(false);

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page dp-create-page",
            div { class: "dp-create-header",
                div {
                    a { class: "dp-back-link", href: "/purchases/direct", "← Back to Direct Purchases" }
                    h1 { "New Direct Purchase" }
                }
                if *is_dirty.read() { span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" } }
            }

            div { class: "dp-section",
                h2 { "Purchase Details" }
                div { class: "dp-form-row",
                    SearchableSelect { options: supplier_opts.clone(), selected_value: Some(supplier_code.read().clone()).filter(|s| !s.is_empty()), on_select: on_supplier_select, placeholder: "Select supplier…", searchable: true }
                    FormInput { label: Some("Purchase Date".to_string()), value: purchase_date.read().clone(), oninput: on_date_change, r#type: InputType::Date }
                }
            }

            div { class: "dp-section",
                h2 { "Items" }
                div { style: "overflow-x: auto;",
                    table { class: "dp-items-table",
                        thead { tr {
                            th { style: "width: 30px;", "#" }
                            th { style: "min-width: 180px;", "Item" }
                            th { style: "width: 70px;", "Qty" }
                            th { style: "width: 90px;", "Rate" }
                            th { style: "width: 70px;", "Disc %" }
                            th { style: "width: 60px;", "Tax %" }
                            th { style: "text-align: right; width: 100px;", "Amount" }
                            th { style: "width: 40px;" }
                        } }
                        tbody {
                            {line_items.read().iter().map(|li| {
                                let item = li.clone();
                                let idx = line_items.read().iter().position(|x| x.id == li.id).unwrap_or(0);
                                let amt = li.line_total();
                                rsx! {
                                    tr { key: "item-{li.id}",
                                        td { class: "dp-item-num", "{idx + 1}" }
                                        td { class: "dp-item-cell-wide",
                                            SearchableSelect { options: item_opts.clone(), selected_value: (!item.item_code.is_empty()).then(|| item.item_code.clone()), on_select: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let cat = resource_data.1.clone();
                                                move |v: String| { let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == item.id) { line.item_code = v.clone(); line.item_name = item_name_from_catalog(&cat, &v); line.rate = item_price_from_catalog(&cat, &v); } dirty.set(true); }
                                            }, placeholder: "Search item…", searchable: true }
                                        }
                                        td { class: "dp-item-cell-narrow",
                                            FormInput { value: if item.quantity == 0.0 { String::new() } else { format!("{:.0}", item.quantity) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.quantity = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) }
                                        }
                                        td { class: "dp-item-cell-narrow",
                                            FormInput { value: if item.rate == 0.0 { String::new() } else { format!("{:.2}", item.rate) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.rate = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(0.01) }
                                        }
                                        td { class: "dp-item-cell-narrow",
                                            FormInput { value: if item.discount_pct == 0.0 { String::new() } else { format!("{:.0}", item.discount_pct) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.discount_pct = val.max(0.0).min(100.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(1.0) }
                                        }
                                        td { class: "dp-item-cell-narrow",
                                            FormInput { value: if item.tax_rate == 0.0 { String::new() } else { format!("{:.0}", item.tax_rate) }, oninput: {
                                                let mut its = line_items.clone(); let mut dirty = is_dirty.clone(); let id = li.id;
                                                move |v: String| { let val = v.parse::<f64>().unwrap_or(0.0); let mut w = its.write(); if let Some(line) = w.iter_mut().find(|x| x.id == id) { line.tax_rate = val.max(0.0); } dirty.set(true); }
                                            }, r#type: InputType::Number, min: Some(0.0), step: Some(1.0) }
                                        }
                                        td { class: "dp-item-amount",
                                            if !item.item_code.is_empty() { span { "PKR {amt:.2}" } }
                                            else { span { style: "color: var(--text-secondary); font-weight: 400;", "—" } }
                                        }
                                        td { class: "dp-item-actions",
                                            button { class: "dp-remove-btn", r#type: "button", onclick: move |_| remove_item(item.id), title: "Remove item", "×" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
                div { class: "dp-add-row",
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, icon: Some("+".to_string()), onclick: add_item, disabled: *is_saving.read(), "Add Item" }
                }
            }

            div { class: "dp-section",
                h2 { "Discount & Tax" }
                div { style: "display: flex; gap: 16px; flex-wrap: wrap;",
                    FormInput { label: Some("Header Discount (%)".to_string()), value: discount_pct.read().clone(), oninput: { let mut d = discount_pct.clone(); let mut dirty = is_dirty.clone(); move |v| { d.set(v); dirty.set(true); } }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5) }
                    FormInput { label: Some("Tax Rate (%)".to_string()), value: tax_rate_str.read().clone(), oninput: { let mut t = tax_rate_str.clone(); let mut dirty = is_dirty.clone(); move |v| { t.set(v); dirty.set(true); } }, r#type: InputType::Number, min: Some(0.0), max: Some(100.0), step: Some(0.5) }
                }
            }

            div { class: "dp-section",
                h2 { "Totals" }
                div { class: "dp-totals-grid",
                    StatCard { title: "Subtotal".to_string(), value: format!("PKR {:.2}", subtotal), variant: StatCardVariant::Default }
                    StatCard { title: "Item Discount".to_string(), value: format!("PKR {:.2}", discount_amt), variant: if discount_amt > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: "Header Discount".to_string(), value: format!("PKR {:.2}", header_disc), variant: if header_disc > 0.0 { StatCardVariant::Warning } else { StatCardVariant::Default } }
                    StatCard { title: format!("Tax ({:.0}%)", tax_rate), value: format!("PKR {:.2}", total_tax), variant: StatCardVariant::Default }
                    StatCard { title: "Grand Total".to_string(), value: format!("PKR {:.2}", grand_total), variant: StatCardVariant::Primary }
                }
            }

            div { class: "dp-section",
                h2 { "Notes" }
                FormInput { value: notes.read().clone(), oninput: { let mut n = notes.clone(); let mut dirty = is_dirty.clone(); move |v| { n.set(v); dirty.set(true); } }, r#type: InputType::TextArea, placeholder: Some("Optional notes…".to_string()) }
            }

            div { class: "dp-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: open_discard, disabled: *is_saving.read(), "Discard" }
                Button { variant: ButtonVariant::Ghost, onclick: save_and_new, loading: *is_saving.read(), icon: Some("💾".to_string()), "Save & New" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *is_saving.read(), icon: Some("✓".to_string()), "Save Purchase" }
            }

            Modal { is_open: show_discard_modal, title: Some("Discard changes?".to_string()), size: ModalSize::Sm, close_on_backdrop: true, close_on_escape: true,
                footer: rsx! { Button { variant: ButtonVariant::Secondary, onclick: cancel_discard, "Cancel" } Button { variant: ButtonVariant::Danger, onclick: confirm_discard, "Discard" } },
                p { style: "margin: 0; color: var(--text-secondary); font-size: 14px;", "You have unsaved changes. Are you sure you want to discard this purchase?" }
            }
        }
    }
}
