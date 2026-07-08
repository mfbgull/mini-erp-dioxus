//! Purchase Order Edit Page
//! ponytail: header fields + PO items table, no "Save & New"

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonSize, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast};
use crate::models::{PurchaseOrderForm, PurchaseOrderItemForm};
use dioxus::prelude::*;

const EDIT_CSS: &str = r#"
.po-edit-page { max-width: 860px; margin: 0 auto; }
.po-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.po-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.po-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.po-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.po-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.po-form-row > * { flex: 1; min-width: 180px; }
.po-comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.po-comp-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.po-comp-table thead th.text-right { text-align: right; }
.po-comp-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); vertical-align: middle; }
.po-comp-table tbody td.text-right { text-align: right; }
.po-comp-table tbody input, .po-comp-table tbody select { width: 100%; padding: 5px 8px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; font-size: 12px; box-sizing: border-box; }
.po-comp-remove { padding: 4px 8px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; background: #fff; cursor: pointer; font-size: 13px; color: var(--text-secondary); }
.po-comp-remove:hover { border-color: #dc3545; color: #dc3545; }
.po-total-row { display: flex; justify-content: flex-end; align-items: center; gap: 24px; padding: 12px 10px 0; margin-top: 8px; border-top: 2px solid var(--border-color, #e0e0e0); }
.po-total-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
.po-action-bar { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
.po-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.po-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .po-form-row { flex-direction: column; } .po-form-row > * { min-width: 100%; } }
"#;

#[derive(Clone)]
struct EditPOItem {
    idx: u64,
    item_id: String,
    item_name: String,
    description: String,
    quantity: f64,
    unit_price: f64,
}

#[component]
pub fn PurchaseOrderEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    // Fetch PO + reference data
    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let po = client.get_purchase_order(parsed_id).await.ok();
            let suppliers = client.list_suppliers().await.unwrap_or_default();
            let warehouses = client.list_warehouses().await.unwrap_or_default();
            let items = client.list_items_catalog().await.unwrap_or_default();
            (po, suppliers, warehouses, items)
        }
    });

    let supplier_id = use_signal(String::new);
    let po_date = use_signal(String::new);
    let warehouse_id = use_signal(String::new);
    let notes = use_signal(String::new);
    let mut po_items = use_signal(Vec::<EditPOItem>::new);
    let mut next_idx = use_signal(|| 0u64);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);
    let data_ready = use_signal(|| false);

    // Pre-fill when PO data arrives
    {
        let res = resource.clone();
        let mut si = supplier_id.clone();
        let mut pd = po_date.clone();
        let mut wi = warehouse_id.clone();
        let mut nt = notes.clone();
        let mut items_sig = po_items.clone();
        let mut nidx = next_idx.clone();
        let mut ld = loaded.clone();
        let mut dr = data_ready.clone();
        use_effect(move || {
            if !*ld.read() {
                let guard = res.read();
                if let Some((ref po_data_opt, _, _, _)) = &*guard {
                    if let Some(ref val) = po_data_opt {
                        if !*ld.read() {
                            si.set(val["supplier_id"].as_i64().unwrap_or(0).to_string());
                            pd.set(val["po_date"].as_str().unwrap_or("").to_string());
                            wi.set(val.get("warehouse_id").and_then(|w| w.as_i64()).unwrap_or(0).to_string());
                            nt.set(val.get("notes").and_then(|n| n.as_str()).unwrap_or("").to_string());
                            let mut comps = Vec::new();
                            if let Some(arr) = val["items"].as_array() {
                                for (i, item) in arr.iter().enumerate() {
                                    comps.push(EditPOItem {
                                        idx: i as u64,
                                        item_id: item["item_id"].as_i64().unwrap_or(0).to_string(),
                                        item_name: item["item_name"].as_str().unwrap_or("").to_string(),
                                        description: item.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string(),
                                        quantity: item["quantity"].as_f64().unwrap_or(0.0),
                                        unit_price: item["unit_price"].as_f64().unwrap_or(0.0),
                                    });
                                }
                                nidx.set(arr.len() as u64);
                            }
                            items_sig.set(comps);
                            ld.set(true);
                            dr.set(true);
                        }
                    }
                }
            }
        });
    }

    let (po_data, suppliers, warehouses, all_items) = resource.read().clone().unwrap_or((None, vec![], vec![], vec![]));
    let supplier_options: Vec<SelectOption> = suppliers.iter().map(|s| SelectOption { value: s.id.to_string(), label: format!("{} - {}", s.supplier_code, s.supplier_name) }).collect();
    let warehouse_options: Vec<SelectOption> = warehouses.iter().map(|w| SelectOption { value: w.id.to_string(), label: format!("{} - {}", w.warehouse_code, w.warehouse_name) }).collect();
    let item_options: Vec<SelectOption> = all_items.iter().map(|i| SelectOption { value: i.id.to_string(), label: format!("{} - {}", i.item_code, i.item_name) }).collect();

    if resource.read().is_none() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "po-edit-page", div { class: "po-loading", div { class: "loading-spinner" }, span { "Loading purchase order..." } } } };
    }
    if !*data_ready.read() && po_data.is_some() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "po-edit-page", div { class: "po-loading", div { class: "loading-spinner" }, span { "Preparing form..." } } } };
    }
    if po_data.is_none() && resource.read().is_some() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "po-edit-page", div { class: "po-loading", h2 { "Purchase Order Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/purchases/orders"); }, "\u{2190} Back" } } } };
    }

    let add_item = move |_| {
        let idx = *next_idx.read();
        po_items.write().push(EditPOItem { idx, item_id: String::new(), item_name: String::new(), description: String::new(), quantity: 1.0, unit_price: 0.0 });
        next_idx.set(idx + 1);
    };

    let mut remove_item = move |idx: u64| {
        po_items.write().retain(|i| i.idx != idx);
    };

    let total = po_items.read().iter().map(|i| i.quantity * i.unit_price).sum::<f64>();

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let si = supplier_id.clone();
        let pd = po_date.clone();
        let wi = warehouse_id.clone();
        let nt = notes.clone();
        let items_sig = po_items.clone();
        move |_| {
            saving.set(true);
            let form_items: Vec<PurchaseOrderItemForm> = items_sig.read().iter().filter(|i| !i.item_id.is_empty()).map(|i| PurchaseOrderItemForm {
                item_id: i.item_id.parse::<i64>().unwrap_or(0),
                description: { let d = i.description.clone(); if d.is_empty() { None } else { Some(d) } },
                quantity: i.quantity,
                unit_price: i.unit_price,
            }).collect();
            let form = PurchaseOrderForm {
                supplier_id: si.read().parse::<i64>().unwrap_or(0),
                po_date: pd.read().clone(),
                warehouse_id: wi.read().parse::<i64>().ok().filter(|&v| v > 0),
                notes: { let n = nt.read(); if n.is_empty() { None } else { Some(n.clone()) } },
                items: form_items,
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_purchase_order(parsed_id, &form).await {
                    Ok(_) => { toast.success("PO Updated", "Purchase order updated successfully."); nav.push(format!("/purchases/orders/{}", parsed_id)); }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page po-edit-page",
            div { class: "po-edit-header", h1 { "Edit Purchase Order" } }

            div { class: "po-section",
                h2 { "Purchase Order Information" }
                div { class: "po-form-row",
                    div {
                        label { style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Supplier *" }
                        SearchableSelect {
                            selected_value: Some(supplier_id.read().clone()),
                            on_select: { let mut s = supplier_id.clone(); move |v: String| { s.set(v); } },
                            options: supplier_options,
                            placeholder: "Select supplier...",
                            searchable: true,
                        }
                    }
                    FormInput { label: Some("PO Date *".to_string()), value: "{po_date}", placeholder: "2024-01-15", r#type: InputType::Date, oninput: { let mut s = po_date.clone(); move |v| { s.set(v); } } }
                    div {
                        label { style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Warehouse" }
                        SearchableSelect {
                            selected_value: Some(warehouse_id.read().clone()).filter(|v| !v.is_empty() && v != "0"),
                            on_select: { let mut s = warehouse_id.clone(); move |v: String| { s.set(v); } },
                            options: warehouse_options,
                            placeholder: "Select warehouse...",
                            searchable: true,
                        }
                    }
                }
                div { class: "po-form-row",
                    FormInput { value: "{notes}", r#type: InputType::TextArea, placeholder: Some("Optional notes...".to_string()), oninput: { let mut s = notes.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "po-section",
                h2 { "Items" }
                div { style: "display:flex;justify-content:flex-end;margin-bottom:8px;",
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, onclick: add_item, icon: Some("+".to_string()), "Add Item" }
                }
                table { class: "po-comp-table",
                    thead {
                        tr {
                            th { "Item" }
                            th { style: "width:100px;", "Qty" }
                            th { style: "width:100px;", "Unit Price" }
                            th { style: "width:100px;", "Amount" }
                            th { style: "width:40px;", "" }
                        }
                    }
                    tbody {
                        {
                            let rows: Vec<_> = po_items.read().iter().map(|item| {
                                let idx = item.idx;
                                let item_id = item.item_id.clone();
                                let quantity = item.quantity;
                                let unit_price = item.unit_price;
                                let amount = item.quantity * item.unit_price;
                                rsx! {
                                    tr {
                                        td {
                                            select {
                                                value: "{item_id}",
                                                onchange: move |e: Event<FormData>| {
                                                    let new_id = e.value();
                                                    po_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.item_id = new_id);
                                                },
                                                option { value: "", "Select item..." }
                                                for opt in &item_options {
                                                    option { value: "{opt.value}", selected: opt.value == item_id, "{opt.label}" }
                                                }
                                            }
                                        }
                                        td {
                                            input { r#type: "number", value: "{quantity}", min: "0.01", step: "any",
                                                oninput: move |e: Event<FormData>| {
                                                    let v = e.value().parse::<f64>().unwrap_or(0.0);
                                                    po_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.quantity = v);
                                                }
                                            }
                                        }
                                        td {
                                            input { r#type: "number", value: "{unit_price}", min: "0", step: "any",
                                                oninput: move |e: Event<FormData>| {
                                                    let v = e.value().parse::<f64>().unwrap_or(0.0);
                                                    po_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.unit_price = v);
                                                }
                                            }
                                        }
                                        td { class: "text-right", "{amount:.2}" }
                                        td {
                                            button { class: "po-comp-remove", onclick: move |_| remove_item(idx), "\u{2716}" }
                                        }
                                    }
                                }
                            }).collect();
                            rows.into_iter().map(|r| r)
                        }
                    }
                }
                div { class: "po-total-row",
                    span { class: "po-total-value", "Total: {total:.2}" }
                }
            }

            div { class: "po-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/purchases/orders/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
