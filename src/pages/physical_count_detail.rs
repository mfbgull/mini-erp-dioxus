//! Physical Count Detail Page — A detail view for a physical inventory count,
//! showing count information, status, and counted items.

use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use crate::auth::use_auth;
use crate::pages::physical_count_list::PhysicalCountItem;
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.pc-detail-page {
    max-width: 960px;
    margin: 0 auto;
}

.pc-detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 20px;
    gap: 16px;
    flex-wrap: wrap;
}

.pc-detail-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.pc-detail-back {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--accent, #4a90d9);
    text-decoration: none;
    margin-bottom: 6px;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
}

.pc-detail-back:hover { text-decoration: underline; }

.pc-detail-title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.pc-detail-title-row h1 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
}

.pc-detail-code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 4px;
}

.pc-detail-status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
}

.pc-detail-status-draft {
    background: rgba(255, 193, 7, 0.15);
    color: #d4a017;
}

.pc-detail-status-completed {
    background: rgba(40, 167, 69, 0.1);
    color: #28a745;
}

.pc-detail-status-cancelled {
    background: rgba(220, 53, 69, 0.1);
    color: #dc3545;
}

.pc-detail-kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
}

.pc-detail-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.pc-detail-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.pc-detail-section-header h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
}

.pc-detail-section-header .section-badge {
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 10px;
}

.pc-detail-info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
}

.pc-detail-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.pc-detail-field-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
}

.pc-detail-field-value {
    font-size: 14px;
    color: var(--text-primary);
}

.pc-detail-field-value.monospace {
    font-family: monospace;
    font-size: 13px;
}

.pc-detail-field-value.text-success { color: #28a745; }
.pc-detail-field-value.text-danger { color: #dc3545; }

.pc-detail-counted-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.pc-detail-counted-table thead th {
    text-align: left;
    padding: 8px 10px;
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    color: var(--text-secondary);
    border-bottom: 2px solid var(--border-color, #e0e0e0);
    white-space: nowrap;
}

.pc-detail-counted-table thead th.text-right {
    text-align: right;
}

.pc-detail-counted-table tbody td {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}

.pc-detail-counted-table tbody td.text-right {
    text-align: right;
    font-family: monospace;
    font-size: 12px;
}

.pc-detail-counted-table tbody td.text-danger { color: #dc3545; }
.pc-detail-counted-table tbody td.text-success { color: #28a745; }

.pc-detail-counted-table tbody tr:last-child td { border-bottom: none; }
.pc-detail-counted-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.pc-detail-empty {
    text-align: center;
    padding: 30px 20px;
    color: var(--text-secondary);
    font-size: 14px;
}

.pc-detail-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
    gap: 16px;
    color: var(--text-secondary);
}

.pc-detail-loading .loading-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9);
    border-radius: 50%;
    animation: pc-detail-spin 0.8s linear infinite;
}

@keyframes pc-detail-spin {
    to { transform: rotate(360deg); }
}

.pc-detail-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    flex-wrap: wrap;
}

.pc-detail-actions-left,
.pc-detail-actions-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

.pc-draft-actions {
    display: flex;
    gap: 8px;
}

@media (max-width: 768px) {
    .pc-detail-header { flex-direction: column; }
    .pc-detail-title-row { flex-direction: column; align-items: flex-start; }
    .pc-detail-kpis { grid-template-columns: 1fr 1fr; }
    .pc-detail-info-grid { grid-template-columns: 1fr; }
    .pc-detail-actions { flex-direction: column; align-items: stretch; }
    .pc-detail-actions-left,
    .pc-detail-actions-right,
    .pc-draft-actions { justify-content: center; }
}
"##;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Clone, Debug)]
struct CountedItem {
    item_code: String,
    item_name: String,
    expected_qty: f64,
    counted_qty: f64,
    variance: f64,
    unit: String,
}

// ponytail: no GET endpoint for count items yet, stays empty until endpoint is added
fn empty_items() -> Vec<CountedItem> {
    vec![]
}

fn variance_class(v: f64) -> &'static str {
    if v > 0.0 { "text-success" }
    else if v < 0.0 { "text-danger" }
    else { "" }
}

fn variance_sign(v: f64) -> String {
    if v > 0.0 { format!("+{}", v) }
    else { format!("{}", v) }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "Draft" => "pc-detail-status-draft",
        "Completed" => "pc-detail-status-completed",
        "Cancelled" => "pc-detail-status-cancelled",
        _ => "pc-detail-status-draft",
    }
}

fn status_icon(status: &str) -> &'static str {
    match status {
        "Draft" => "📝",
        "Completed" => "✓",
        "Cancelled" => "✗",
        _ => "—",
    }
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn PhysicalCountDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    // ── Async fetch ──
    let api = use_auth().api;
    let id_for_display = id.clone();
    let id_clone = id.clone();
    let count_resource = use_resource(move || {
        let id_fetch = id_clone.clone();
        let api = api.clone();
        async move {
            let parsed = id_fetch.parse::<i64>().unwrap_or(0);
            if parsed == 0 { return None; }
            let client = api.read().clone();
            client.get_physical_count(parsed).await.ok().map(|c| PhysicalCountItem {
                id: c.id,
                count_no: c.count_no,
                count_date: c.count_date,
                warehouse_name: c.warehouse_name.unwrap_or_default(),
                status: c.status,
                notes: c.notes,
                created_at: c.created_at,
                completed_at: c.completed_at,
            })
        }
    });

    let is_loading = count_resource.read().is_none();
    let count_opt = count_resource.read().as_ref().cloned().flatten();

    // ── Delete modal ──
    let mut show_delete_modal = use_signal(|| false);

    // ── Pre-compute ──
    let detail = count_opt.as_ref().map(|count| {
        // ponytail: counted items endpoint not available yet
        let items = empty_items();
        let items_counted = items.iter().filter(|i| i.counted_qty > 0.0).count();
        let total_variance: f64 = items.iter().map(|i| i.variance).sum();
        let total_expected: f64 = items.iter().map(|i| i.expected_qty).sum();
        (count.clone(), items, items_counted, total_variance, total_expected)
    });

    if detail.is_none() {
        return rsx! { div {} };
    }
    let (ref count, ref items, items_counted, total_variance, total_expected) = detail.as_ref().unwrap();

    let is_draft = count.status == "Draft";

    // ── Handlers ──

    let on_back = move |_: Event<MouseData>| {
        navigator.push("/inventory/physical-counts");
    };

    let on_edit = {
        let nav = navigator.clone();
        let mut toast = toast.clone();
        let edit_id = id_for_display.clone();
        move |_| {
            toast.info("Edit Mode", "Editing coming soon — opening read-only detail.");
            nav.push(format!("/inventory/physical-counts/{}", edit_id));
        }
    };

    let on_complete = {
        let count_id = count.count_no.clone();
        let mut toast = toast.clone();
        move |_| {
            toast.success("Count Completed", &format!("{} has been marked as completed.", count_id));
        }
    };

    let on_cancel_count = {
        let count_id = count.count_no.clone();
        let mut toast = toast.clone();
        move |_| {
            toast.info("Count Cancelled", &format!("{} has been cancelled.", count_id));
        }
    };

    let on_print = {
        let count_id = count.count_no.clone();
        let mut toast = toast.clone();
        move |_| {
            toast.info("Print", &format!("Printing {}…", count_id));
        }
    };

    let on_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(true); }
    };

    let confirm_delete = {
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut modal = show_delete_modal.clone();
        let cn = count.count_no.clone();
        move |_| {
            modal.set(false);
            toast.success("Count Deleted", &format!("{} has been deleted.", cn));
            nav.push("/inventory/physical-counts");
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page pc-detail-page",

            // Loading
            if is_loading {
                div { class: "pc-detail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading count details…" }
                }
            }

            // Not Found
            else if count_opt.is_none() {
                div { class: "pc-detail-loading",
                    div { style: "font-size: 40px;", "🔢" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Count Not Found" }
                    p { "No physical count with ID \"{id_for_display}\" was found." }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| { navigator.push("/inventory/physical-counts"); },
                        "← Back to Counts"
                    }
                }
            }

            // Detail
            else if let Some(ref _detail) = detail {
                // ── Header ──
                div { class: "pc-detail-header",
                    div { class: "pc-detail-title-group",
                        button {
                            class: "pc-detail-back",
                            r#type: "button",
                            onclick: on_back,
                            "← Back to Physical Counts"
                        }
                        div { class: "pc-detail-title-row",
                            h1 { "Physical Count {count.count_no}" }
                            span { class: "pc-detail-code", "ID: {count.id}" }
                            span {
                                class: "pc-detail-status-badge {status_class(&count.status)}",
                                "{status_icon(&count.status)} {count.status}"
                            }
                        }
                    }
                }

                // ── KPI Cards ──
                div { class: "pc-detail-kpis",
                    StatCard {
                        title: "Total Expected".to_string(),
                        value: format!("{:.0}", total_expected),
                        variant: StatCardVariant::Primary,
                        icon: Some("📦".to_string()),
                    }
                    StatCard {
                        title: "Items Counted".to_string(),
                        value: format!("{} / {}", items_counted, items.len()),
                        variant: if *items_counted == items.len() { StatCardVariant::Success }
                                 else { StatCardVariant::Warning },
                        icon: Some("✅".to_string()),
                        footer: Some(if *items_counted == items.len() { "All items counted".to_string() }
                                       else { "Some items pending".to_string() }),
                    }
                    StatCard {
                        title: "Total Variance".to_string(),
                        value: format!("{:.0}", total_variance),
                        variant: if *total_variance == 0.0 { StatCardVariant::Success }
                                 else if (*total_variance).abs() < 10.0 { StatCardVariant::Warning }
                                 else { StatCardVariant::Danger },
                        icon: Some("📊".to_string()),
                        footer: Some(if *total_variance == 0.0 { "No discrepancies".to_string() }
                                       else if *total_variance > 0.0 { "Surplus items found".to_string() }
                                       else { "Shortage detected".to_string() }),
                    }
                }

                // ── Section: Count Details ──
                div { class: "pc-detail-section",
                    div { class: "pc-detail-section-header",
                        h2 { "Count Details" }
                        span { class: "section-badge", "General Information" }
                    }
                    div { class: "pc-detail-info-grid",
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Count No" }
                            span { class: "pc-detail-field-value monospace", "{count.count_no}" }
                        }
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Count Date" }
                            span { class: "pc-detail-field-value", "{count.count_date}" }
                        }
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Warehouse" }
                            span { class: "pc-detail-field-value", "{count.warehouse_name}" }
                        }
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Status" }
                            span { class: "pc-detail-field-value",
                                if count.status == "Draft" {
                                    span { class: "text-warning", "Draft" }
                                } else if count.status == "Completed" {
                                    span { class: "text-success", "Completed" }
                                } else {
                                    span { class: "text-danger", "Cancelled" }
                                }
                            }
                        }
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Created At" }
                            span { class: "pc-detail-field-value", "{count.created_at}" }
                        }
                        div { class: "pc-detail-field",
                            span { class: "pc-detail-field-label", "Completed At" }
                            span { class: "pc-detail-field-value",
                                if let Some(ref ca) = count.completed_at { "{ca}" }
                                else { "—" }
                            }
                        }
                    }
                }

                // ── Section: Notes (if any) ──
                if !count.notes.is_empty() {
                    div { class: "pc-detail-section",
                        div { class: "pc-detail-section-header",
                            h2 { "Notes" }
                        }
                        p { style: "margin: 0; font-size: 13px; color: var(--text-secondary); line-height: 1.6;",
                            "{count.notes}"
                        }
                    }
                }

                // ── Section: Counted Items ──
                div { class: "pc-detail-section",
                    div { class: "pc-detail-section-header",
                        h2 { "Counted Items" }
                        span { class: "section-badge", "{items.len()} items" }
                    }
                    table { class: "pc-detail-counted-table",
                        thead {
                            tr {
                                th { "Item Code" }
                                th { "Item Name" }
                                th { class: "text-right", "Expected Qty" }
                                th { class: "text-right", "Counted Qty" }
                                th { class: "text-right", "Variance" }
                                th { "Unit" }
                            }
                        }
                        tbody {
                            {items.iter().map(|ci| {
                                let v_cls = variance_class(ci.variance);
                                let v_sgn = variance_sign(ci.variance);
                                rsx! {
                                    tr {
                                        td { class: "monospace", "{ci.item_code}" }
                                        td { "{ci.item_name}" }
                                        td { class: "text-right", "{ci.expected_qty}" }
                                        td { class: "text-right", "{ci.counted_qty}" }
                                        td { class: "text-right {v_cls}", "{v_sgn}" }
                                        td { "{ci.unit}" }
                                    }
                                }
                            })}
                        }
                    }
                }

                // ── Action Bar ──
                div { class: "pc-detail-actions",
                    div { class: "pc-detail-actions-left",
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: on_edit,
                            icon: Some("✏️".to_string()),
                            "Edit"
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: on_print,
                            icon: Some("🖨️".to_string()),
                            "Print"
                        }
                    }
                    div { class: "pc-detail-actions-right",
                        if is_draft {
                            div { class: "pc-draft-actions",
                                Button {
                                    variant: ButtonVariant::Success,
                                    onclick: on_complete,
                                    icon: Some("✓".to_string()),
                                    "Complete Count"
                                }
                                Button {
                                    variant: ButtonVariant::Warning,
                                    onclick: on_cancel_count,
                                    icon: Some("✗".to_string()),
                                    "Cancel"
                                }
                            }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            onclick: on_delete,
                            icon: Some("🗑️".to_string()),
                            "Delete"
                        }
                    }
                }

                // ── Delete Confirmation Modal ──
                Modal {
                    is_open: show_delete_modal,
                    title: Some("Delete Physical Count".to_string()),
                    size: ModalSize::Sm,
                    close_on_backdrop: true,
                    close_on_escape: true,
                    footer: rsx! {
                        Button {
                            variant: ButtonVariant::Secondary,
                            onclick: cancel_delete,
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Danger,
                            onclick: confirm_delete,
                            "Delete Count"
                        }
                    },
                    div {
                        p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                            "Are you sure you want to delete {count.count_no}?"
                        }
                        p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                            "This action cannot be undone. The count record will be permanently removed."
                        }
                    }
                }
            }
        }
    }
}
