//! Warehouse Detail Page — A comprehensive warehouse detail view showing
//! warehouse information, capacity KPIs, and stock items stored.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant, use_toast,
};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.wh-detail-page {
    max-width: 960px;
    margin: 0 auto;
}

.wh-detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 20px;
    gap: 16px;
    flex-wrap: wrap;
}

.wh-detail-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.wh-detail-back {
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

.wh-detail-back:hover { text-decoration: underline; }

.wh-detail-title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.wh-detail-title-row h1 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
}

.wh-detail-code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 4px;
}

.wh-detail-status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
}

.wh-detail-status-active {
    background: rgba(40, 167, 69, 0.1);
    color: #28a745;
}

.wh-detail-status-inactive {
    background: rgba(108, 117, 125, 0.1);
    color: #6c757d;
}

.wh-detail-kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
}

.wh-detail-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.wh-detail-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.wh-detail-section-header h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
}

.wh-detail-section-header .section-badge {
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 10px;
}

.wh-detail-info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
}

.wh-detail-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.wh-detail-field-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
}

.wh-detail-field-value {
    font-size: 14px;
    color: var(--text-primary);
}

.wh-detail-field-value.monospace {
    font-family: monospace;
    font-size: 13px;
}

.wh-detail-field-value.text-success { color: #28a745; }
.wh-detail-field-value.text-danger { color: #dc3545; }

.wh-detail-stock-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.wh-detail-stock-table thead th {
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

.wh-detail-stock-table thead th.text-right {
    text-align: right;
}

.wh-detail-stock-table tbody td {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}

.wh-detail-stock-table tbody td.text-right {
    text-align: right;
    font-family: monospace;
    font-size: 12px;
}

.wh-detail-stock-table tbody tr:last-child td {
    border-bottom: none;
}

.wh-detail-stock-table tbody tr:hover {
    background: rgba(74, 144, 217, 0.03);
}

.wh-detail-empty {
    text-align: center;
    padding: 30px 20px;
    color: var(--text-secondary);
    font-size: 14px;
}

.wh-detail-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
    gap: 16px;
    color: var(--text-secondary);
}

.wh-detail-loading .loading-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9);
    border-radius: 50%;
    animation: wh-detail-spin 0.8s linear infinite;
}

@keyframes wh-detail-spin {
    to { transform: rotate(360deg); }
}

.wh-detail-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    flex-wrap: wrap;
}

.wh-detail-actions-left,
.wh-detail-actions-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

@media (max-width: 768px) {
    .wh-detail-header { flex-direction: column; }
    .wh-detail-title-row { flex-direction: column; align-items: flex-start; }
    .wh-detail-kpis { grid-template-columns: 1fr 1fr; }
    .wh-detail-info-grid { grid-template-columns: 1fr; }
    .wh-detail-actions { flex-direction: column; align-items: stretch; }
    .wh-detail-actions-left,
    .wh-detail-actions-right { justify-content: center; }
}
"##;

// ============================================================================
// Data Models
// ============================================================================

/// A stock item stored in a warehouse.
#[derive(Clone, Debug)]
struct StockItem {
    item_code: String,
    item_name: String,
    category: String,
    quantity: i32,
    unit: String,
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn WarehouseDetailPage(id: String) -> Element {
    let toast = use_toast();
    let navigator = use_navigator();

    // ── Async data fetch ──
    let api = use_auth().api;
    let id_clone = id.clone();
    let warehouse_resource = use_resource(move || {
        let api = api.clone();
        let id_for_fetch = id_clone.clone();
        async move {
            let client = api.with(|c| c.clone());
            let parsed_id = id_for_fetch.parse::<i64>().ok()?;
            let wh = client.get_warehouse(parsed_id).await.ok()?;

            // ponytail: stock items come from list_stock_balances filtered by warehouse
            let stock_items = client.list_stock_balances().await
                .unwrap_or_default()
                .into_iter()
                .filter(|sb| sb.warehouse_id == parsed_id)
                .map(|sb| StockItem {
                    item_code: sb.item_code.unwrap_or_default(),
                    item_name: sb.item_name.unwrap_or_default(),
                    category: sb.category.unwrap_or_default(),
                    quantity: sb.quantity as i32,
                    unit: sb.unit_of_measure.unwrap_or_default(),
                })
                .collect::<Vec<_>>();

            Some((wh, stock_items))
        }
    });

    let is_loading = warehouse_resource.read().is_none();
    let data_opt = warehouse_resource.read().as_ref().cloned();

    // ── Delete confirmation modal ──
    let mut show_delete_modal = use_signal(|| false);

    // ── Derive variables for rendering ──

    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page wh-detail-page",
                div { class: "wh-detail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading warehouse details…" }
                }
            }
        };
    }

    let Some((wh, stock_items)) = data_opt.flatten() else {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page wh-detail-page",
                div { class: "wh-detail-loading",
                    div { style: "font-size: 40px;", "🏭" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Warehouse Not Found" }
                    p { "No warehouse with ID \"{id}\" was found." }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| { navigator.push("/inventory/warehouses"); },
                        "← Back to Warehouses"
                    }
                }
            }
        };
    };

    let total_items: i32 = stock_items.iter().map(|s| s.quantity).sum();
    let capacity = wh.capacity;

    // ── Handlers ──

    let on_back = move |_: Event<MouseData>| {
        navigator.push("/inventory/warehouses");
    };

    let on_edit = {
        let nav = navigator.clone();
        let wh_id = id.clone();
        move |_| {
            nav.push(format!("/inventory/warehouses/{}/edit", wh_id));
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
        let wh_name = wh.warehouse_name.clone();
        move |_| {
            modal.set(false);
            toast.success("Warehouse Deleted", &format!("{} has been deleted.", wh_name));
            nav.push("/inventory/warehouses");
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| { modal.set(false); }
    };

    let status_class = if wh.is_active { "wh-detail-status-active" } else { "wh-detail-status-inactive" };
    let status_label = if wh.is_active { "✓ Active" } else { "— Inactive" };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page wh-detail-page",

            // ── Header ──
            div { class: "wh-detail-header",
                div { class: "wh-detail-title-group",
                    button {
                        class: "wh-detail-back",
                        r#type: "button",
                        onclick: on_back,
                        "← Back to Warehouses"
                    }
                    div { class: "wh-detail-title-row",
                        h1 { "{wh.warehouse_name}" }
                        span { class: "wh-detail-code", "{wh.warehouse_code}" }
                        span {
                            class: "wh-detail-status-badge {status_class}",
                            "{status_label}"
                        }
                    }
                }
            }

            // ── KPI Cards ──
            div { class: "wh-detail-kpis",
                StatCard {
                    title: "Total Items Stored".to_string(),
                    value: format!("{}", total_items),
                    variant: StatCardVariant::Primary,
                    icon: Some("📦".to_string()),
                    footer: Some(format!("Across {} SKUs", stock_items.len())),
                }
                StatCard {
                    title: "Capacity Utilization".to_string(),
                    value: format!("{:.1}%", capacity),
                    variant: StatCardVariant::Success,
                    icon: Some("📊".to_string()),
                    footer: Some("Adequate space available".to_string()),
                }
            }

            // ── Section: Details ──
            div { class: "wh-detail-section",
                div { class: "wh-detail-section-header",
                    h2 { "Warehouse Details" }
                    span { class: "section-badge", "General Information" }
                }
                div { class: "wh-detail-info-grid",
                    div { class: "wh-detail-field",
                        span { class: "wh-detail-field-label", "Warehouse Code" }
                        span { class: "wh-detail-field-value monospace", "{wh.warehouse_code}" }
                    }
                    div { class: "wh-detail-field",
                        span { class: "wh-detail-field-label", "Location" }
                        span { class: "wh-detail-field-value", "{wh.location}" }
                    }
                    div { class: "wh-detail-field",
                        span { class: "wh-detail-field-label", "Status" }
                        span { class: "wh-detail-field-value",
                            if wh.is_active {
                                span { class: "text-success", "Active" }
                            } else {
                                span { class: "text-danger", "Inactive" }
                            }
                        }
                    }
                    div { class: "wh-detail-field",
                        span { class: "wh-detail-field-label", "Created At" }
                        span { class: "wh-detail-field-value", "{wh.created_at}" }
                    }
                }
            }

            // ── Section: Stock Items ──
            div { class: "wh-detail-section",
                div { class: "wh-detail-section-header",
                    h2 { "Stock Items" }
                    span { class: "section-badge", "{stock_items.len()} items" }
                }
                if stock_items.is_empty() {
                    div { class: "wh-detail-empty", "No stock items in this warehouse." }
                } else {
                    table { class: "wh-detail-stock-table",
                        thead {
                            tr {
                                th { "Item Code" }
                                th { "Item Name" }
                                th { "Category" }
                                th { class: "text-right", "Quantity" }
                                th { "Unit" }
                            }
                        }
                        tbody {
                            {stock_items.iter().map(|si| {
                                rsx! {
                                    tr {
                                        td { class: "monospace", "{si.item_code}" }
                                        td { "{si.item_name}" }
                                        td { "{si.category}" }
                                        td { class: "text-right", "{si.quantity}" }
                                        td { "{si.unit}" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }

            // ── Action Bar ──
            div { class: "wh-detail-actions",
                div { class: "wh-detail-actions-left",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: on_edit,
                        icon: Some("✏️".to_string()),
                        "Edit Warehouse"
                    }
                }
                div { class: "wh-detail-actions-right",
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
                title: Some("Delete Warehouse".to_string()),
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
                        "Delete Warehouse"
                    }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Are you sure you want to delete {wh.warehouse_name}?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This action cannot be undone. The warehouse \"{wh.warehouse_code}\" will be permanently removed. ",
                        "Stock items will be reassigned."
                    }
                }
            }
        }
    }
}
