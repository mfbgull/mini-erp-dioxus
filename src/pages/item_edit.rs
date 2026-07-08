//! Item Edit Page — loads existing item and submits updates.
//! ponytail: reuses create form structure with pre-filled signals

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonSize, ButtonVariant, FormInput, InputType, use_toast};
use crate::models::ItemForm;
use dioxus::prelude::*;

const EDIT_CSS: &str = r##"
.item-edit-page { max-width: 800px; margin: 0 auto; }
.item-edit-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
.item-edit-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.item-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px; }
.item-section h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0 0 16px 0; padding-bottom: 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.item-form-row { display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap; }
.item-form-row > * { flex: 1; min-width: 180px; }
.item-type-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 10px; }
.item-type-chip { display: flex; align-items: center; gap: 8px; padding: 10px 14px; border: 1.5px solid var(--border-color, #e0e0e0); border-radius: 8px; cursor: pointer; transition: all 0.15s ease; font-size: 13px; color: var(--text-primary); background: #fff; user-select: none; }
.item-type-chip:hover { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.04); }
.item-type-chip-active { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.08); color: var(--accent, #4a90d9); }
.item-action-bar { display: flex; gap: 10px; justify-content: flex-end; margin-top: 24px; }
.item-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.item-loading .loading-spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: spin 0.8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 768px) { .item-form-row { flex-direction: column; } .item-form-row > * { min-width: 100%; } }
"##;

#[component]
pub fn ItemEditPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let parsed_id = id.parse::<i64>().unwrap_or(0);

    let item_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_item(parsed_id).await.ok()
        }
    });

    let item_code = use_signal(String::new);
    let item_name = use_signal(String::new);
    let category = use_signal(String::new);
    let uom = use_signal(String::new);
    let reorder_level = use_signal(|| String::new());
    let standard_cost = use_signal(|| String::new());
    let selling_price = use_signal(|| String::new());
    let notes = use_signal(String::new);
    let mut is_raw_material = use_signal(|| false);
    let mut is_finished_good = use_signal(|| false);
    let mut is_purchased = use_signal(|| false);
    let mut is_manufactured = use_signal(|| false);
    let is_active = use_signal(|| true);
    let is_saving = use_signal(|| false);
    let data_loaded = use_signal(|| false);

    // Pre-fill form signals when item data loads
    {
        let resource = item_resource.clone();
        let mut ic = item_code.clone();
        let mut iname = item_name.clone();
        let mut cat = category.clone();
        let mut u = uom.clone();
        let mut rl = reorder_level.clone();
        let mut sc = standard_cost.clone();
        let mut sp = selling_price.clone();
        let mut n = notes.clone();
        let mut raw = is_raw_material.clone();
        let mut fin = is_finished_good.clone();
        let mut pur = is_purchased.clone();
        let mut man = is_manufactured.clone();
        let mut act = is_active.clone();
        let mut dl = data_loaded.clone();
        use_effect(move || {
            if !*dl.read() {
                let guard = resource.read();
                if let Some(Some(ref i)) = &*guard {
                    ic.set(i.item_code.clone());
                    iname.set(i.item_name.clone());
                    cat.set(i.category.clone());
                    u.set(i.unit_of_measure.clone());
                    rl.set(i.reorder_level.to_string());
                    sc.set(i.standard_cost.to_string());
                    sp.set(i.selling_price.to_string());
                    n.set(i.description.clone());
                    raw.set(i.is_raw_material);
                    fin.set(i.is_finished_good);
                    pur.set(i.is_purchased);
                    man.set(i.is_manufactured);
                    act.set(i.is_active);
                    dl.set(true);
                }
            }
        });
    }

    let is_loading = item_resource.read().is_none();
    let item_data = item_resource.read().clone().flatten();

    if is_loading {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "item-edit-page",
                div { class: "item-loading",
                    div { class: "loading-spinner" }
                    span { "Loading item..." }
                }
            }
        };
    }

    if item_data.is_none() {
        return rsx! {
            style { "{EDIT_CSS}" }
            div { class: "item-edit-page",
                div { class: "item-loading",
                    h2 { style: "margin: 0;", "Item Not Found" }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { let _ = navigator.push("/inventory/items"); }, "\u{2190} Back to Items" }
                }
            }
        };
    }

    let validate = {
        let code = item_code.clone();
        let name = item_name.clone();
        move || -> bool {
            if code.read().trim().is_empty() {
                return false;
            }
            if name.read().trim().is_empty() {
                return false;
            }
            true
        }
    };

    let save = {
        let api = api.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut saving = is_saving.clone();
        let code = item_code.clone();
        let name = item_name.clone();
        let cat = category.clone();
        let u = uom.clone();
        let rl = reorder_level.clone();
        let sc = standard_cost.clone();
        let sp = selling_price.clone();
        let notes = notes.clone();
        let raw = is_raw_material.clone();
        let fin = is_finished_good.clone();
        let pur = is_purchased.clone();
        let man = is_manufactured.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let form = ItemForm {
                item_code: code.read().clone(),
                item_name: name.read().clone(),
                description: { let d = notes.read(); if d.is_empty() { None } else { Some(d.clone()) } },
                category: { let c = cat.read(); if c.is_empty() { None } else { Some(c.clone()) } },
                unit_of_measure: Some(u.read().clone()),
                reorder_level: rl.read().parse::<f64>().ok(),
                standard_cost: sc.read().parse::<f64>().ok(),
                selling_price: sp.read().parse::<f64>().ok(),
                is_raw_material: Some(*raw.read()),
                is_finished_good: Some(*fin.read()),
                is_purchased: Some(*pur.read()),
                is_manufactured: Some(*man.read()),
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let nav = nav.clone();
            let item_name = name.read().clone();
            let item_code = code.read().clone();
            let mut saving = saving.clone();
            spawn(async move {
                let client = api.with(|c| c.clone());
                match client.update_item(parsed_id, &form).await {
                    Ok(_) => {
                        toast.success("Item Updated", &format!("{} ({}) updated.", item_name, item_code));
                        nav.push(format!("/inventory/items/{}", parsed_id));
                    }
                    Err(e) => { toast.error("Error", &e); saving.set(false); }
                }
            });
        }
    };

    let raw_class = move || if *is_raw_material.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let fin_class = move || if *is_finished_good.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let pur_class = move || if *is_purchased.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };
    let man_class = move || if *is_manufactured.read() { "item-type-chip item-type-chip-active" } else { "item-type-chip" };

    let toggle_raw = move |_| { let val = *is_raw_material.read(); is_raw_material.set(!val); };
    let toggle_finished = move |_| { let val = *is_finished_good.read(); is_finished_good.set(!val); };
    let toggle_purchased = move |_| { let val = *is_purchased.read(); is_purchased.set(!val); };
    let toggle_manufactured = move |_| { let val = *is_manufactured.read(); is_manufactured.set(!val); };

    rsx! {
        style { "{EDIT_CSS}" }
        div { class: "page item-edit-page",
            div { class: "item-edit-header",
                h1 { "Edit Item" }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| { let _ = navigator.push(format!("/inventory/items/{}", parsed_id)); }, "\u{2190} Back" }
            }

            div { class: "item-section",
                h2 { "Item Information" }
                div { class: "item-form-row",
                    FormInput { label: Some("Item Code *".to_string()), value: "{item_code}", placeholder: "e.g. ITM-001", r#type: InputType::Text, oninput: { let mut ic = item_code.clone(); move |v| { ic.set(v); } } }
                    FormInput { label: Some("Item Name *".to_string()), value: "{item_name}", placeholder: "e.g. Premium Widget", r#type: InputType::Text, oninput: { let mut n = item_name.clone(); move |v| { n.set(v); } } }
                }
                div { class: "item-form-row",
                    FormInput { label: Some("Category".to_string()), value: "{category}", placeholder: "e.g. Widgets", r#type: InputType::Text, oninput: { let mut c = category.clone(); move |v| { c.set(v); } } }
                    FormInput { label: Some("Unit of Measure".to_string()), value: "{uom}", placeholder: "e.g. Nos, Kg", r#type: InputType::Text, oninput: { let mut u = uom.clone(); move |v| { u.set(v); } } }
                }
                div { class: "item-form-row",
                    FormInput { label: Some("Standard Cost".to_string()), value: "{standard_cost}", placeholder: "0.00", r#type: InputType::Number, oninput: { let mut s = standard_cost.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Selling Price".to_string()), value: "{selling_price}", placeholder: "0.00", r#type: InputType::Number, oninput: { let mut s = selling_price.clone(); move |v| { s.set(v); } } }
                    FormInput { label: Some("Reorder Level".to_string()), value: "{reorder_level}", placeholder: "0", r#type: InputType::Number, oninput: { let mut r = reorder_level.clone(); move |v| { r.set(v); } } }
                }
            }

            div { class: "item-section",
                h2 { "Item Type" }
                div { class: "item-type-grid",
                    div { class: "{raw_class()}", onclick: toggle_raw, span { "\u{1f331}" } span { "Raw Material" } }
                    div { class: "{fin_class()}", onclick: toggle_finished, span { "\u{2699}" } span { "Finished Good" } }
                    div { class: "{pur_class()}", onclick: toggle_purchased, span { "\u{1f4b3}" } span { "Purchased" } }
                    div { class: "{man_class()}", onclick: toggle_manufactured, span { "\u{2692}" } span { "Manufactured" } }
                }
            }

            div { class: "item-section",
                h2 { "Notes" }
                FormInput { value: "{notes}", r#type: InputType::TextArea, placeholder: Some("Optional notes...".to_string()), oninput: { let mut n = notes.clone(); move |v| { n.set(v); } } }
            }

            div { class: "item-action-bar",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| { let _ = navigator.push(format!("/inventory/items/{}", parsed_id)); }, disabled: *is_saving.read(), "Cancel" }
                Button { variant: ButtonVariant::Primary, onclick: save, loading: *is_saving.read(), icon: Some("\u{2713}".to_string()), "Save Changes" }
            }
        }
    }
}
