//! Stock Movement Create Page — Form to create a new stock movement
//! (receipt, issue, adjustment, or transfer) using common UI components.

use crate::components::common::{
    Button, ButtonSize, ButtonVariant, FormInput, InputType, Modal, ModalSize,
    SearchableSelect, SelectOption, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;
use crate::auth::use_auth;
use crate::models::{Item, StockMovementForm, Warehouse};
use std::collections::HashMap;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.stock-move-create-page {
    max-width: 800px;
    margin: 0 auto;
}

.stock-move-create-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 20px;
}

.stock-move-create-header h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
}

.stock-move-back-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent);
    text-decoration: none;
    margin-bottom: 16px;
}

.stock-move-back-link:hover { text-decoration: underline; }

.stock-move-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.stock-move-section h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0 0 16px 0;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.stock-move-form-row {
    display: flex;
    gap: 16px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.stock-move-form-row > * {
    flex: 1;
    min-width: 180px;
}

.stock-move-action-bar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
}

@media (max-width: 768px) {
    .stock-move-form-row { flex-direction: column; }
    .stock-move-form-row > * { min-width: 100%; }
    .stock-move-action-bar { flex-direction: column; }
}
"##;

// ============================================================================
// Helpers
// ============================================================================

fn movement_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "IN".to_string(), label: "IN — Receipt".to_string() },
        SelectOption { value: "OUT".to_string(), label: "OUT — Issue".to_string() },
        SelectOption { value: "ADJUSTMENT".to_string(), label: "ADJUSTMENT".to_string() },
        SelectOption { value: "TRANSFER".to_string(), label: "TRANSFER".to_string() },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn StockMovementCreatePage() -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();

    // ── API-loaded data ──
    let api = use_auth().api;
    let mut item_map = use_signal(HashMap::<i64, Item>::new);
    let mut warehouse_map = use_signal(HashMap::<i64, Warehouse>::new);
    let mut item_options_signal = use_signal(Vec::<SelectOption>::new);
    let mut warehouse_options_signal = use_signal(Vec::<SelectOption>::new);

    let load_api = api.clone();
    use_effect(move || {
        let client = load_api.read().clone();
        let mut item_map = item_map.clone();
        let mut warehouse_map = warehouse_map.clone();
        let mut item_opts = item_options_signal.clone();
        let mut wh_opts = warehouse_options_signal.clone();
        spawn(async move {
            if let Ok(items) = client.list_items().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for i in &items {
                    opts.push(SelectOption {
                        value: i.id.to_string(),
                        label: format!("{} ({})", i.item_name, i.item_code),
                    });
                    map.insert(i.id, i.clone());
                }
                item_map.set(map);
                item_opts.set(opts);
            }
            if let Ok(warehouses) = client.list_warehouses().await {
                let mut map = HashMap::new();
                let mut opts = Vec::new();
                for w in &warehouses {
                    opts.push(SelectOption {
                        value: w.id.to_string(),
                        label: w.warehouse_name.clone(),
                    });
                    map.insert(w.id, w.clone());
                }
                warehouse_map.set(map);
                wh_opts.set(opts);
            }
        });
    });

    // ── Form State ──
    let selected_item_id = use_signal(|| 0i64);
    let selected_warehouse_id = use_signal(|| 0i64);
    let movement_type = use_signal(|| "IN".to_string());
    let quantity = use_signal(|| String::new());
    let unit_cost = use_signal(|| String::new());
    let ref_doctype = use_signal(|| String::new());
    let ref_docno = use_signal(|| String::new());
    let notes = use_signal(String::new);

    // UI state
    let is_saving = use_signal(|| false);
    let mut is_dirty = use_signal(|| false);
    let mut show_discard_modal = use_signal(|| false);
    let errors = use_signal(HashMap::<&'static str, String>::new);

    // ── Computed ──
    let qty_val = quantity.read().parse::<f64>().unwrap_or(0.0);
    let cost_val = unit_cost.read().parse::<f64>().unwrap_or(0.0);
    let total_value = qty_val * cost_val;

    // ── Validation ──
    let validate = {
        let it = selected_item_id.clone();
        let wh = selected_warehouse_id.clone();
        let qty = quantity.clone();
        let cost = unit_cost.clone();
        let mut toast = toast.clone();
        move || -> bool {
            let mut errs = HashMap::<&str, String>::new();
            if *it.read() == 0 {
                errs.insert("item", "Item is required.".to_string());
            }
            if *wh.read() == 0 {
                errs.insert("warehouse", "Warehouse is required.".to_string());
            }
            let q = qty.read().parse::<f64>().unwrap_or(0.0);
            if q <= 0.0 {
                errs.insert("qty", "Quantity must be greater than 0.".to_string());
            }
            if let Ok(c) = cost.read().parse::<f64>() {
                if c < 0.0 { errs.insert("cost", "Unit cost cannot be negative.".to_string()); }
            } else if !cost.read().is_empty() {
                errs.insert("cost", "Invalid number.".to_string());
            }
            let is_valid = errs.is_empty();
            if !is_valid { toast.warning("Validation Error", "Please fix the highlighted fields."); }
            is_valid
        }
    };

    // ── Handlers ──

    let on_item_select = {
        let mut sel_id = selected_item_id.clone();
        let mut dirty = is_dirty.clone();
        move |value: String| {
            let id = value.parse::<i64>().unwrap_or(0);
            sel_id.set(id);
            dirty.set(true);
        }
    };

    let on_warehouse_change = {
        let mut wh_id = selected_warehouse_id.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| {
            let id = v.parse::<i64>().unwrap_or(0);
            wh_id.set(id);
            dirty.set(true);
        }
    };

    let on_type_change = {
        let mut mt = movement_type.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { mt.set(v); dirty.set(true); }
    };

    let on_qty_change = {
        let mut q = quantity.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { q.set(v); dirty.set(true); }
    };

    let on_cost_change = {
        let mut c = unit_cost.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { c.set(v); dirty.set(true); }
    };

    let on_ref_doc_change = {
        let mut rd = ref_doctype.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { rd.set(v); dirty.set(true); }
    };

    let on_ref_no_change = {
        let mut rn = ref_docno.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { rn.set(v); dirty.set(true); }
    };

    let on_notes_change = {
        let mut n = notes.clone();
        let mut dirty = is_dirty.clone();
        move |v: String| { n.set(v); dirty.set(true); }
    };

    // Save
    let save_movement = {
        let mut saving = is_saving.clone();
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let api = api.clone();
        let mut validate = validate.clone();
        let mut dirty = is_dirty.clone();
        let sel_item = selected_item_id.clone();
        let sel_wh = selected_warehouse_id.clone();
        let mv_type = movement_type.clone();
        let qty = quantity.clone();
        let cost = unit_cost.clone();
        let ref_dt = ref_doctype.clone();
        let ref_dn = ref_docno.clone();
        let nts = notes.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let mut toast = toast.clone();

            let dt = ref_dt.read().clone();
            let dn = ref_dn.read().clone();
            let form = StockMovementForm {
                item_id: *sel_item.read(),
                warehouse_id: *sel_wh.read(),
                movement_type: mv_type.read().clone(),
                quantity: qty.read().parse().unwrap_or(0.0),
                unit_cost: cost.read().parse::<f64>().ok(),
                reference_doctype: if dt.is_empty() { None } else { Some(dt) },
                reference_docno: if dn.is_empty() { None } else { Some(dn) },
                notes: Some(nts.read().clone()),
            };
            let client = api.read().clone();

            spawn(async move {
                match client.create_stock_movement(&form).await {
                    Ok(m) => {
                        toast.success("Movement Created", &format!("Stock movement {} has been recorded.", m.movement_no));
                        saving.set(false);
                        dirty.set(false);
                        nav.push("/inventory/stock-movements");
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Save & New
    let save_and_new = {
        let mut saving = is_saving.clone();
        let toast = toast.clone();
        let mut validate = validate.clone();
        let api = api.clone();
        let mut sel_item = selected_item_id.clone();
        let mut sel_wh = selected_warehouse_id.clone();
        let mut mv_type = movement_type.clone();
        let mut qty = quantity.clone();
        let mut cost = unit_cost.clone();
        let mut ref_dt = ref_doctype.clone();
        let mut ref_dn = ref_docno.clone();
        let mut nts = notes.clone();
        let mut dirty = is_dirty.clone();

        move |_| {
            if !validate() { return; }
            saving.set(true);
            let mut toast = toast.clone();

            let dt = ref_dt.read().clone();
            let dn = ref_dn.read().clone();
            let form = StockMovementForm {
                item_id: *sel_item.read(),
                warehouse_id: *sel_wh.read(),
                movement_type: mv_type.read().clone(),
                quantity: qty.read().parse().unwrap_or(0.0),
                unit_cost: cost.read().parse::<f64>().ok(),
                reference_doctype: if dt.is_empty() { None } else { Some(dt) },
                reference_docno: if dn.is_empty() { None } else { Some(dn) },
                notes: Some(nts.read().clone()),
            };
            let client = api.read().clone();

            spawn(async move {
                match client.create_stock_movement(&form).await {
                    Ok(m) => {
                        toast.success("Movement Created", &format!("{}. Creating another…", m.movement_no));
                        // Reset form
                        sel_item.set(0);
                        sel_wh.set(0);
                        mv_type.set("IN".to_string());
                        qty.set(String::new());
                        cost.set(String::new());
                        ref_dt.set(String::new());
                        ref_dn.set(String::new());
                        nts.set(String::new());
                        saving.set(false);
                        dirty.set(false);
                    }
                    Err(e) => {
                        toast.error("Error", &e);
                        saving.set(false);
                    }
                }
            });
        }
    };

    // Discard
    let open_discard = {
        let mut modal = show_discard_modal.clone();
        let dirty = is_dirty.clone();
        let nav = navigator.clone();
        move |_| {
            if *dirty.read() { modal.set(true); }
            else { nav.push("/inventory/stock-movements"); }
        }
    };

    let confirm_discard = {
        let nav = navigator.clone();
        let mut modal = show_discard_modal.clone();
        move |_| { modal.set(false); nav.push("/inventory/stock-movements"); }
    };

    let cancel_discard = {
        let mut modal = show_discard_modal.clone();
        move |_| modal.set(false)
    };

    // ── Derived ──
    let item_err = errors.read().get("item").cloned();
    let wh_err = errors.read().get("warehouse").cloned();
    let qty_err = errors.read().get("qty").cloned();
    let cost_err = errors.read().get("cost").cloned();

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page stock-move-create-page",

            // ── Header ──
            div { class: "stock-move-create-header",
                div {
                    a {
                        class: "stock-move-back-link",
                        href: "/inventory/stock-movements",
                        "← Back to Stock Movements"
                    }
                    h1 { "New Stock Movement" }
                }
                if *is_dirty.read() {
                    span {
                        style: "font-size: 12px; color: var(--warning); font-weight: 500;",
                        "⚠ Unsaved changes"
                    }
                }
            }

            // ── Section: Movement Details ──
            div { class: "stock-move-section",
                h2 { "Movement Details" }
                div { class: "stock-move-form-row",
                    div {
                        SearchableSelect {
                            options: movement_type_options(),
                            selected_value: Some(movement_type.read().clone()),
                            on_select: on_type_change,
                            placeholder: "Select type…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            // ── Section: Item & Warehouse ──
            div { class: "stock-move-section",
                h2 { "Item & Warehouse" }
                div { class: "stock-move-form-row",
                    div {
                        SearchableSelect {
                            options: item_options_signal.read().clone(),
                            selected_value: Some(selected_item_id.read().to_string()).filter(|s| s != "0"),
                            on_select: on_item_select,
                            placeholder: "Search item…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                        if let Some(e) = &item_err {
                            span { style: "color: var(--danger); font-size: 12px;", "{e}" }
                        }
                    }
                    div {
                        SearchableSelect {
                            options: warehouse_options_signal.read().clone(),
                            selected_value: Some(selected_warehouse_id.read().to_string()).filter(|s| s != "0"),
                            on_select: on_warehouse_change,
                            placeholder: "Select warehouse…",
                            searchable: true,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                }
            }

            // ── Section: Quantities & Cost ──
            div { class: "stock-move-section",
                h2 { "Quantity & Cost" }
                div { class: "stock-move-form-row",
                    FormInput {
                        label: Some("Quantity".to_string()),
                        value: quantity.read().clone(),
                        oninput: on_qty_change,
                        r#type: InputType::Number,
                        placeholder: Some("0".to_string()),
                        min: Some(0.0),
                        step: Some(1.0),
                        required: true,
                        error: qty_err,
                    }
                    FormInput {
                        label: Some("Unit Cost (PKR)".to_string()),
                        value: unit_cost.read().clone(),
                        oninput: on_cost_change,
                        r#type: InputType::Number,
                        placeholder: Some("0.00".to_string()),
                        min: Some(0.0),
                        step: Some(0.01),
                        error: cost_err,
                    }
                }

                // Total value preview
                if total_value > 0.0 {
                    div { style: "margin-top: 12px;",
                        StatCard {
                            title: format!("Total Value ({} x PKR {:.2})", qty_val, cost_val),
                            value: format!("PKR {:.2}", total_value),
                            variant: StatCardVariant::Primary,
                            icon: Some("💰".to_string()),
                        }
                    }
                }
            }

            // ── Section: Reference ──
            div { class: "stock-move-section",
                h2 { "Reference Document (Optional)" }
                div { class: "stock-move-form-row",
                    div {
                        SearchableSelect {
                            options: vec![
                                SelectOption { value: "PURCHASE".to_string(), label: "Purchase Order".to_string() },
                                SelectOption { value: "INVOICE".to_string(), label: "Sales Invoice".to_string() },
                                SelectOption { value: "GRN".to_string(), label: "Goods Receipt Note".to_string() },
                                SelectOption { value: "ADJ".to_string(), label: "Adjustment".to_string() },
                            ],
                            selected_value: Some(ref_doctype.read().clone()).filter(|s| !s.is_empty()),
                            on_select: on_ref_doc_change,
                            placeholder: "Doc type…",
                            searchable: false,
                            class: Some("cb-input-group".to_string()),
                        }
                    }
                    FormInput {
                        label: Some("Document No".to_string()),
                        value: ref_docno.read().clone(),
                        oninput: on_ref_no_change,
                        r#type: InputType::Text,
                        placeholder: Some("e.g. PO-2026-0001".to_string()),
                        hint: Some("Optional reference document number".to_string()),
                    }
                }
            }

            // ── Section: Notes ──
            div { class: "stock-move-section",
                h2 { "Notes" }
                FormInput {
                    value: notes.read().clone(),
                    oninput: on_notes_change,
                    r#type: InputType::TextArea,
                    placeholder: Some("Optional notes about this movement…".to_string()),
                    hint: Some("Internal notes for audit trail.".to_string()),
                }
            }

            // ── Action Bar ──
            div { class: "stock-move-action-bar",
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
                    onclick: save_movement,
                    loading: *is_saving.read(),
                    icon: Some("✓".to_string()),
                    "Save Movement"
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
                    "You have unsaved changes. Are you sure you want to discard this stock movement?"
                }
            }
        }
    }
}
