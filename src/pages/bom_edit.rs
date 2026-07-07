//! BOM Edit Page
//! ponytail: header fields + component items table, no "Save & New"

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonSize, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast};
use crate::models::{BomForm, BomItemForm, Item};
use dioxus::prelude::*;
use std::collections::HashMap;

const EDIT_CSS: &str = r##"
.bom-edit-page { max-width: 860px; margin: 0 auto; }
.bom-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.bom-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.bom-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.bom-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.bom-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.bom-form-row > * { flex: 1; min-width: 180px; }
.bom-comp-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.bom-comp-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.bom-comp-table thead th.text-right { text-align: right; }
.bom-comp-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); vertical-align: middle; }
.bom-comp-table tbody td.text-right { text-align: right; }
.bom-comp-table tbody input, .bom-comp-table tbody select { width: 100%; padding: 5px 8px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; font-size: 12px; box-sizing: border-box; }
.bom-comp-remove { padding: 4px 8px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 4px; background: #fff; cursor: pointer; font-size: 13px; color: var(--text-secondary); }
.bom-comp-remove:hover { border-color: #dc3545; color: #dc3545; }
.bom-total-row { display: flex; justify-content: flex-end; align-items: center; gap: 24px; padding: 12px 10px 0; margin-top: 8px; border-top: 2px solid var(--border-color, #e0e0e0); }
.bom-total-value { font-size: 18px; font-weight: 700; color: var(--text-primary); }
.bom-action-bar { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color, #e0e0e0); }
.bom-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.bom-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .bom-form-row { flex-direction: column; } .bom-form-row > * { min-width: 100%; } }
"##;

#[derive(Clone)]
struct EditCompItem {
    idx: u64,
    item_id: String,
    item_name: String,
    quantity: f64,
    unit_cost: f64,
}

#[component]
pub fn BomEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let bom_data = client.get_bom(parsed_id).await.ok();
            let items = client.list_items_catalog().await.unwrap_or_default();
            (bom_data, items)
        }
    });

    let (bom_data, all_items) = resource.read().clone().unwrap_or((None, vec![]));
    let item_map: HashMap<String, &Item> = all_items.iter().map(|i| (i.id.to_string(), i)).collect();
    let item_options: Vec<SelectOption> = all_items.iter().map(|i| SelectOption { value: i.id.to_string(), label: format!("{} - {}", i.item_code, i.item_name) }).collect();

    let bom_name = use_signal(String::new);
    let finished_item_id = use_signal(String::new);
    let quantity = use_signal(|| String::new());
    let description = use_signal(String::new);
    let mut comp_items = use_signal(Vec::<EditCompItem>::new);
    let next_idx = use_signal(|| 0u64);
    let saving = use_signal(|| false);
    let loaded = use_signal(|| false);
    let data_ready = use_signal(|| false);

    // Parse bom data when it arrives
    {
        let raw = bom_data.clone();
        let mut bn = bom_name.clone();
        let mut fi = finished_item_id.clone();
        let mut qty = quantity.clone();
        let mut desc = description.clone();
        let mut items = comp_items.clone();
        let mut nidx = next_idx.clone();
        let mut ld = loaded.clone();
        let mut dr = data_ready.clone();
        use_effect(move || {
            if let Some(ref val) = raw {
                if !*ld.read() {
                    let bom = &val["bom"];
                    bn.set(bom["bom_name"].as_str().unwrap_or("").to_string());
                    fi.set(bom["finished_item_id"].as_i64().unwrap_or(0).to_string());
                    qty.set(bom["quantity"].as_f64().unwrap_or(0.0).to_string());
                    desc.set(bom.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string());
                    let mut comps = Vec::new();
                    if let Some(arr) = val["items"].as_array() {
                        for (i, item) in arr.iter().enumerate() {
                            let q = item["quantity"].as_f64().unwrap_or(0.0);
                            let uc = item["unit_cost"].as_f64().unwrap_or(0.0);
                            comps.push(EditCompItem {
                                idx: i as u64,
                                item_id: item["item_id"].as_i64().unwrap_or(0).to_string(),
                                item_name: item["item_name"].as_str().unwrap_or("").to_string(),
                                quantity: q,
                                unit_cost: uc,
                            });
                        }
                        nidx.set(arr.len() as u64);
                    }
                    items.set(comps);
                    ld.set(true);
                    dr.set(true);
                }
            }
        });
    }

    if resource.read().is_none() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "bom-edit-page", div { class: "bom-loading", div { class: "loading-spinner" }, span { "Loading BOM..." } } } };
    }
    if !*data_ready.read() && bom_data.is_some() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "bom-edit-page", div { class: "bom-loading", div { class: "loading-spinner" }, span { "Preparing form..." } } } };
    }
    if bom_data.is_none() && resource.read().is_some() {
        return rsx! { style { "{EDIT_CSS}" } div { class: "bom-edit-page", div { class: "bom-loading", h2 { "BOM Not Found" }, Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/manufacturing/boms"); }, "\u{2190} Back" } } } };
    }

    let add_item = {
        let mut items = comp_items.clone();
        let mut nidx = next_idx.clone();
        move |_| {
            let idx = *nidx.read();
            items.write().push(EditCompItem { idx, item_id: String::new(), item_name: String::new(), quantity: 1.0, unit_cost: 0.0 });
            nidx.set(idx + 1);
        }
    };

    let mut remove_item = {
        let mut items = comp_items.clone();
        move |idx: u64| {
            items.write().retain(|i| i.idx != idx);
        }
    };

    let total_cost = comp_items.read().iter().map(|i| i.quantity * i.unit_cost).sum::<f64>();

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = saving.clone();
        let bn = bom_name.clone();
        let fi = finished_item_id.clone();
        let qty = quantity.clone();
        let desc = description.clone();
        let items = comp_items.clone();
        move |_| {
            saving.set(true);
            let item_id = fi.read().parse::<i64>().unwrap_or(0);
            let q = qty.read().parse::<f64>().unwrap_or(1.0);
            let bom_items: Vec<BomItemForm> = items.read().iter().filter(|i| !i.item_id.is_empty()).map(|i| BomItemForm {
                item_id: i.item_id.parse::<i64>().unwrap_or(0),
                quantity: i.quantity,
                unit_cost: Some(i.unit_cost),
            }).collect();
            let form = BomForm {
                bom_name: bn.read().clone(),
                finished_item_id: item_id,
                quantity: q,
                description: { let d = desc.read(); if d.is_empty() { None } else { Some(d.clone()) } },
                items: bom_items,
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let bn_display = bn.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_bom(parsed_id, &form).await {
                    Ok(_) => { toast.success("BOM Updated", &format!("BOM '{}' updated.", bn_display)); nav.push(format!("/manufacturing/boms/{}", parsed_id)); }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page bom-edit-page",
            div { class: "bom-edit-header", h1 { "Edit BOM" } }

            div { class: "bom-section",
                h2 { "BOM Information" }
                div { class: "bom-form-row",
                    FormInput { label: Some("BOM Name *".to_string()), value: "{bom_name}", placeholder: "e.g. Assembly A", r#type: InputType::Text, oninput: { let mut s = bom_name.clone(); move |v| { s.set(v); } } }
                    div {
                        label { style: "font-size:13px;font-weight:500;margin-bottom:4px;display:block;", "Finished Item *" }
                        SearchableSelect {
                            selected_value: Some(finished_item_id.read().clone()),
                            on_select: { let mut s = finished_item_id.clone(); move |v: String| { s.set(v); } },
                            options: item_options.clone(),
                            placeholder: "Select item...",
                            searchable: true,
                        }
                    }
                    FormInput { label: Some("Quantity *".to_string()), value: "{quantity}", placeholder: "e.g. 1", r#type: InputType::Number, oninput: { let mut s = quantity.clone(); move |v| { s.set(v); } } }
                }
                div { class: "bom-form-row",
                    FormInput { value: "{description}", r#type: InputType::TextArea, placeholder: Some("Optional description...".to_string()), oninput: { let mut s = description.clone(); move |v| { s.set(v); } } }
                }
            }

            div { class: "bom-section",
                h2 { "Components" }
                div { style: "display:flex;justify-content:flex-end;margin-bottom:8px;",
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, onclick: add_item, icon: Some("+".to_string()), "Add Component" }
                }
                table { class: "bom-comp-table",
                    thead {
                        tr {
                            th { "Item" }
                            th { style: "width:100px;", "Qty" }
                            th { style: "width:100px;", "Unit Cost" }
                            th { style: "width:40px;", "" }
                        }
                    }
                    tbody {
                        {
                            let rows: Vec<_> = comp_items.read().iter().map(|item| {
                                let idx = item.idx;
                                let item_id = item.item_id.clone();
                                let quantity = item.quantity;
                                let unit_cost = item.unit_cost;
                                let opts = item_options.clone();
                                rsx! {
                                    tr {
                                        td {
                                            select {
                                                value: "{item_id}",
                                                onchange: move |e: Event<FormData>| {
                                                    let new_id = e.value();
                                                    comp_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.item_id = new_id);
                                                },
                                                option { value: "", "Select item..." }
                                                for opt in &opts {
                                                    option { value: "{opt.value}", selected: opt.value == item_id, "{opt.label}" }
                                                }
                                            }
                                        }
                                        td {
                                            input { r#type: "number", value: "{quantity}", min: "0.01", step: "any",
                                                oninput: move |e: Event<FormData>| {
                                                    let v = e.value().parse::<f64>().unwrap_or(0.0);
                                                    comp_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.quantity = v);
                                                }
                                            }
                                        }
                                        td {
                                            input { r#type: "number", value: "{unit_cost}", min: "0", step: "any",
                                                oninput: move |e: Event<FormData>| {
                                                    let v = e.value().parse::<f64>().unwrap_or(0.0);
                                                    comp_items.write().iter_mut().find(|i| i.idx == idx).map(|i| i.unit_cost = v);
                                                }
                                            }
                                        }
                                        td {
                                            button { class: "bom-comp-remove", onclick: move |_| remove_item(idx), "\u{2716}" }
                                        }
                                    }
                                }
                            }).collect();
                            rows.into_iter().map(|r| r)
                        }
                    }
                }

                div { class: "bom-total-row",
                    span { class: "bom-total-value", "Total: {total_cost}" }
                }
            }

            div { class: "bom-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/manufacturing/boms/{}", parsed_id)); }, disabled: *saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *saving.read(), "Save Changes" }
            }
        }
    }
}
