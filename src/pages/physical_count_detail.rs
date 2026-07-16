//! Physical Count Detail Page — Detail view with per-item counting workflow.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::auth::use_auth;
use dioxus::prelude::*;

const PAGE_CSS: &str = r##"
.pc-page { max-width: 1100px; margin: 0 auto; padding: 20px; }
.pc-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 20px; gap: 16px; flex-wrap: wrap; }
.pc-title-group { display: flex; flex-direction: column; gap: 4px; }
.pc-back { font-size: 13px; color: var(--accent, #4a90d9); cursor: pointer; background: none; border: none; padding: 0; }
.pc-back:hover { text-decoration: underline; }
.pc-title-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.pc-title-row h1 { font-size: 22px; font-weight: 700; color: var(--text-primary); margin: 0; }
.pc-code { font-family: monospace; font-size: 13px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 4px; }
.pc-status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 12px; font-size: 12px; font-weight: 600; }
.pc-status-draft { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.pc-status-completed { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.pc-status-cancelled { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
.pc-kpis { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; margin-bottom: 20px; }
.pc-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: 8px; padding: 20px; margin-bottom: 16px; }
.pc-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px solid var(--border-color); }
.pc-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.pc-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.pc-table thead th { text-align: left; padding: 10px 12px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); background: var(--bg-muted, #f8f9fa); border-bottom: 2px solid var(--border-color); }
.pc-table thead th.text-right { text-align: right; }
.pc-table tbody td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); color: var(--text-primary); }
.pc-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.pc-table tbody tr:last-child td { border-bottom: none; }
.pc-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.pc-counted-bold { font-weight: 600; }
.pc-counted-pending { color: var(--text-secondary); font-style: italic; }
.pc-variance-pos { color: #dc3545; font-weight: 600; }
.pc-variance-neg { color: #28a745; font-weight: 600; }
.pc-variance-zero { color: var(--text-secondary); }
.pc-actions { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-top: 20px; padding-top: 16px; border-top: 1px solid var(--border-color); flex-wrap: wrap; }
.pc-actions-left, .pc-actions-right { display: flex; align-items: center; gap: 8px; }
.pc-empty { text-align: center; padding: 30px 20px; color: var(--text-secondary); font-size: 14px; }
.pc-loading { display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 40vh; gap: 16px; color: var(--text-secondary); }
.pc-loading .spinner { width: 36px; height: 36px; border: 3px solid var(--border-color); border-top-color: var(--accent); border-radius: 50%; animation: pc-spin 0.8s linear infinite; }
@keyframes pc-spin { to { transform: rotate(360deg); } }
/* Edit modal */
.pc-edit-modal { display: flex; flex-direction: column; gap: 16px; }
.pc-edit-field { display: flex; flex-direction: column; gap: 4px; }
.pc-edit-field label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
.pc-edit-field input, .pc-edit-field textarea { padding: 8px 10px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; font-size: 13px; background: #fff; color: var(--text-primary); }
.pc-edit-field input[type="number"] { font-family: monospace; font-size: 16px; text-align: right; }
.pc-edit-preview { padding: 12px; background: var(--bg-muted, #f8f9fa); border-radius: 6px; font-size: 13px; }
.pc-edit-preview-row { display: flex; justify-content: space-between; margin-bottom: 4px; }
.pc-edit-preview-label { color: var(--text-secondary); }
.pc-edit-preview-value { font-weight: 600; font-family: monospace; }
.pc-info-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.pc-field { display: flex; flex-direction: column; gap: 3px; }
.pc-field-label { font-size: 11px; font-weight: 600; color: var(--text-secondary); text-transform: uppercase; }
.pc-field-value { font-size: 14px; color: var(--text-primary); }
"##;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Clone, Debug)]
struct CountedItem {
    id: i64,
    item_id: i64,
    item_code: String,
    item_name: String,
    system_quantity: f64,
    counted_quantity: Option<f64>,
    variance: Option<f64>,
    unit_cost: f64,
    variance_value: Option<f64>,
}

#[derive(Clone, Debug)]
struct CountDetail {
    id: i64,
    count_no: String,
    count_date: String,
    warehouse_id: i64,
    warehouse_name: String,
    status: String,
    notes: String,
    created_at: String,
    completed_at: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn status_class(status: &str) -> &'static str {
    match status {
        "Draft" => "pc-status-draft",
        "Completed" => "pc-status-completed",
        "Cancelled" => "pc-status-cancelled",
        _ => "pc-status-draft",
    }
}

fn format_qty(q: Option<f64>) -> String {
    match q {
        Some(v) => format!("{:.2}", v),
        None => "-".to_string(),
    }
}

fn variance_class(v: Option<f64>) -> &'static str {
    match v {
        Some(x) if x > 0.0 => "pc-variance-pos",
        Some(x) if x < 0.0 => "pc-variance-neg",
        _ => "pc-variance-zero",
    }
}

fn format_variance(v: Option<f64>) -> String {
    match v {
        Some(x) if x > 0.0 => format!("+{:.2}", x),
        Some(x) => format!("{:.2}", x),
        None => "-".to_string(),
    }
}

fn format_currency(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("PKR {:.2}", x),
        None => "-".to_string(),
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn PhysicalCountDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();
    let api = use_auth().api;
    let id_clone = id.clone();
    let id_for_complete = id.clone();
    let id_for_delete = id.clone();
    let id_for_cancel = id.clone();

    // State
    let mut show_edit_modal = use_signal(|| false);
    let mut editing_item = use_signal(|| None::<CountedItem>);
    let mut edit_value = use_signal(|| String::new());
    let mut edit_notes = use_signal(|| String::new());
    let mut show_delete_modal = use_signal(|| false);
    let mut counter = use_signal(|| 0u32);

    // Fetch count + items
    let resource = use_resource(move || {
        let api = api.clone();
        let id = id_clone.clone();
        let _ = *counter.read();
        async move {
            let parsed = id.parse::<i64>().unwrap_or(0);
            if parsed == 0 { return None; }
            let client = api.read().clone();
            match client.get_physical_count_with_items(parsed).await {
                Ok((count, items_json)) => {
                    let items: Vec<CountedItem> = items_json.into_iter().map(|item| {
                        let sq = item["system_quantity"].as_f64().unwrap_or(0.0);
                        let cq = item["counted_quantity"].as_f64();
                        let variance = cq.map(|v| v - sq);
                        let uc = item["unit_cost"].as_f64().unwrap_or(0.0);
                        let vv = variance.map(|v| v * uc);
                        CountedItem {
                            id: item["id"].as_i64().unwrap_or(0),
                            item_id: item["item_id"].as_i64().unwrap_or(0),
                            item_code: item["item_code"].as_str().unwrap_or("").to_string(),
                            item_name: item["item_name"].as_str().unwrap_or("").to_string(),
                            system_quantity: sq,
                            counted_quantity: cq,
                            variance,
                            unit_cost: uc,
                            variance_value: vv,
                        }
                    }).collect();
                    Some((CountDetail {
                        id: count.id,
                        count_no: count.count_no,
                        count_date: count.count_date,
                        warehouse_id: count.warehouse_id,
                        warehouse_name: count.warehouse_name.unwrap_or_default(),
                        status: count.status,
                        notes: count.notes,
                        created_at: count.created_at,
                        completed_at: count.completed_at,
                    }, items))
                }
                Err(_) => None,
            }
        }
    });

    let snap = resource.read();
    let is_loading = snap.is_none();
    let data = snap.as_ref().and_then(|d| d.clone());
    let is_draft = data.as_ref().map(|(c, _)| c.status == "Draft").unwrap_or(false);

    // Edit handlers
    let open_edit = {
        let mut editing_item = editing_item.clone();
        let mut edit_value = edit_value.clone();
        let mut edit_notes = edit_notes.clone();
        let mut show = show_edit_modal.clone();
        move |item: CountedItem| {
            edit_value.set(item.counted_quantity.map(|v| format!("{:.2}", v)).unwrap_or_default());
            edit_notes.set(String::new());
            editing_item.set(Some(item));
            show.set(true);
        }
    };

    let save_count = {
        let api = use_auth().api;
        let mut toast = toast.clone();
        let mut show = show_edit_modal.clone();
        let editing = editing_item.clone();
        let val = edit_value.clone();
        let notes = edit_notes.clone();
        let count_id: i64 = id.parse().unwrap_or(0);
        let mut counter = counter.clone();
        move |_| {
            let item_opt = editing.read().clone();
            let item = match item_opt {
                Some(i) => i,
                None => return,
            };
            let counted_qty: f64 = match val.read().parse() {
                Ok(v) => v,
                Err(_) => {
                    toast.error("Error", "Invalid quantity.");
                    return;
                }
            };
            let api = api.clone();
            let mut toast = toast.clone();
            let item_id = item.id;
            let n = notes.read().clone();
            spawn(async move {
                let client = api.read().clone();
                match client.update_count_item(count_id, item_id, counted_qty).await {
                    Ok(_) => {
                        toast.success("Saved", "Count recorded.");
                        show.set(false);
                        let current = *counter.read();
                        counter.set(current + 1);
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    // Complete handler
    let complete_count = {
        let api = use_auth().api;
        let mut toast = toast.clone();
        let mut navigator = navigator.clone();
        let mut counter = counter.clone();
        let count_id: i64 = id_for_complete.parse().unwrap_or(0);
        move |_| {
            let api = api.clone();
            let mut toast = toast.clone();
            let mut nav = navigator.clone();
            let mut counter = counter.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.complete_physical_count(count_id).await {
                    Ok(_) => {
                        toast.success("Completed", "Physical count completed and stock adjustments posted.");
                        let current = *counter.read();
                        counter.set(current + 1);
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    // Cancel handler
    let cancel_count = {
        let api = use_auth().api;
        let mut toast = toast.clone();
        let mut counter = counter.clone();
        let count_id: i64 = id_for_cancel.parse().unwrap_or(0);
        move |_| {
            let api = api.clone();
            let mut toast = toast.clone();
            let mut counter = counter.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.cancel_physical_count(count_id).await {
                    Ok(_) => {
                        toast.success("Cancelled", "Physical count cancelled.");
                        let current = *counter.read();
                        counter.set(current + 1);
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    // Delete handler
    let confirm_delete = {
        let api = use_auth().api;
        let mut toast = toast.clone();
        let mut nav = navigator.clone();
        let mut show = show_delete_modal.clone();
        let count_id: i64 = id_for_delete.parse().unwrap_or(0);
        move |_| {
            let api = api.clone();
            let mut toast = toast.clone();
            let mut nav = nav.clone();
            let mut show = show.clone();
            spawn(async move {
                let client = api.read().clone();
                match client.delete_physical_count(count_id).await {
                    Ok(_) => {
                        toast.success("Deleted", "Physical count deleted.");
                        show.set(false);
                        nav.push("/inventory/physical-counts");
                    }
                    Err(e) => toast.error("Error", &e),
                }
            });
        }
    };

    // Render
    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "pc-page",
            if is_loading {
                div { class: "pc-loading",
                    div { class: "spinner" }
                    span { "Loading count details..." }
                }
            } else if let Some((count, items)) = data {{
                let total_items = items.len();
                let counted_items = items.iter().filter(|i| i.counted_quantity.is_some()).count();
                let variance_items = items.iter().filter(|i| i.variance.map(|v| v != 0.0).unwrap_or(false)).count();
                let total_variance_value: f64 = items.iter().filter_map(|i| i.variance_value).sum();
                let total_system: f64 = items.iter().map(|i| i.system_quantity).sum();

                rsx! {
                    // Header
                    div { class: "pc-header",
                        div { class: "pc-title-group",
                            button { class: "pc-back", r#type: "button", onclick: move |_| { navigator.push("/inventory/physical-counts"); }, "← Back to Physical Counts" }
                            div { class: "pc-title-row",
                                h1 { "Physical Count {count.count_no}" }
                                span { class: "pc-status-badge {status_class(&count.status)}", "{count.status}" }
                            }
                        }
                    }

                    // KPIs
                    div { class: "pc-kpis",
                        StatCard {
                            title: "Total Items".to_string(),
                            value: format!("{}", total_items),
                            variant: StatCardVariant::Primary,
                            icon: Some("📦".to_string()),
                        }
                        StatCard {
                            title: "Counted".to_string(),
                            value: format!("{} / {}", counted_items, total_items),
                            variant: if counted_items == total_items { StatCardVariant::Success } else { StatCardVariant::Warning },
                            icon: Some("✅".to_string()),
                            footer: Some(if counted_items == total_items { "All items counted".to_string() } else { "Some items pending".to_string() }),
                        }
                        StatCard {
                            title: "Variances".to_string(),
                            value: format!("{}", variance_items),
                            variant: if variance_items == 0 { StatCardVariant::Success } else { StatCardVariant::Danger },
                            icon: Some("📊".to_string()),
                            footer: Some(format!("Value: {}", format_currency(Some(total_variance_value)))),
                        }
                    }

                    // Count Details
                    div { class: "pc-section",
                        div { class: "pc-section-header", h2 { "Count Information" } }
                        div { class: "pc-info-grid",
                            div { class: "pc-field", span { class: "pc-field-label", "Count No" } span { class: "pc-field-value", style: "font-family: monospace;", "{count.count_no}" } }
                            div { class: "pc-field", span { class: "pc-field-label", "Date" } span { class: "pc-field-value", "{count.count_date}" } }
                            div { class: "pc-field", span { class: "pc-field-label", "Warehouse" } span { class: "pc-field-value", "{count.warehouse_name}" } }
                            div { class: "pc-field", span { class: "pc-field-label", "Status" } span { class: "pc-field-value", "{count.status}" } }
                            div { class: "pc-field", span { class: "pc-field-label", "Created" } span { class: "pc-field-value", "{count.created_at}" } }
                            if let Some(ref ca) = count.completed_at {
                                div { class: "pc-field", span { class: "pc-field-label", "Completed" } span { class: "pc-field-value", "{ca}" } }
                            }
                        }
                    }

                    // Counted Items
                    div { class: "pc-section",
                        div { class: "pc-section-header",
                            h2 { "Counted Items" }
                            span { style: "font-size: 11px; color: var(--text-secondary); background: var(--bg-muted, #f5f5f5); padding: 2px 8px; border-radius: 10px;", "{total_items} items" }
                        }
                        if items.is_empty() {
                            div { class: "pc-empty", "No items in this count." }
                        } else {
                            table { class: "pc-table",
                                thead { tr {
                                    th { "Item Code" }
                                    th { "Item Name" }
                                    th { class: "text-right", "System Qty" }
                                    th { class: "text-right", "Counted Qty" }
                                    th { class: "text-right", "Variance" }
                                    th { class: "text-right", "Value" }
                                    if is_draft { th { style: "text-align: center;", "Action" } }
                                }}
                                tbody {
                                    for item in items.iter() {
                                        {let item_clone = item.clone();
                                        let is_counted = item.counted_quantity.is_some();
                                        rsx! {
                                            tr {
                                                td { style: "font-family: monospace;", "{item.item_code}" }
                                                td { "{item.item_name}" }
                                                td { class: "text-right", "{item.system_quantity:.2}" }
                                                td { class: if is_counted { "text-right pc-counted-bold" } else { "text-right pc-counted-pending" },
                                                    {format_qty(item.counted_quantity)}
                                                }
                                                td { class: "text-right {variance_class(item.variance)}", "{format_variance(item.variance)}" }
                                                td { class: "text-right", "{format_currency(item.variance_value)}" }
                                                if is_draft {
                                                    td { style: "text-align: center;",
                                                        Button {
                                                            variant: ButtonVariant::Secondary,
                                                            onclick: { let mut open = open_edit.clone(); move |_| { open(item_clone.clone()); } },
                                                            if is_counted { "Recount" } else { "Count" }
                                                        }
                                                    }
                                                }
                                            }
                                        }}
                                    }
                                }
                            }
                        }
                    }

                    // Actions
                    div { class: "pc-actions",
                        div { class: "pc-actions-left",
                            if is_draft {
                                Button { variant: ButtonVariant::Danger, onclick: { let mut show = show_delete_modal.clone(); move |_| show.set(true) }, "Delete" }
                            }
                        }
                        div { class: "pc-actions-right",
                            if is_draft {
                                Button { variant: ButtonVariant::Warning, onclick: cancel_count, "Cancel Count" }
                                Button {
                                    variant: ButtonVariant::Success,
                                    onclick: complete_count,
                                    disabled: counted_items == 0,
                                    "Complete & Post Adjustments"
                                }
                            }
                        }
                    }

                    // Edit Modal
                    Modal {
                        is_open: show_edit_modal,
                        title: Some("Record Count".to_string()),
                        size: ModalSize::Sm,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: move |_| show_edit_modal.set(false), "Cancel" }
                            Button { variant: ButtonVariant::Primary, onclick: save_count, "Save Count" }
                        },
                        if let Some(ref item) = *editing_item.read() {{
                            let sys_qty = item.system_quantity;
                            let input_val: f64 = edit_value.read().parse().unwrap_or(0.0);
                            let variance = input_val - sys_qty;
                            rsx! {
                                div { class: "pc-edit-modal",
                                    div { style: "font-size: 14px; font-weight: 600; color: var(--text-primary); margin-bottom: 8px;",
                                        "{item.item_code} — {item.item_name}"
                                    }
                                    div { class: "pc-edit-field",
                                        label { "System Quantity" }
                                        div { style: "font-size: 20px; font-weight: 700; font-family: monospace; color: var(--text-primary);",
                                            "{sys_qty:.2}"
                                        }
                                    }
                                    div { class: "pc-edit-field",
                                        label { "Counted Quantity" }
                                        input {
                                            r#type: "number",
                                            step: "0.01",
                                            min: "0",
                                            placeholder: "Enter counted quantity",
                                            value: "{edit_value}",
                                            autofocus: "true",
                                            onchange: move |e| edit_value.set(e.value()),
                                        }
                                    }
                                    div { class: "pc-edit-preview",
                                        div { class: "pc-edit-preview-row",
                                            span { class: "pc-edit-preview-label", "Variance:" }
                                            span { class: "pc-edit-preview-value {variance_class(Some(variance))}", "{format_variance(Some(variance))}" }
                                        }
                                        div { class: "pc-edit-preview-row",
                                            span { class: "pc-edit-preview-label", "Variance Value:" }
                                            span { class: "pc-edit-preview-value", "{format_currency(Some(variance * item.unit_cost))}" }
                                        }
                                    }
                                    div { class: "pc-edit-field",
                                        label { "Notes (optional)" }
                                        input {
                                            r#type: "text",
                                            placeholder: "e.g., Damaged, expired",
                                            value: "{edit_notes}",
                                            onchange: move |e| edit_notes.set(e.value()),
                                        }
                                    }
                                }
                            }
                        }}
                    }

                    // Delete Modal
                    Modal {
                        is_open: show_delete_modal,
                        title: Some("Delete Physical Count".to_string()),
                        size: ModalSize::Sm,
                        close_on_backdrop: true,
                        close_on_escape: true,
                        footer: rsx! {
                            Button { variant: ButtonVariant::Secondary, onclick: move |_| show_delete_modal.set(false), "Cancel" }
                            Button { variant: ButtonVariant::Danger, onclick: confirm_delete, "Delete Count" }
                        },
                        div {
                            p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                                "Are you sure you want to delete {count.count_no}?"
                            }
                            p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                                "This action cannot be undone."
                            }
                        }
                    }
                }
            }} else {
                div { class: "pc-loading",
                    h2 { style: "margin: 0; color: var(--text-primary);", "Count Not Found" }
                    p { "No physical count with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: move |_| { navigator.push("/inventory/physical-counts"); }, "← Back to Counts" }
                }
            }
        }
    }
}
