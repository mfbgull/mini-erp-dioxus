//! Production Order Create Page — Form to create a new production order.
//! Only items with active BOMs appear in the item selector.
//! After selecting an item + quantity, a raw-materials availability table is shown.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, use_toast,
};
use crate::models::{Bom, BomItem, Item, ProductionForm, StockBalance};
use dioxus::prelude::*;
use std::collections::HashMap;

const PAGE_CSS: &str = r##"
.prd-create-page {
    max-width: 900px;
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
.prd-bom-hint {
    font-size: 12px; color: var(--text-secondary); margin-top: 8px;
    padding: 8px 12px; background: #f8f9fa; border-radius: 6px;
}
.prd-avail-table {
    width: 100%; border-collapse: collapse; font-size: 13px;
}
.prd-avail-table th {
    text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px;
    text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary);
    border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap;
}
.prd-avail-table th.text-right { text-align: right; }
.prd-avail-table td {
    padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}
.prd-avail-table td.text-right {
    text-align: right; font-family: monospace; font-size: 12px;
}
.prd-avail-table td.ok { color: #28a745; font-weight: 600; }
.prd-avail-table td.short { color: #dc3545; font-weight: 600; }
.prd-avail-table tr:hover { background: rgba(74, 144, 217, 0.03); }
.prd-avail-summary {
    display: flex; gap: 16px; margin-top: 12px; flex-wrap: wrap;
}
.prd-avail-badge {
    display: inline-flex; align-items: center; gap: 4px;
    padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600;
}
.prd-avail-badge.all-ok { background: #d4edda; color: #155724; }
.prd-avail-badge.has-short { background: #f8d7da; color: #721c24; }
.prd-loading { text-align: center; padding: 20px; color: var(--text-secondary); font-size: 13px; }
@media (max-width: 768px) {
    .prd-form-row { flex-direction: column; }
    .prd-form-row > * { min-width: 100%; }
    .prd-action-bar { flex-direction: column; }
}
"##;

/// Parsed row for the availability table.
#[derive(Clone)]
struct MaterialAvailability {
    item_name: String,
    item_code: String,
    qty_per_unit: f64,
    total_required: f64,
    available_stock: f64,
}

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

    // Raw-materials availability data
    let bom_items = use_signal(Vec::<BomItem>::new);
    let stock_map = use_signal(HashMap::<i64, f64>::new); // item_id -> total quantity
    let material_rows = use_signal(Vec::<MaterialAvailability>::new);
    let loading_materials = use_signal(|| false);

    // Load items and BOMs from API — only items with active BOMs appear
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
                // Load BOMs first so we know which items have active BOMs
                let active_bom_item_ids: Vec<i64> = if let Ok(boms) = client.list_boms().await {
                    let active: Vec<Bom> = boms.iter().filter(|b| b.is_active).cloned().collect();
                    let ids: Vec<i64> = active.iter().map(|b| b.finished_item_id).collect();
                    let opts: Vec<SelectOption> = active.iter()
                        .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                        .collect();
                    bom_list.set(active.clone());
                    bom_options_signal.set(opts);
                    ids
                } else {
                    vec![]
                };
                if let Ok(items) = client.list_items_catalog().await {
                    let mut map = HashMap::new();
                    let mut opts = Vec::new();
                    for item in &items {
                        map.insert(item.id, item.clone());
                        if active_bom_item_ids.contains(&item.id) {
                            opts.push(SelectOption { value: item.id.to_string(), label: format!("{} ({})", item.item_name, item.item_code) });
                        }
                    }
                    item_map.set(map);
                    item_options_signal.set(opts);
                }
            });
        });
    }

    // When item changes, auto-select the first active BOM for that item.
    // Must be a use_effect — setting signals inside a bare render block causes
    // an infinite re-render loop (each set dirties the component → re-render → set again).
    {
        let mut item_to_produce = item_to_produce.clone();
        let mut bom_list = bom_list.clone();
        let mut bom_sig = bom.clone();
        let mut bom_opts = bom_options_signal.clone();
        use_effect(move || {
            let item_id_str = item_to_produce.read().clone();
            let boms = bom_list.read().clone();
            if !item_id_str.is_empty() {
                let item_id_num = item_id_str.parse::<i64>().unwrap_or(0);
                let filtered: Vec<SelectOption> = boms.iter()
                    .filter(|b| b.is_active && b.finished_item_id == item_id_num)
                    .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                    .collect();
                bom_opts.set(filtered);
                // Auto-select the first matching BOM if current selection is empty or invalid
                let current_bom = bom_sig.read().clone();
                if current_bom.is_empty() || !boms.iter().any(|b| b.id.to_string() == current_bom && b.is_active && b.finished_item_id == item_id_num) {
                    if let Some(first) = boms.iter().find(|b| b.is_active && b.finished_item_id == item_id_num) {
                        bom_sig.set(first.id.to_string());
                    }
                }
            } else {
                bom_sig.set(String::new());
                let all_active: Vec<SelectOption> = boms.iter().filter(|b| b.is_active)
                    .map(|b| SelectOption { value: b.id.to_string(), label: format!("{} - {}", b.bom_no, b.bom_name) })
                    .collect();
                bom_opts.set(all_active);
            }
        });
    }

    let bom_disabled = item_to_produce.read().is_empty();

    // Load BOM details + stock when BOM or quantity changes
    {
        let api = api.clone();
        let mut bom_items_sig = bom_items.clone();
        let mut stock_map_sig = stock_map.clone();
        let mut material_rows_sig = material_rows.clone();
        let mut loading = loading_materials.clone();
        use_effect(move || {
            // Read signals here for dependency tracking, then pass cloned values to spawn
            let bom_id_str = bom.read().clone();
            let qty_str = planned_qty.read().clone();
            let api = api.clone();
            let mut bom_items_sig = bom_items_sig.clone();
            let mut stock_map_sig = stock_map_sig.clone();
            let mut material_rows_sig = material_rows_sig.clone();
            let mut loading = loading.clone();
            spawn(async move {
                let bom_id = match bom_id_str.parse::<i64>() {
                    Ok(id) if id > 0 => id,
                    _ => {
                        bom_items_sig.set(Vec::new());
                        material_rows_sig.set(Vec::new());
                        return;
                    }
                };
                let qty = qty_str.parse::<f64>().unwrap_or(0.0);
                if qty <= 0.0 {
                    material_rows_sig.set(Vec::new());
                    return;
                }
                loading.set(true);
                let client = api.read().clone();
                // Fetch BOM details and stock in parallel
                let bom_result = client.get_bom(bom_id).await;
                let stock_result = client.list_stock_balances().await;
                loading.set(false);
                if let Ok(data) = bom_result {
                    // Parse BOM items directly from JSON (same pattern as bom_edit.rs)
                    let mut parsed_items = Vec::new();
                    if let Some(arr) = data["items"].as_array() {
                        for item in arr {
                            parsed_items.push(BomItem {
                                id: item["id"].as_i64().unwrap_or(0),
                                bom_id: item["bom_id"].as_i64().unwrap_or(0),
                                item_id: item["item_id"].as_i64().unwrap_or(0),
                                item_name: item["item_name"].as_str().map(|s| s.to_string()),
                                item_code: item["item_code"].as_str().map(|s| s.to_string()),
                                quantity: item["quantity"].as_f64().unwrap_or(0.0),
                                unit_cost: item["unit_cost"].as_f64().unwrap_or(0.0),
                            });
                        }
                    }
                    bom_items_sig.set(parsed_items.clone());
                    // Build stock map
                    let mut s_map: HashMap<i64, f64> = HashMap::new();
                    if let Ok(balances) = stock_result {
                        for b in &balances {
                            *s_map.entry(b.item_id).or_insert(0.0) += b.quantity;
                        }
                    }
                    stock_map_sig.set(s_map.clone());
                    // Compute availability rows
                    let rows: Vec<MaterialAvailability> = parsed_items.iter().map(|bi| {
                        let total_req = bi.quantity * qty;
                        let avail = s_map.get(&bi.item_id).copied().unwrap_or(0.0);
                        MaterialAvailability {
                            item_name: bi.item_name.clone().unwrap_or_default(),
                            item_code: bi.item_code.clone().unwrap_or_default(),
                            qty_per_unit: bi.quantity,
                            total_required: total_req,
                            available_stock: avail,
                        }
                    }).collect();
                    material_rows_sig.set(rows);
                } else {
                    bom_items_sig.set(Vec::new());
                    material_rows_sig.set(Vec::new());
                }
            });
        });
    }

    let on_item_change = {
        let mut item = item_to_produce.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| {
            item.set(v);
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
        let bom_sig = bom.clone();
        let mat_rows = material_rows.clone();
        let mut toast = toast.clone();
        move || -> bool {
            if item.read().is_empty() { toast.warning("Validation", "Please select an item to produce."); return false; }
            if qty.read().parse::<f64>().unwrap_or(0.0) <= 0.0 { toast.warning("Validation", "Quantity must be greater than 0."); return false; }
            if !bom_sig.read().is_empty() {
                let rows = mat_rows.read();
                let short: Vec<String> = rows.iter()
                    .filter(|r| r.available_stock < r.total_required)
                    .map(|r| r.item_name.clone())
                    .collect();
                if !short.is_empty() {
                    toast.error("Insufficient Materials", &format!("Raw materials short: {}. Please adjust quantity or restock.", short.join(", ")));
                    return false;
                }
            }
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

    // Read material data for rendering
    let rows = material_rows.read();
    let is_loading = *loading_materials.read();
    let has_bom = !bom.read().is_empty();
    let qty_val: f64 = planned_qty.read().parse().unwrap_or(0.0);
    let show_materials = has_bom && qty_val > 0.0;
    let short_count = rows.iter().filter(|r| r.available_stock < r.total_required).count();
    let all_ok = short_count == 0 && !rows.is_empty();

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
                        placeholder: "Select item to produce (items with BOM only)...",
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

            // ── Raw Materials Availability ──
            if show_materials {
                div { class: "prd-section",
                    h2 { "Raw Materials Availability" }
                    if is_loading {
                        div { class: "prd-loading", "Loading BOM details..." }
                    } else if rows.is_empty() {
                        div { class: "prd-loading", "No raw materials found for this BOM." }
                    } else {
                        table { class: "prd-avail-table",
                            thead { tr {
                                th { "Material" }
                                th { "Code" }
                                th { class: "text-right", "Qty/Unit" }
                                th { class: "text-right", "Total Required" }
                                th { class: "text-right", "Available Stock" }
                                th { class: "text-right", "Status" }
                            } }
                            tbody {
                                {rows.iter().map(|r| {
                                    let sufficient = r.available_stock >= r.total_required;
                                    let status_class = if sufficient { "ok" } else { "short" };
                                    let status_text = if sufficient { "✓ Available" } else { "✗ Short" };
                                    rsx! {
                                        tr {
                                            td { "{r.item_name}" }
                                            td { style: "font-family: monospace; font-size: 12px;", "{r.item_code}" }
                                            td { class: "text-right", "{r.qty_per_unit:.2}" }
                                            td { class: "text-right", "{r.total_required:.2}" }
                                            td { class: "text-right", "{r.available_stock:.2}" }
                                            td { class: "text-right {status_class}", "{status_text}" }
                                        }
                                    }
                                })}
                            }
                        }
                        div { class: "prd-avail-summary",
                            if all_ok {
                                span { class: "prd-avail-badge all-ok", "✓ All raw materials available" }
                            } else if short_count > 0 {
                                span { class: "prd-avail-badge has-short", "✗ {short_count} material(s) insufficient" }
                            }
                        }
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
                    disabled: !all_ok && show_materials && !rows.is_empty(),
                    icon: Some("💾".to_string()),
                    "Save & New"
                }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: save_prd,
                    loading: *is_saving.read(),
                    disabled: !all_ok && show_materials && !rows.is_empty(),
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
