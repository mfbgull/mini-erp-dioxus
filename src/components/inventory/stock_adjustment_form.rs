//! Stock Adjustment / Transfer modal form.
//!
//! Ported from the source project's `StockAdjustmentForm.tsx`:
//! - Two user-driven movement types: ADJUSTMENT (single warehouse, signed qty)
//!   and TRANSFER (from → to, positive qty).
//! - Multi-line items: add/remove rows, one submit records N movements.
//! - Live available-stock per line (from stock_balances for the source warehouse).
//! - Live financial-impact preview for ADJUSTMENT (|qty| × standard_cost).
//! - On submit fans out one create_stock_movement call per movement row.
//!   TRANSFER emits two rows per item (−from, +to); the server posts the
//!   ADJUSTMENT journal entry.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, FormInput, InputType, SearchableSelect, SelectOption, use_toast,
};
use crate::models::{Item, StockBalance, StockMovementForm, Warehouse};
use dioxus::prelude::*;

const FORM_CSS: &str = r##"
.saf-row { display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 12px; }
.saf-row > * { flex: 1; min-width: 160px; }
.saf-items-header { display: flex; justify-content: space-between; align-items: center; margin: 16px 0 10px; }
.saf-items-header h4 { margin: 0; font-size: 14px; font-weight: 600; }
.saf-line { display: flex; align-items: flex-start; gap: 10px; padding: 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; margin-bottom: 8px; }
.saf-line-idx { width: 22px; height: 22px; border-radius: 50%; background: var(--bg-muted, #f5f5f5); color: var(--text-secondary); font-size: 12px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; margin-top: 6px; }
.saf-line-fields { flex: 1; display: flex; gap: 10px; flex-wrap: wrap; }
.saf-line-fields > .saf-item { flex: 2; min-width: 200px; }
.saf-line-fields > .saf-qty { flex: 1; min-width: 120px; }
.saf-avail { font-size: 11px; color: var(--text-secondary); margin: 2px 0 4px; }
.saf-avail-pos { color: #28a745; font-weight: 600; }
.saf-avail-zero { color: #dc3545; font-weight: 600; }
.saf-remove { background: none; border: none; color: var(--danger, #dc3545); font-size: 20px; cursor: pointer; line-height: 1; margin-top: 4px; }
.saf-remove:disabled { color: var(--border-color, #ccc); cursor: not-allowed; }
.saf-preview { margin-top: 14px; padding: 12px; background: var(--bg-muted, #f8f9fa); border: 1px solid var(--border-color, #e2e8f0); border-radius: 8px; }
.saf-preview h4 { margin: 0 0 8px; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-secondary); }
.saf-preview-line { display: flex; justify-content: space-between; align-items: center; padding: 4px 0; border-bottom: 1px solid var(--border-color, #e2e8f0); font-size: 13px; }
.saf-preview-line:last-child { border-bottom: none; }
.saf-warn { color: #f59e0b; font-size: 12px; }
"##;

#[derive(Clone, PartialEq, Default)]
struct LineItem {
    item_id: i64,
    quantity: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct StockAdjustmentFormProps {
    /// Called after a successful save so the parent can refresh + close.
    pub on_success: EventHandler<()>,
    /// Called when the user cancels.
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn StockAdjustmentForm(props: StockAdjustmentFormProps) -> Element {
    let toast = use_toast();
    let api = use_auth().api;

    // ── Loaded reference data ──
    let items = use_signal(Vec::<Item>::new);
    let warehouses = use_signal(Vec::<Warehouse>::new);
    let balances = use_signal(Vec::<StockBalance>::new);

    let load_api = api.clone();
    use_effect(move || {
        let client = load_api.read().clone();
        let mut items = items.clone();
        let mut warehouses = warehouses.clone();
        let mut balances = balances.clone();
        spawn(async move {
            if let Ok(v) = client.list_items().await { items.set(v); }
            if let Ok(v) = client.list_warehouses().await { warehouses.set(v); }
            if let Ok(v) = client.list_stock_balances().await { balances.set(v); }
        });
    });

    // ── Form state ──
    let movement_type = use_signal(|| "ADJUSTMENT".to_string());
    let from_warehouse = use_signal(|| 0i64);
    let to_warehouse = use_signal(|| 0i64);
    let remarks = use_signal(String::new);
    let lines = use_signal(|| vec![LineItem::default()]);
    let is_saving = use_signal(|| false);

    let is_transfer = movement_type.read().clone() == "TRANSFER";
    // Source warehouse for available-stock lookups.
    let source_wh = if is_transfer { *from_warehouse.read() } else { *to_warehouse.read() };

    // available stock for (item, warehouse) from loaded balances
    let get_stock = {
        let balances = balances.clone();
        move |item_id: i64, wh_id: i64| -> f64 {
            if item_id == 0 || wh_id == 0 { return 0.0; }
            balances.read().iter()
                .find(|b| b.item_id == item_id && b.warehouse_id == wh_id)
                .map(|b| b.quantity)
                .unwrap_or(0.0)
        }
    };

    // In TRANSFER mode, only show items that have stock in the source warehouse.
    let item_options: Vec<SelectOption> = {
        let items = items.read();
        let balances = balances.read();
        items.iter().filter(|i| {
            if !is_transfer || *from_warehouse.read() == 0 { return true; }
            balances.iter().any(|b| b.item_id == i.id && b.warehouse_id == *from_warehouse.read() && b.quantity > 0.0)
        }).map(|i| SelectOption { value: i.id.to_string(), label: format!("{} - {}", i.item_code, i.item_name) })
        .collect()
    };
    let warehouse_options: Vec<SelectOption> = warehouses.read().iter()
        .map(|w| SelectOption { value: w.id.to_string(), label: format!("{} - {}", w.warehouse_code, w.warehouse_name) })
        .collect();
    // TRANSFER destination excludes the chosen source warehouse.
    let to_warehouse_options: Vec<SelectOption> = warehouses.read().iter()
        .filter(|w| w.id != *from_warehouse.read())
        .map(|w| SelectOption { value: w.id.to_string(), label: format!("{} - {}", w.warehouse_code, w.warehouse_name) })
        .collect();

    // ── Handlers ──
    let on_type = {
        let mut mt = movement_type.clone();
        let mut to_wh = to_warehouse.clone();
        move |v: String| {
            mt.set(v.clone());
            if v == "TRANSFER" { to_wh.set(0); } // clear stale ADJUSTMENT warehouse
        }
    };
    let on_from = { let mut w = from_warehouse.clone(); move |v: String| w.set(v.parse().unwrap_or(0)) };
    let on_to = { let mut w = to_warehouse.clone(); move |v: String| w.set(v.parse().unwrap_or(0)) };
    let on_remarks = { let mut r = remarks.clone(); move |v: String| r.set(v) };

    let add_line = { let mut lines = lines.clone(); move |_| { let mut v = lines.read().clone(); v.push(LineItem::default()); lines.set(v); } };

    // ── Submit: build movements and fan out ──
    // The submit closure captures signals by clone; reading them at call time
    // gets the latest value.  from_wh == source, to_wh == destination (TRANSFER)
    // or the single warehouse (ADJUSTMENT).
    let submit = {
        let api = api.clone();
        let toast_h = toast.clone();
        let on_success = props.on_success;
        let lines = lines.clone();
        let mt = movement_type.clone();
        let from_wh = from_warehouse.clone();
        let to_wh = to_warehouse.clone();
        let remarks = remarks.clone();
        let saving = is_saving.clone();
        move |_| {
            let mut toast = toast_h.clone();
            let mtype = mt.read().clone();
            let is_transfer = mtype == "TRANSFER";
            let from_id = *from_wh.read();
            let to_id = *to_wh.read();

            // Warehouse validation
            if is_transfer {
                if from_id == 0 || to_id == 0 {
                    toast.warning("Validation Error", "Select both source and destination warehouses.");
                    return;
                }
                if from_id == to_id {
                    toast.warning("Validation Error", "Source and destination must differ.");
                    return;
                }
            } else if to_id == 0 {
                toast.warning("Validation Error", "Select a warehouse.");
                return;
            }

            // Valid lines: adjustment allows ±, transfer requires > 0.
            let valid: Vec<(i64, f64)> = lines.read().iter()
                .filter_map(|l| {
                    if l.item_id == 0 { return None; }
                    let q = l.quantity.parse::<f64>().unwrap_or(0.0);
                    if is_transfer {
                        if q > 0.0 { Some((l.item_id, q)) } else { None }
                    } else if q != 0.0 {
                        Some((l.item_id, q))
                    } else {
                        None
                    }
                })
                .collect();

            if valid.is_empty() {
                toast.warning("Validation Error", "Add at least one item with a quantity.");
                return;
            }

            // Build movement forms.
            let note = remarks.read().clone();
            let mut forms: Vec<StockMovementForm> = Vec::new();
            for (item_id, qty) in &valid {
                if is_transfer {
                    let n = if note.is_empty() { "Stock transfer".to_string() } else { note.clone() };
                    forms.push(StockMovementForm {
                        item_id: *item_id, warehouse_id: from_id,
                        movement_type: "TRANSFER".to_string(), quantity: -qty.abs(),
                        unit_cost: None, reference_doctype: None, reference_docno: None,
                        notes: Some(n.clone()),
                    });
                    forms.push(StockMovementForm {
                        item_id: *item_id, warehouse_id: to_id,
                        movement_type: "TRANSFER".to_string(), quantity: qty.abs(),
                        unit_cost: None, reference_doctype: None, reference_docno: None,
                        notes: Some(n),
                    });
                } else {
                    let n = if note.is_empty() { "Stock adjustment".to_string() } else { note.clone() };
                    forms.push(StockMovementForm {
                        item_id: *item_id, warehouse_id: to_id,
                        movement_type: "ADJUSTMENT".to_string(), quantity: *qty,
                        unit_cost: None, reference_doctype: None, reference_docno: None,
                        notes: Some(n),
                    });
                }
            }

            let client = api.read().clone();
            let mut saving = saving.clone();
            let count = valid.len();
            saving.set(true);
            spawn(async move {
                for f in &forms {
                    if let Err(e) = client.create_stock_movement(f).await {
                        toast.error("Error", &e);
                        saving.set(false);
                        return;
                    }
                }
                toast.success("Recorded", &format!("{} stock movement(s) recorded.", count));
                saving.set(false);
                on_success.call(());
            });
        }
    };

    // ── Financial preview (adjustment only) ──
    let show_preview = !is_transfer
        && lines.read().iter().any(|l| l.item_id != 0 && l.quantity.parse::<f64>().unwrap_or(0.0) != 0.0);

    rsx! {
        style { "{FORM_CSS}" }

        // Type + warehouse selectors
        div { class: "saf-row",
            div {
                label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "Movement Type" }
                SearchableSelect {
                    options: vec![
                        SelectOption { value: "ADJUSTMENT".to_string(), label: "Stock Adjustment".to_string() },
                        SelectOption { value: "TRANSFER".to_string(), label: "Stock Transfer".to_string() },
                    ],
                    selected_value: Some(movement_type.read().clone()),
                    on_select: on_type,
                    searchable: false,
                }
            }
            if is_transfer {
                div {
                    label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "From Warehouse" }
                    SearchableSelect {
                        options: warehouse_options.clone(),
                        selected_value: Some(from_warehouse.read().to_string()).filter(|s| s != "0"),
                        on_select: on_from,
                        placeholder: "Source…",
                    }
                }
                div {
                    label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "To Warehouse" }
                    SearchableSelect {
                        options: to_warehouse_options.clone(),
                        selected_value: Some(to_warehouse.read().to_string()).filter(|s| s != "0"),
                        on_select: on_to,
                        placeholder: "Destination…",
                    }
                }
            } else {
                div {
                    label { style: "font-size:12px; font-weight:600; color:var(--text-secondary);", "Warehouse" }
                    SearchableSelect {
                        options: warehouse_options.clone(),
                        selected_value: Some(to_warehouse.read().to_string()).filter(|s| s != "0"),
                        on_select: on_to,
                        placeholder: "Select warehouse…",
                    }
                }
            }
        }

        // Items
        div { class: "saf-items-header",
            h4 { "Items" }
            Button { variant: ButtonVariant::Secondary, onclick: add_line, "+ Add Row" }
        }

        {lines.read().iter().enumerate().map(|(idx, line)| {
            let line_item_id = line.item_id;
            let line_qty = line.quantity.clone();
            let avail = get_stock(line_item_id, source_wh);
            let uom = items.read().iter().find(|i| i.id == line_item_id).map(|i| i.unit_of_measure.clone()).unwrap_or_default();
            let can_remove = lines.read().len() > 1;

            let on_item = {
                let mut lines = lines.clone();
                move |v: String| {
                    let mut vec = lines.read().clone();
                    vec[idx].item_id = v.parse().unwrap_or(0);
                    lines.set(vec);
                }
            };
            let on_qty = {
                let mut lines = lines.clone();
                move |v: String| {
                    let mut vec = lines.read().clone();
                    vec[idx].quantity = v;
                    lines.set(vec);
                }
            };
            let on_remove = {
                let mut lines = lines.clone();
                move |_| {
                    let mut vec = lines.read().clone();
                    if vec.len() > 1 { vec.remove(idx); lines.set(vec); }
                }
            };

            let avail_class = if avail > 0.0 { "saf-avail-pos" } else if line_item_id != 0 { "saf-avail-zero" } else { "" };
            let avail_text = if line_item_id != 0 && source_wh != 0 {
                format!("{} {}", avail, uom)
            } else { "—".to_string() };

            rsx! {
                div { class: "saf-line", key: "{idx}",
                    span { class: "saf-line-idx", "{idx + 1}" }
                    div { class: "saf-line-fields",
                        div { class: "saf-item",
                            SearchableSelect {
                                options: item_options.clone(),
                                selected_value: Some(line_item_id.to_string()).filter(|s| s != "0"),
                                on_select: on_item,
                                placeholder: "Search items…",
                            }
                        }
                        div { class: "saf-qty",
                            div { class: "saf-avail",
                                "Available: "
                                span { class: "{avail_class}", "{avail_text}" }
                            }
                            FormInput {
                                value: line_qty,
                                oninput: on_qty,
                                r#type: InputType::Number,
                                step: Some(0.01),
                                placeholder: Some("0.00".to_string()),
                            }
                        }
                    }
                    button {
                        class: "saf-remove",
                        r#type: "button",
                        disabled: !can_remove,
                        title: "Remove row",
                        onclick: on_remove,
                        "×"
                    }
                }
            }
        })}

        // Financial impact preview (adjustment only)
        if show_preview {
            div { class: "saf-preview",
                h4 { "Financial Impact Preview" }
                {lines.read().iter().filter_map(|line| {
                    let qty = line.quantity.parse::<f64>().unwrap_or(0.0);
                    if line.item_id == 0 || qty == 0.0 { return None; }
                    let item = items.read().iter().find(|i| i.id == line.item_id).cloned()?;
                    let cost = item.standard_cost;
                    let value = qty.abs() * cost;
                    let is_removal = qty < 0.0;
                    let (label, color) = if is_removal {
                        ("Inventory Shrinkage (Expense)", "#ef4444")
                    } else {
                        ("Inventory Correction (Income)", "#22c55e")
                    };
                    let sign = if is_removal { "-" } else { "+" };
                    Some(rsx! {
                        div { class: "saf-preview-line",
                            div {
                                strong { "{item.item_name}" }
                                span { style: "color:var(--text-secondary); margin-left:8px;",
                                    "{sign}{qty.abs()} {item.unit_of_measure} × {cost:.2}"
                                }
                            }
                            div { style: "text-align:right;",
                                if cost == 0.0 {
                                    span { class: "saf-warn", "⚠ No standard_cost set" }
                                } else {
                                    span { style: "font-weight:600; color:{color};", "{value:.2}" }
                                    br {}
                                    span { style: "font-size:11px; color:{color};", "{label}" }
                                }
                            }
                        }
                    })
                })}
            }
        }

        // Remarks
        div { style: "margin-top:12px;",
            FormInput {
                label: Some("Remarks".to_string()),
                value: remarks.read().clone(),
                oninput: on_remarks,
                r#type: InputType::TextArea,
                placeholder: Some("Reason for movement…".to_string()),
            }
        }

        // Actions
        div { style: "display:flex; justify-content:flex-end; gap:8px; margin-top:16px;",
            Button {
                variant: ButtonVariant::Secondary,
                onclick: move |_| props.on_cancel.call(()),
                disabled: *is_saving.read(),
                "Cancel"
            }
            Button {
                variant: ButtonVariant::Primary,
                onclick: submit,
                loading: *is_saving.read(),
                {if is_transfer { "Record Transfer" } else { "Record Adjustment" }}
            }
        }
    }
}
