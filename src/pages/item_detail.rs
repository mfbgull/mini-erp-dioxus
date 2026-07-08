//! Item Detail Page — A comprehensive item detail view showing full item
//! information, stock ledger entries, pricing KPIs, and action buttons.
//!
//! This page reuses the `InventoryItem` model from `item_list.rs` and
//! fetches item data and stock ledger from the API.

use crate::auth::use_auth;
use crate::components::common::{
    Button, ButtonVariant, Modal, ModalSize, StatCard, StatCardVariant,
    use_toast,
};
use crate::pages::item_list::{derive_status, InventoryItem};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
/* ── Layout ── */
.item-detail-page {
    max-width: 960px;
    margin: 0 auto;
}

.item-detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 20px;
    gap: 16px;
    flex-wrap: wrap;
}

.item-detail-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.item-detail-back {
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

.item-detail-back:hover { text-decoration: underline; }

.item-detail-title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.item-detail-title-row h1 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
}

.item-detail-code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 4px;
}

.item-detail-status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
}

.item-detail-status-active {
    background: rgba(40, 167, 69, 0.1);
    color: #28a745;
}

.item-detail-status-low {
    background: rgba(255, 193, 7, 0.15);
    color: #d4a017;
}

.item-detail-status-out {
    background: rgba(108, 117, 125, 0.1);
    color: #6c757d;
}

.item-detail-status-discontinued {
    background: rgba(220, 53, 69, 0.1);
    color: #dc3545;
}

/* ── KPI Grid ── */
.item-detail-kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
}

/* ── Section Cards ── */
.item-detail-section {
    background: #fff;
    border: 1px solid var(--border-color, #e0e0e0);
    border-radius: var(--radius, 8px);
    padding: 20px;
    margin-bottom: 16px;
}

.item-detail-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.item-detail-section-header h2 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
}

.item-detail-section-header .section-badge {
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 10px;
}

/* ── Info Grid ── */
.item-detail-info-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 14px;
}

.item-detail-field {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.item-detail-field-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
}

.item-detail-field-value {
    font-size: 14px;
    color: var(--text-primary);
}

.item-detail-field-value.monospace {
    font-family: monospace;
    font-size: 13px;
}

.item-detail-field-value.text-success { color: #28a745; }
.item-detail-field-value.text-danger { color: #dc3545; }
.item-detail-field-value.text-warning { color: #d4a017; }

/* ── Classification Chips ── */
.item-detail-classification {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
}

.classification-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    background: var(--bg-muted, #f5f5f5);
    color: var(--text-primary);
    border: 1px solid var(--border-color, #e0e0e0);
}

.classification-chip.active {
    background: rgba(74, 144, 217, 0.08);
    border-color: var(--accent, #4a90d9);
    color: var(--accent, #4a90d9);
}

/* ── Notes Box ── */
.item-detail-notes {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.6;
    padding: 12px;
    background: var(--bg-muted, #f5f5f5);
    border-radius: 6px;
    margin: 0;
}

/* ── Action Bar ── */
.item-detail-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color, #e0e0e0);
    flex-wrap: wrap;
}

.item-detail-actions-left,
.item-detail-actions-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

/* ── Ledger Table ── */
.item-detail-ledger {
    margin-top: 0;
}

.ledger-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.ledger-table thead th {
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

.ledger-table thead th.text-right {
    text-align: right;
}

.ledger-table tbody td {
    padding: 8px 10px;
    border-bottom: 1px solid var(--border-color, #e0e0e0);
    color: var(--text-primary);
}

.ledger-table tbody td.text-right {
    text-align: right;
    font-family: monospace;
    font-size: 12px;
}

.ledger-table tbody td.text-danger { color: #dc3545; }
.ledger-table tbody td.text-success { color: #28a745; }

.ledger-table tbody tr:last-child td {
    border-bottom: none;
}

.ledger-table tbody tr:hover {
    background: rgba(74, 144, 217, 0.03);
}

.ledger-type-in {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: #28a745;
    font-weight: 500;
}

.ledger-type-out {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: #dc3545;
    font-weight: 500;
}

.ledger-type-adjust {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: #d4a017;
    font-weight: 500;
}

.ledger-empty {
    text-align: center;
    padding: 30px 20px;
    color: var(--text-secondary);
    font-size: 14px;
}

/* ── Loading ── */
.item-detail-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
    gap: 16px;
    color: var(--text-secondary);
}

.item-detail-loading .loading-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9);
    border-radius: 50%;
    animation: item-detail-spin 0.8s linear infinite;
}

@keyframes item-detail-spin {
    to { transform: rotate(360deg); }
}

/* ── Responsive ── */
@media (max-width: 768px) {
    .item-detail-header { flex-direction: column; }
    .item-detail-title-row { flex-direction: column; align-items: flex-start; }
    .item-detail-kpis { grid-template-columns: 1fr 1fr; }
    .item-detail-info-grid { grid-template-columns: 1fr; }
    .item-detail-actions { flex-direction: column; align-items: stretch; }
    .item-detail-actions-left,
    .item-detail-actions-right {
        justify-content: center;
    }
    .ledger-table { font-size: 12px; }
    .ledger-table thead th,
    .ledger-table tbody td { padding: 6px 8px; }
}
"##;

// ============================================================================
// Data Models
// ============================================================================

/// A single stock movement entry for the ledger table.
#[derive(Clone, Debug)]
struct LedgerEntry {
    date: String,
    entry_type: String,    // "IN" | "OUT" | "ADJ"
    reference: String,     // e.g. "GRN-001", "SINV-045", "ADJ-003"
    quantity: i32,
    unit_cost: f64,
    total_value: f64,
    running_balance: i32,
    notes: String,
}


// ============================================================================
// Helpers
// ============================================================================

fn compute_margin(item: &InventoryItem) -> f64 {
    if item.standard_selling_price > 0.0 {
        ((item.standard_selling_price - item.standard_cost) / item.standard_selling_price) * 100.0
    } else {
        0.0
    }
}

fn stock_value(item: &InventoryItem) -> f64 {
    item.current_stock as f64 * item.standard_cost
}

fn status_class(status: &str) -> &'static str {
    match status {
        "Active" => "item-detail-status-active",
        "Low Stock" => "item-detail-status-low",
        "Out of Stock" => "item-detail-status-out",
        "Discontinued" => "item-detail-status-discontinued",
        _ => "item-detail-status-active",
    }
}

// ============================================================================
// Component
// ============================================================================

/// The Item Detail page — shows full item information, KPI cards,
/// stock ledger entries, and action buttons.
#[component]
pub fn ItemDetailPage(id: String) -> Element {
    let mut toast = use_toast();
    let navigator = use_navigator();

    // ── Async data fetch ──
    let api = use_auth().api;
    let id_for_resource = id.clone();
    let item_resource = use_resource(move || {
        let api = api.clone();
        let id_for_fetch = id_for_resource.clone();
        async move {
            let parsed_id = id_for_fetch.parse::<i64>().unwrap_or(0);
            let client = api.with(|c| c.clone());
            // Fetch item detail
            let item = client.get_item(parsed_id).await.ok()?;
            let status = derive_status(&item);
            // Fetch stock ledger for this item
            let movements = client.list_stock_movements_by_item(parsed_id).await.unwrap_or_default();
            Some((item, status, movements))
        }
    });

    let is_loading = item_resource.read().is_none();

    // ── Delete confirmation modal state ──
    let mut show_delete_modal = use_signal(|| false);

    // ── Handler closures (defined early so they're available everywhere) ──

    let on_back2 = {
        let nav = navigator.clone();
        move |_: Event<MouseData>| {
            nav.push("/inventory/items");
        }
    };

    let on_edit = {
        let nav = navigator.clone();
        let item_id = id.clone();
        move |_| {
            nav.push(format!("/inventory/items/{}/edit", item_id));
        }
    };

    let on_stock_movement = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/inventory/stock-movements/new");
        }
    };

    let on_stock_ledger = {
        let nav = navigator.clone();
        let id_for_nav = id.clone();
        move |_| {
            nav.push(format!("/inventory/stock-ledger/{}", id_for_nav));
        }
    };

    let on_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| {
            modal.set(true);
        }
    };

    let confirm_delete = {
        let mut toast = toast.clone();
        let nav = navigator.clone();
        let mut modal = show_delete_modal.clone();
        move |_| {
            modal.set(false);
            toast.success("Item Deleted", "Item has been deleted successfully.");
            nav.push("/inventory/items");
        }
    };

    let cancel_delete = {
        let mut modal = show_delete_modal.clone();
        move |_| {
            modal.set(false);
        }
    };

    // ── Handle loading state ──
    if is_loading {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page item-detail-page",
                div { class: "item-detail-loading",
                    div { class: "loading-spinner" }
                    span { "Loading item details…" }
                }
            }
        };
    }

    // ── Pre-compute detail values from fetched data ──
    let data_opt = item_resource.read().as_ref().cloned().flatten();
    let Some((server_item, status, movements)) = data_opt else {
        return rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page item-detail-page",
                div { class: "item-detail-loading",
                    div { style: "font-size: 40px;", "📦" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Item Not Found" }
                    p { "No item with ID \"{id}\" was found." }
                    Button { variant: ButtonVariant::Primary, onclick: on_back2, "← Back to Items" }
                }
            }
        };
    };

    let item = InventoryItem {
        id: server_item.id,
        item_code: server_item.item_code.clone(),
        item_name: server_item.item_name.clone(),
        category: server_item.category.clone(),
        unit_of_measure: server_item.unit_of_measure.clone(),
        current_stock: server_item.current_stock as i32,
        reorder_level: server_item.reorder_level as i32,
        standard_cost: server_item.standard_cost,
        standard_selling_price: server_item.selling_price,
        is_raw_material: server_item.is_raw_material,
        is_finished_good: server_item.is_finished_good,
        is_purchased: server_item.is_purchased,
        is_manufactured: server_item.is_manufactured,
        status: status.clone(),
        last_updated: server_item.updated_at.clone(),
        warehouse: String::new(),
    };

    let margin_pct = compute_margin(&item);
    let stock_val = stock_value(&item);
    let ledger: Vec<LedgerEntry> = movements.iter().map(|m| {
        let qty = m.quantity as i32;
        let total_val = m.quantity * m.unit_cost;
        LedgerEntry {
            date: m.created_at.clone(),
            entry_type: match m.movement_type.as_str() {
                "IN" => "IN".to_string(),
                "OUT" => "OUT".to_string(),
                _ => "ADJ".to_string(),
            },
            reference: m.movement_no.clone(),
            quantity: qty,
            unit_cost: m.unit_cost,
            total_value: total_val,
            running_balance: 0,
            notes: m.notes.clone(),
        }
    }).collect();

    let raw_mat_chip = if item.is_raw_material { "classification-chip active" } else { "classification-chip" };
    let fg_chip = if item.is_finished_good { "classification-chip active" } else { "classification-chip" };
    let purchased_chip = if item.is_purchased { "classification-chip active" } else { "classification-chip" };
    let manufactured_chip = if item.is_manufactured { "classification-chip active" } else { "classification-chip" };
    let source_desc = if item.is_purchased { "purchased from suppliers." } else { "manufactured in-house." };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page item-detail-page",

            // ── Header ──
            div { class: "item-detail-header",
                div { class: "item-detail-title-group",
                    button {
                        class: "item-detail-back",
                        r#type: "button",
                        onclick: on_back2,
                        "← Back to Items"
                    }
                    div { class: "item-detail-title-row",
                        h1 { "{item.item_name}" }
                        span { class: "item-detail-code", "{item.item_code}" }
                        span {
                            class: "item-detail-status-badge {status_class(&item.status)}",
                            match item.status.as_str() {
                                "Active" => "✓ Active",
                                "Low Stock" => "⚠ Low Stock",
                                "Out of Stock" => "✗ Out of Stock",
                                "Discontinued" => "— Discontinued",
                                _ => "— {item.status}",
                            }
                        }
                    }
                }
            }

            // ── KPI Stat Cards ──
            div { class: "item-detail-kpis",
                StatCard {
                    title: "Current Stock".to_string(),
                    value: format!("{} {}", item.current_stock, item.unit_of_measure),
                    variant: if item.current_stock == 0 { StatCardVariant::Danger }
                             else if item.current_stock <= item.reorder_level { StatCardVariant::Warning }
                             else { StatCardVariant::Primary },
                    icon: Some("📦".to_string()),
                    footer: Some(format!("Reorder at {}", item.reorder_level)),
                }
                StatCard {
                    title: "Stock Value".to_string(),
                    value: format!("PKR {:.0}", stock_val),
                    variant: StatCardVariant::Primary,
                    icon: Some("💰".to_string()),
                    footer: Some(format!("Cost: PKR {:.2}/{}", item.standard_cost, item.unit_of_measure)),
                }
                StatCard {
                    title: "Selling Price".to_string(),
                    value: format!("PKR {:.2}", item.standard_selling_price),
                    variant: StatCardVariant::Success,
                    icon: Some("🏷️".to_string()),
                    footer: Some(format!("PKR {:.2} margin", item.standard_selling_price - item.standard_cost)),
                }
                StatCard {
                    title: "Profit Margin".to_string(),
                    value: format!("{:.1}%", margin_pct),
                    variant: if margin_pct >= 30.0 { StatCardVariant::Success }
                             else if margin_pct >= 15.0 { StatCardVariant::Primary }
                             else { StatCardVariant::Warning },
                    icon: if margin_pct >= 30.0 { Some("📈".to_string()) }
                          else if margin_pct >= 15.0 { Some("📊".to_string()) }
                          else { Some("📉".to_string()) },
                    footer: Some(if margin_pct >= 30.0 { "Healthy margin".to_string() }
                                   else if margin_pct >= 15.0 { "Average margin".to_string() }
                                   else { "Low margin — review pricing".to_string() }),
                }
            }

            // ── Section: Item Details ──
            div { class: "item-detail-section",
                div { class: "item-detail-section-header",
                    h2 { "Item Details" }
                    span { class: "section-badge", "General Information" }
                }
                div { class: "item-detail-info-grid",
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Category" }
                        span { class: "item-detail-field-value", "{item.category}" }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Unit of Measure" }
                        span { class: "item-detail-field-value", "{item.unit_of_measure}" }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Warehouse" }
                        span { class: "item-detail-field-value", "{item.warehouse}" }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Standard Cost" }
                        span { class: "item-detail-field-value monospace",
                            "PKR {item.standard_cost:.2}"
                        }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Selling Price" }
                        span { class: "item-detail-field-value monospace",
                            "PKR {item.standard_selling_price:.2}"
                        }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Reorder Level" }
                        span { class: "item-detail-field-value monospace",
                            if item.current_stock <= item.reorder_level {
                                span { class: "text-warning", "{item.reorder_level} — STOCK LOW" }
                            } else {
                                span { "{item.reorder_level}" }
                            }
                        }
                    }
                    div { class: "item-detail-field",
                        span { class: "item-detail-field-label", "Last Updated" }
                        span { class: "item-detail-field-value", "{item.last_updated}" }
                    }
                }
            }

            // ── Section: Classification ──
            div { class: "item-detail-section",
                div { class: "item-detail-section-header",
                    h2 { "Classification" }
                    span { class: "section-badge", "Item Type" }
                }
                div { class: "item-detail-classification",
                    div {
                        class: "{raw_mat_chip}",
                        span { "🛢 Raw Material" }
                    }
                    div {
                        class: "{fg_chip}",
                        span { "📦 Finished Good" }
                    }
                    div {
                        class: "{purchased_chip}",
                        span { "🛒 Purchased" }
                    }
                    div {
                        class: "{manufactured_chip}",
                        span { "⚙ Manufactured" }
                    }
                }
            }

            // ── Section: Stock Ledger ──
            div { class: "item-detail-section item-detail-ledger",
                div { class: "item-detail-section-header",
                    h2 { "Stock Ledger" }
                    span { class: "section-badge", "{ledger.len()} entries" }
                }
                table { class: "ledger-table",
                    thead {
                        tr {
                            th { "Date" }
                            th { "Type" }
                            th { "Reference" }
                            th { class: "text-right", "Qty" }
                            th { class: "text-right", "Unit Cost" }
                            th { class: "text-right", "Total" }
                            th { class: "text-right", "Balance" }
                            th { "Notes" }
                        }
                    }
                    tbody {
                        {ledger.iter().map(|entry: &LedgerEntry| {
                            let type_class = match entry.entry_type.as_str() {
                                "IN" => "ledger-type-in",
                                "OUT" => "ledger-type-out",
                                _ => "ledger-type-adjust",
                            };
                            let type_icon = match entry.entry_type.as_str() {
                                "IN" => "⬇",
                                "OUT" => "⬆",
                                _ => "↔",
                            };
                            let qty_class = if entry.quantity > 0 { "text-success" } else { "text-danger" };
                            rsx! {
                                tr {
                                    td { "{entry.date}" }
                                    td {
                                        span { class: type_class,
                                            "{type_icon} {entry.entry_type}"
                                        }
                                    }
                                    td { "{entry.reference}" }
                                    td { class: "text-right {qty_class}",
                                        if entry.quantity > 0 { "+{entry.quantity}" }
                                        else { "{entry.quantity}" }
                                    }
                                    td { class: "text-right",
                                        if entry.unit_cost > 0.0 {
                                            "PKR {entry.unit_cost:.2}"
                                        } else {
                                            "—"
                                        }
                                    }
                                    td { class: "text-right",
                                        if entry.total_value > 0.0 {
                                            "PKR {entry.total_value:.2}"
                                        } else {
                                            "—"
                                        }
                                    }
                                    td { class: "text-right", "{entry.running_balance}" }
                                    td { "{entry.notes}" }
                                }
                            }
                        })}
                    }
                }
            }

            // ── Section: Notes ──
            div { class: "item-detail-section",
                div { class: "item-detail-section-header",
                    h2 { "Notes" }
                }
                p { class: "item-detail-notes",
                    "Item {item.item_name} ({item.item_code}) is a {item.category} item stored at {item.warehouse}. ",
                    if item.is_finished_good { "This is a finished good. " }
                    else if item.is_raw_material { "This is a raw material. " }
                    else { "This item is {source_desc} " },
                    "Current stock level: {item.current_stock} {item.unit_of_measure} (reorder at {item.reorder_level})."
                }
            }

            // ── Action Bar ──
            div { class: "item-detail-actions",
                div { class: "item-detail-actions-left",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: on_edit,
                        icon: Some("✏️".to_string()),
                        "Edit Item"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: on_stock_movement,
                        icon: Some("📦".to_string()),
                        "Record Movement"
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: on_stock_ledger,
                        icon: Some("📋".to_string()),
                        "Full Ledger"
                    }
                }
                div { class: "item-detail-actions-right",
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
                title: Some("Delete Item".to_string()),
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
                        "Delete Item"
                    }
                },
                div {
                    p { style: "margin: 0 0 8px 0; color: var(--text-primary); font-size: 14px; font-weight: 500;",
                        "Are you sure you want to delete {item.item_name}?"
                    }
                    p { style: "margin: 0; color: var(--text-secondary); font-size: 13px;",
                        "This action cannot be undone. The item \"{item.item_code}\" will be permanently removed from the system. ",
                        "All stock ledger history will be preserved."
                    }
                }
            }
        }
    }
}
