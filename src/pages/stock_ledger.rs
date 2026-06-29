//! Stock Ledger Page — DataGrid-backed ledger view of all stock movements
//! for a specific item, showing running balance, cost, and value.

use crate::auth::use_auth;
use crate::components::data_grid::{
    BadgeColor, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType, PaginationMode,
    RowHeight, SelectionMode, TextAlign,
};
use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant};
use dioxus::prelude::*;
use std::collections::HashSet;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.ledger-page {
    max-width: 1100px;
    margin: 0 auto;
}

.ledger-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 20px;
    gap: 16px;
    flex-wrap: wrap;
}

.ledger-title-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.ledger-back {
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

.ledger-back:hover { text-decoration: underline; }

.ledger-title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.ledger-title-row h1 {
    font-size: 22px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
}

.ledger-item-code {
    font-family: monospace;
    font-size: 13px;
    color: var(--text-secondary);
    background: var(--bg-muted, #f5f5f5);
    padding: 2px 8px;
    border-radius: 4px;
}

.ledger-kpis {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
    margin-bottom: 20px;
}

.ledger-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 40vh;
    gap: 16px;
    color: var(--text-secondary);
}

.ledger-loading .loading-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--border-color, #e0e0e0);
    border-top-color: var(--accent, #4a90d9);
    border-radius: 50%;
    animation: ledger-spin 0.8s linear infinite;
}

@keyframes ledger-spin {
    to { transform: rotate(360deg); }
}

@media (max-width: 768px) {
    .ledger-header { flex-direction: column; }
    .ledger-title-row { flex-direction: column; align-items: flex-start; }
    .ledger-kpis { grid-template-columns: 1fr 1fr; }
}
"##;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Clone, PartialEq, Debug)]
pub struct LedgerEntry {
    pub date: String,
    pub movement_type: String, // "IN" | "OUT" | "ADJ" | "TRANSFER"
    pub reference: String,
    pub quantity: f64,
    pub unit_cost: f64,
    pub total_value: f64,
    pub running_balance: f64,
    pub notes: String,
}



// ============================================================================
// Component
// ============================================================================

#[component]
pub fn StockLedgerPage(item_id: String) -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;
    let item_id_for_display = item_id.clone();
    let item_id_for_ledger = item_id.clone();

    // ── Find item info ──
    let item_id_for_fetch = item_id_for_display.clone();
    let item_resource = use_resource(move || {
        let id = item_id_for_fetch.clone();
        let api = api.clone();
        async move {
            let client = api.read().clone();
            client.get_item(id.parse().unwrap_or(0)).await.ok()
        }
    });

    let is_loading = item_resource.read().is_none();
    let item_opt = item_resource.read().as_ref().cloned().flatten();

    // ── Ledger data ──
    let refresh_counter = use_signal(|| 0u32);
    let ledger_resource = use_resource(move || {
        let id = item_id_for_ledger.clone();
        let api = api.clone();
        async move {
            let _ = *refresh_counter.read();
            let client = api.read().clone();
            let item_id_num = id.parse::<i64>().unwrap_or(0);
            let movements = client.list_stock_movements().await.unwrap_or_default();
            // ponytail: compute running balance from all movements for this item
            let mut entries: Vec<LedgerEntry> = movements
                .into_iter()
                .filter(|m| m.item_id == item_id_num)
                .map(|m| {
                    let qty = if m.movement_type == "OUT" { -m.quantity } else { m.quantity };
                    LedgerEntry {
                        date: m.created_at,
                        movement_type: m.movement_type,
                        reference: match (&m.reference_doctype, &m.reference_docno) {
                            (Some(dt), Some(dn)) => format!("{} {}", dt, dn),
                            _ => "-".to_string(),
                        },
                        quantity: qty,
                        unit_cost: m.unit_cost,
                        total_value: qty * m.unit_cost,
                        running_balance: 0.0,
                        notes: m.notes,
                    }
                })
                .collect::<Vec<_>>();
            entries.sort_by(|a, b| a.date.cmp(&b.date));
            let mut balance = 0.0;
            for e in entries.iter_mut() {
                balance += e.quantity;
                e.running_balance = balance;
            }
            entries
        }
    });

    let selected_ids = use_signal(|| HashSet::<usize>::new());
    let ledger_is_loading = ledger_resource.read().is_none();
    let entries = ledger_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    // ── Running totals ──
    let total_in: f64 = entries.iter().filter(|e| e.quantity > 0.0).map(|e| e.quantity).sum();
    let total_out: f64 = entries.iter().filter(|e| e.quantity < 0.0).map(|e| e.quantity.abs()).sum();
    let current_balance = entries.last().map(|e| e.running_balance).unwrap_or(0.0);

    // ── Columns ──
    let columns: Vec<ColumnDef<LedgerEntry>> = vec![
        ColumnDef::text("date", "Date", |e: &LedgerEntry| e.date.clone())
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
        ColumnDef::text("type", "Type", |e: &LedgerEntry| e.movement_type.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("IN", BadgeColor::Green),
                    ("OUT", BadgeColor::Red),
                    ("ADJ", BadgeColor::Yellow),
                    ("TRANSFER", BadgeColor::Blue),
                ],
                default_color: BadgeColor::Gray,
            })
            .with_filter(FilterType::Select {
                options: vec!["IN".to_string(), "OUT".to_string(), "ADJ".to_string(), "TRANSFER".to_string()],
            }),
        ColumnDef::text("ref", "Reference", |e: &LedgerEntry| e.reference.clone())
            .with_width(ColumnWidth::Px(130)),
        ColumnDef::text("qty", "Qty", |e: &LedgerEntry| e.quantity.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(90))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
        ColumnDef::text("cost", "Unit Cost", |e: &LedgerEntry| e.unit_cost.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("value", "Total Value", |e: &LedgerEntry| e.total_value.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),
        ColumnDef::text("balance", "Running Balance", |e: &LedgerEntry| e.running_balance.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(130))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),
        ColumnDef::text("notes", "Notes", |e: &LedgerEntry| e.notes.clone())
            .with_width(ColumnWidth::Fr(1.0))
            .with_filter(FilterType::Text),
    ];

    // ── Handlers ──

    let on_back = move |_: Event<MouseData>| {
        navigator.push("/inventory/stock-movements");
    };

    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| { counter += 1; }
    };

    // ── Render ──

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page ledger-page",

            // Loading state
            if is_loading {
                div { class: "ledger-loading",
                    div { class: "loading-spinner" }
                    span { "Loading ledger…" }
                }
            } else if item_opt.is_none() {
                div { class: "ledger-loading",
                    div { style: "font-size: 40px;", "📋" }
                    h2 { style: "margin: 0; color: var(--text-primary);", "Item Not Found" }
                    p { "No item with ID \"{item_id_for_display}\" was found." }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| { navigator.push("/inventory/stock-movements"); },
                        "← Back to Movements"
                    }
                }
            } else if let Some(ref item) = item_opt {
                // ── Header ──
                div { class: "ledger-header",
                    div { class: "ledger-title-group",
                        button {
                            class: "ledger-back",
                            r#type: "button",
                            onclick: on_back,
                            "← Back to Movements"
                        }
                        div { class: "ledger-title-row",
                            h1 { "Stock Ledger — {item.item_name}" }
                            span { class: "ledger-item-code", "{item.item_code}" }
                        }
                        p { style: "margin: 4px 0 0 0; font-size: 13px; color: var(--text-secondary);",
                            "Category: {item.category} | UoM: {item.unit_of_measure}"
                        }
                    }
                }

                // ── KPI Summary ──
                div { class: "ledger-kpis",
                    StatCard {
                        title: "Current Balance".to_string(),
                        value: format!("{:.0} {:.1}", current_balance, item.unit_of_measure),
                        variant: if current_balance == 0.0 { StatCardVariant::Danger }
                                 else { StatCardVariant::Primary },
                        icon: Some("📊".to_string()),
                    }
                    StatCard {
                        title: "Total Received".to_string(),
                        value: format!("{:.0}", total_in),
                        variant: StatCardVariant::Success,
                        icon: Some("⬇".to_string()),
                    }
                    StatCard {
                        title: "Total Issued".to_string(),
                        value: format!("{:.0}", total_out),
                        variant: StatCardVariant::Warning,
                        icon: Some("⬆".to_string()),
                    }
                    StatCard {
                        title: "Entries".to_string(),
                        value: format!("{}", entries.len()),
                        variant: StatCardVariant::Default,
                        icon: Some("📝".to_string()),
                    }
                }

                // ── Toolbar ──
                div { class: "invoice-toolbar",
                    div { class: "toolbar-left",
                        button { class: "toolbar-btn", r#type: "button", onclick: on_refresh, "🔄 Refresh" }
                    }
                    div { class: "toolbar-right",
                        span { style: "font-size: 12px; color: var(--text-secondary);",
                            "Last entry: {entries.last().map(|e| e.date.clone()).unwrap_or_default()}"
                        }
                    }
                }

                // ── DataGrid ──
                DataGrid {
                    columns: columns.clone(),
                    rows: entries.clone(),
                    pagination: PaginationMode::Client { page_size: 15 },
                    selection_mode: SelectionMode::None,
                    striped: true,
                    hoverable: true,
                    row_height: RowHeight::Standard,
                    selected_rows: selected_ids,
                    loading: ledger_is_loading,
                    skeleton: ledger_is_loading,
                    skeleton_rows: 8,
                }
            }
        }
    }
}
