//! Production Order Create Page — Form to create a new production order.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::models::{Bom, Item, ProductionForm};
use dioxus::prelude::*;
use std::collections::HashMap;

const PAGE_CSS: &str = r##"
.prd-create-page {
    max-width: 800px;
    margin: 0 auto;
}
.prd-create-header {
    display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px;
}
.prd-create-header h1 {
    font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary);
}
.prd-back-link {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 13px; color: var(--accent); text-decoration: none; margin-bottom: 16px;
}
.prd-back-link:hover { text-decoration: underline; }
.prd-section {
    background: #fff; border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px); padding: 20px; margin-bottom: 16px;
}
.prd-section h2 {
    font-size: 15px; font-weight: 600; color: var(--text-primary);
    margin: 0 0 16px 0; padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}
.prd-form-row {
    display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap;
}
.prd-form-row > * { flex: 1; min-width: 180px; }
.prd-action-bar {
    display: flex; justify-content: flex-end; gap: 8px;
    margin-top: 20px; padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}
@media (max-width: 768px) {
    .prd-form-row { flex-direction: column; }
    .prd-form-row > * { min-width: 100%; }
    .prd-action-bar { flex-direction: column; }
}
"##;

#[component]
pub fn ProductionCreatePage() -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;

    let prd_no = use_signal(String::new);
    let item_to_produce = use_signal(String::new);
    let bom = use_signal(String::new);
    let planned_qty = use_signal(|| "100".to_string());
    let start_date = use_signal(String::new);
    let expected_end_date = use_signal(String::new);
    let notes = use_signal(String::new);

    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    // API-loaded data
    let item_map = use_signal(HashMap::<i64, Item>::new);
    let item_options_signal = use_signal(Vec::<SelectOption>::new);
    let bom_list = use_signal(Vec::<Bom>::new);
    let bom_options_signal = use_signal(Vec::<SelectOption>::new);

    // Load items and BOMs from API
    {
        let api = api.clone();
        let mut item_map = item_map.clone();
        let mut item_options_signal = item_options_signal.clone();
        let mut bom_list = bom_list.clone();
        let mut bom_options_signal = bom_options_signal.clone();
        use_effect(move || {
            let api = api.clone();
            let mut item_map = item_map.clone();
            let mut item_options_signal = item_options_signal.clone();
            let mut bom_list = bom_list.clone();
            let mut bom_options_signal = bom_options_signal.clone();
            spawn(async move {
                let client = api.read().clone();
                if let Ok(items) = client.list_items_catalog().await {
                    let mut map = HashMap::new();
                    let mut opts = Vec::new();
                    for item in &items {
                        map.insert(item.id, item.clone());
                        opts.push(SelectOption { value: item.id.to_string(), label: format!("{} ({})", item.item_name, item.item_code) });
                    }
                    item_map.set(map);
                    item_options_signal.set(opts);
                }
                if let Ok(boms) = client.list_boms().await {
                    let opts: Vec<SelectOption> = boms.iter()
                        .filter(|b| b.is_active)
                        .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                        .collect();
                    bom_list.set(boms);
                    bom_options_signal.set(opts);
                }
            });
        });
    }

    // Update BOM options when item changes
    {
        let item_id = item_to_produce.read().clone();
        let boms = bom_list.read().clone();
        let mut bom_opts = bom_options_signal.clone();
        let filtered: Vec<SelectOption> = if item_id.is_empty() {
            boms.iter().filter(|b| b.is_active)
                .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                .collect()
        } else {
            let item_id_num = item_id.parse::<i64>().unwrap_or(0);
            boms.iter().filter(|b| b.is_active && b.finished_item_id == item_id_num)
                .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                .collect()
        };
        bom_opts.set(filtered);
    }

    let bom_disabled = item_to_produce.read().is_empty();

    let on_item_change = {
        let mut item = item_to_produce.clone();
        let mut bom = bom.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| {
            item.set(v.clone());
            bom.set(String::new());
            dirty.set(true);
        }
    };

    let on_bom_change = {
        let mut b = bom.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { b.set(v); dirty.set(true); }
    };

    let on_qty_change = {
        let mut q = planned_qty.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { q.set(v); dirty.set(true); }
    };

    let on_start_change = {
        let mut d = start_date.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { d.set(v); dirty.set(true); }
    };

    let on_end_change = {
        let mut d = expected_end_date.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { d.set(v); dirty.set(true); }
    };

    let on_notes_change = {
        let mut n = notes.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { n.set(v); dirty.set(true); }
    };

    let validate = {
        let item = item_to_produce.clone();
        let qty = planned_qty.clone();
        let mut toast = toast.clone();
        move || -> bool {
            if item.read().is_empty() { toast.warning("Validation", "Please select an item to produce."); return false; }
            if qty.read().parse::<f64>().unwrap_or(0.0) <= 0.0 { toast.warning("Validation", "Quantity must be greater than 0."); return false; }
            true
        }
    };

    let save_prd = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut item = item_to_produce.clone();
        let mut bom_sig = bom.clone();
        let mut qty = planned_qty.clone();
        let mut nts = notes.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let item_id = item.read().parse::<i64>().unwrap_or(0);
            let bom_id = bom_sig.read().parse::<i64>().ok();
            let qty_val = qty.read().parse::<f64>().unwrap_or(100.0);
            let nts_val = nts.read().clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let api = api.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = ProductionForm {
                    output_item_id: item_id,
                    output_quantity: qty_val,
                    warehouse_id: 1,
                    bom_id,
                    overhead_cost: None,
                    notes: if nts_val.is_empty() { None } else { Some(nts_val) },
                    inputs: vec![],
                };
                match client.create_production(&form).await {
                    Ok(data) => {
                        let no = data["production_no"].as_str().unwrap_or("PRD-????");
                        toast.success("Production Order Created", &format!("{} has been created.", no));
                        dirty.set(false);
                        nav.push("/manufacturing/production");
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let save_and_new = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let mut item = item_to_produce.clone();
        let mut bom_sig = bom.clone();
        let mut qty = planned_qty.clone();
        let mut nts = notes.clone();
        let mut start = start_date.clone();
        let mut end = expected_end_date.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let api = api.clone();
        move |_| {
            if !validate() { return; }
            saving.set(true);
            let item_id = item.read().parse::<i64>().unwrap_or(0);
            let bom_id = bom_sig.read().parse::<i64>().ok();
            let qty_val = qty.read().parse::<f64>().unwrap_or(100.0);
            let nts_val = nts.read().clone();
            let mut toast = toast.clone();
            let mut saving = saving.clone();
            let mut dirty = dirty.clone();
            let api = api.clone();
            spawn(async move {
                let client = api.read().clone();
                let form = ProductionForm {
                    output_item_id: item_id,
                    output_quantity: qty_val,
                    warehouse_id: 1,
                    bom_id,
                    overhead_cost: None,
                    notes: if nts_val.is_empty() { None } else { Some(nts_val) },
                    inputs: vec![],
                };
                match client.create_production(&form).await {
                    Ok(data) => {
                        let no = data["production_no"].as_str().unwrap_or("PRD-????");
                        toast.success("Production Order Created", &format!("{} created. Creating another...", no));
                        item.set(String::new());
                        bom_sig.set(String::new());
                        qty.set("100".to_string());
                        start.set(String::new());
                        end.set(String::new());
                        nts.set(String::new());
                        dirty.set(false);
                        saving.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let mut dirty = is_dirty.clone();
        let mut nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/manufacturing/production"); }
        }
    };

    let confirm_discard = {
        let mut nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/manufacturing/production"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page prd-create-page",
            div { class: "prd-create-header",
                div {
                    a { class: "prd-back-link", href: "/manufacturing/production", "← Back to Production Orders" }
                    h1 { "New Production Order" }
                }
                if *is_dirty.read() {
                    span { style: "font-size: 12px; color: var(--warning); font-weight: 500;", "⚠ Unsaved changes" }
                }
            }

            div { class: "prd-section",
                h2 { "Order Information" }
                div { class: "prd-form-row",
                    FormInput {
                        label: Some("Production No".to_string()),
                        value: prd_no.read().clone(),
                        oninput: move |_| {},
                        r#type: InputType::Text,
                        disabled: true,
                        hint: Some("Auto-generated".to_string()),
                    }
                }
                div { class: "prd-form-row", style: "margin-top: 12px;",
                    SearchableSelect {
                        options: item_options_signal.read().clone(),
                        selected_value: Some(item_to_produce.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_item_change,
                        placeholder: "Select item to produce...",
                        searchable: true,
                        class: Some("cb-input-group".to_string()),
                    }
                }
            }

            div { class: "prd-section",
                h2 { "BOM & Quantities" }
                div { class: "prd-form-row",
                    SearchableSelect {
                        options: bom_options_signal.read().clone(),
                        selected_value: Some(bom.read().clone()).filter(|s| !s.is_empty()),
                        on_select: on_bom_change,
                        placeholder: if bom_disabled { "Select item first..." } else { "Select BOM..." },
                        searchable: true,
                        disabled: bom_disabled,
                        class: Some("cb-input-group".to_string()),
                    }
                    FormInput {
                        label: Some("Planned Quantity".to_string()),
                        value: planned_qty.read().clone(),
                        oninput: on_qty_change,
                        r#type: InputType::Number,
                        placeholder: Some("100".to_string()),
                        min: Some(1.0),
                        step: Some(1.0),
                    }
                }
            }

            div { class: "prd-section",
                h2 { "Schedule" }
                div { class: "prd-form-row",
                    FormInput {
                        label: Some("Start Date".to_string()),
                        value: start_date.read().clone(),
                        oninput: on_start_change,
                        r#type: InputType::Date,
                    }
                    FormInput {
                        label: Some("Expected End Date".to_string()),
                        value: expected_end_date.read().clone(),
                        oninput: on_end_change,
                        r#type: InputType::Date,
                    }
                }
            }

            div { class: "prd-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes_change,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes about this production order...".to_string()),
                    hint: Some("Internal notes.".to_string()),
                }
            }

            div { class: "prd-action-bar",
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
                    onclick: save_prd,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Production Order"
                }
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
                    "You have unsaved changes. Are you sure you want to discard this production order?"
                }
            }
        }
    }
}
