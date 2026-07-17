//! Inventory Item List Page — A full-featured DataGrid-backed list view for
//! managing inventory items with sorting, filtering, pagination, cell class
//! rules for stock levels, and toolbar actions.

use crate::components::data_grid::{
    BadgeColor, CellClassRule, CellRenderer, ColumnDef, ColumnWidth, DataGrid, FilterType,
    PaginationMode, RowHeight, SelectionMode, TextAlign,
};
use dioxus::prelude::*;
use std::collections::HashSet;

use crate::auth::use_auth;

// ============================================================================
// Data Model
// ============================================================================

/// Represents an inventory item in the MiniERP system.
///
/// Fields map to the `items` table described in the PRD (§4, table 7).
#[derive(Clone, PartialEq, Debug)]
pub struct InventoryItem {
    pub id: i64,
    pub item_code: String,
    pub item_name: String,
    pub category: String,
    pub unit_of_measure: String,
    pub current_stock: i32,
    pub reorder_level: i32,
    pub standard_cost: f64,
    pub standard_selling_price: f64,
    pub is_raw_material: bool,
    pub is_finished_good: bool,
    pub is_purchased: bool,
    pub is_manufactured: bool,
    pub status: String,        // "Active" | "Discontinued" | "Out of Stock" | "Low Stock"
    pub last_updated: String,  // "2026-06-15"
    pub warehouse: String,
}

/// Derive a display status string from the server-side item model.
pub fn derive_status(item: &crate::models::Item) -> String {
    if !item.is_active {
        "Discontinued".to_string()
    } else if item.current_stock <= 0.0 {
        "Out of Stock".to_string()
    } else if item.reorder_level > 0.0 && item.current_stock <= item.reorder_level {
        "Low Stock".to_string()
    } else {
        "Active".to_string()
    }
}

// ============================================================================
// Summary Calculations
// ============================================================================

struct ItemSummary {
    total_count: usize,
    category_count: usize,
    low_stock_count: usize,
    out_of_stock_count: usize,
    total_stock_value: f64,
}

fn compute_summary(items: &[InventoryItem]) -> ItemSummary {
    let total_count = items.len();
    let mut categories = HashSet::new();
    let mut low_stock_count = 0;
    let mut out_of_stock_count = 0;
    let mut total_stock_value = 0.0;

    for item in items {
        categories.insert(item.category.clone());
        if item.current_stock == 0 {
            out_of_stock_count += 1;
        } else if item.current_stock <= item.reorder_level {
            low_stock_count += 1;
        }
        total_stock_value += item.current_stock as f64 * item.standard_cost;
    }

    ItemSummary {
        total_count,
        category_count: categories.len(),
        low_stock_count,
        out_of_stock_count,
        total_stock_value,
    }
}

// ============================================================================
// Component
// ============================================================================

/// The Inventory Item List page — full DataGrid with filters, sorting,
/// pagination, cell styling, and toolbar actions.
#[component]
pub fn ItemListPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    // ── Async data fetch (with refresh support) ──
    let refresh_counter = use_signal(|| 0u32);
    let items_resource = use_resource(move || async move {
        // Read the counter to create a dependency that can be bumped
        let _ = *refresh_counter.read();
        let client = api.with(|c| c.clone());
        client.list_items().await.unwrap_or_default().into_iter().map(|i| InventoryItem {
            id: i.id,
            item_code: i.item_code.clone(),
            item_name: i.item_name.clone(),
            category: i.category.clone(),
            unit_of_measure: i.unit_of_measure.clone(),
            current_stock: i.current_stock as i32,
            reorder_level: i.reorder_level as i32,
            standard_cost: i.standard_cost,
            standard_selling_price: i.selling_price,
            is_raw_material: i.is_raw_material,
            is_finished_good: i.is_finished_good,
            is_purchased: i.is_purchased,
            is_manufactured: i.is_manufactured,
            status: derive_status(&i),
            last_updated: i.updated_at.clone(),
            warehouse: String::new(), // ponytail: not in list endpoint
        }).collect::<Vec<_>>()
    });
    let selected_ids = use_signal(|| HashSet::<usize>::new());

    // ── Derive loading state and data ──
    let is_loading = items_resource.read().is_none();
    let items = items_resource.read()
        .as_ref()
        .cloned()
        .unwrap_or_default();

    // ── Summary ──
    let summary = compute_summary(&items);

    // ── Column Definitions ──

    let columns: Vec<ColumnDef<InventoryItem>> = vec![
        // Item code — text with text filter
        ColumnDef::text("code", "Code", |item: &InventoryItem| item.item_code.clone())
            .with_width(ColumnWidth::Px(120))
            .with_filter(FilterType::Text)
            .with_editable(true),

        // Item name — text with text filter, fills remaining space
        ColumnDef::text("name", "Item Name", |item: &InventoryItem| item.item_name.clone())
            .with_width(ColumnWidth::Fr(1.2))
            .with_filter(FilterType::Text)
            .with_editable(true),

        // Category — text with select filter
        ColumnDef::text("category", "Category", |item: &InventoryItem| item.category.clone())
            .with_width(ColumnWidth::Px(140))
            .with_filter(FilterType::Select {
                options: vec![
                    "Widgets".to_string(), "Fasteners".to_string(), "Raw Materials".to_string(),
                    "Equipment".to_string(), "Consumables".to_string(), "Electrical".to_string(),
                    "Packaging".to_string(), "Safety".to_string(),
                ],
            }),

        // Unit — text
        ColumnDef::text("uom", "UOM", |item: &InventoryItem| item.unit_of_measure.clone())
            .with_width(ColumnWidth::Px(80)),

        // Stock — number renderer with number filter
        ColumnDef::text("stock", "Stock", |item: &InventoryItem| item.current_stock.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(100))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 })
            .with_filter(FilterType::Number)
            .with_cell_class(CellClassRule::new(|item: &InventoryItem| {
                if item.current_stock == 0 { "text-danger fw-bold".to_string() }
                else if item.current_stock <= item.reorder_level { "text-warning".to_string() }
                else { String::new() }
            })),

        // Reorder Level — number renderer
        ColumnDef::text("reorder", "Reorder", |item: &InventoryItem| item.reorder_level.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Number { prefix: "", decimals: 0 }),

        // Cost — currency renderer
        ColumnDef::text("cost", "Cost", |item: &InventoryItem| item.standard_cost.to_string())
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 }),

        // Selling Price — currency renderer with number filter, editable
        ColumnDef::text("price", "Sell Price", |item: &InventoryItem| {
            item.standard_selling_price.to_string()
        })
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(110))
            .with_renderer(CellRenderer::Currency { code: "PKR", decimals: 2 })
            .with_filter(FilterType::Number)
            .with_editable(true),

        // Margin — percentage renderer (computed)
        ColumnDef::text("margin", "Margin", |item: &InventoryItem| {
            if item.standard_cost > 0.0 {
                ((item.standard_selling_price - item.standard_cost) / item.standard_selling_price)
                    .to_string()
            } else {
                "0".to_string()
            }
        })
            .with_align(TextAlign::Right)
            .with_width(ColumnWidth::Px(80))
            .with_renderer(CellRenderer::Percentage { decimals: 1 }),

        // Status — badge renderer with select filter
        ColumnDef::text("status", "Status", |item: &InventoryItem| item.status.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Badge {
                color_map: vec![
                    ("Active", BadgeColor::Green),
                    ("Discontinued", BadgeColor::Red),
                    ("Out of Stock", BadgeColor::Gray),
                    ("Low Stock", BadgeColor::Yellow),
                ],
                default_color: BadgeColor::Blue,
            })
            .with_filter(FilterType::Select {
                options: vec![
                    "Active".to_string(),
                    "Discontinued".to_string(),
                    "Out of Stock".to_string(),
                    "Low Stock".to_string(),
                ],
            }),

        // Warehouse — text
        ColumnDef::text("warehouse", "Warehouse", |item: &InventoryItem| item.warehouse.clone())
            .with_width(ColumnWidth::Px(150)),

        // Last Updated — date renderer with date filter
        ColumnDef::text("updated", "Updated", |item: &InventoryItem| item.last_updated.clone())
            .with_width(ColumnWidth::Px(120))
            .with_renderer(CellRenderer::Date { format: "%d-%b-%Y" })
            .with_filter(FilterType::Date),
    ];

    // ── Navigation Handlers ──

    // Navigate to item detail on row click
    let on_row_click = {
        let nav = navigator.clone();
        move |(_idx, item): (usize, InventoryItem)| {
            nav.push(format!("/inventory/items/{}", item.id));
        }
    };

    // Navigate to create new item
    let on_new_item = {
        let nav = navigator.clone();
        move |_| {
            nav.push("/inventory/items/new");
        }
    };

    // Refresh: bump the counter to re-trigger the resource
    let on_refresh = {
        let mut counter = refresh_counter.clone();
        move |_| {
            counter += 1;
        }
    };

    // Export: log for now (placeholder for CSV download)
    let on_export = move |_| {
        tracing::info!("Export inventory items to CSV");
    };

    let on_cell_edit = move |(row_idx, col_key, _old_val, new_val): (usize, &'static str, String, String)| {
        tracing::info!(
            "Cell edited: row={}, col={}, new_value={}",
            row_idx, col_key, new_val,
        );
    };

    // ── Render ──

    let sel_count = selected_ids.read().len();

    rsx! {
        div { class: "page item-list-page",

            // ── Page Header ──
            div { class: "page-header",
                div {
                    h1 { "Inventory Items" }
                    p { class: "page-subtitle",
                        "Manage your inventory — track stock levels, costs, and prices. ",
                        "Click any row to view details, or select items for batch actions."
                    }
                }
                if is_loading {
                    div { class: "loading-badge",
                        div { class: "loading-badge-spinner" }
                        span { "Loading…" }
                    }
                }
            }

            // ── Summary Bar ──
            div { class: "invoice-summary-bar",
                if is_loading {
                    {[0; 5].iter().map(|_| {
                        rsx! {
                            div { class: "summary-item summary-skeleton",
                                div { class: "skeleton-text", style: "width: 60%; height: 10px;" }
                                div { class: "skeleton-text", style: "width: 80%; height: 20px; margin-top: 6px;" }
                            }
                        }
                    })}
                } else {
                    div { class: "summary-item",
                        span { class: "summary-label", "Total Items" }
                        span { class: "summary-value", "{summary.total_count}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Categories" }
                        span { class: "summary-value", "{summary.category_count}" }
                    }
                    div { class: "summary-item summary-warning",
                        span { class: "summary-label", "Low Stock" }
                        span { class: "summary-value", "{summary.low_stock_count}" }
                    }
                    div { class: "summary-item summary-warning",
                        span { class: "summary-label", "Out of Stock" }
                        span { class: "summary-value", "{summary.out_of_stock_count}" }
                    }
                    div { class: "summary-item",
                        span { class: "summary-label", "Stock Value" }
                        span { class: "summary-value summary-amount",
                            "PKR {summary.total_stock_value:.0}"
                        }
                    }
                }
            }

            // ── Toolbar ──
            div { class: "invoice-toolbar",
                div { class: "toolbar-left",
                    button {
                        class: "toolbar-btn toolbar-btn-primary",
                        r#type: "button",
                        disabled: is_loading,
                        onclick: on_new_item,
                        "＋ New Item"
                    }
                    button {
                        class: "toolbar-btn",
                        r#type: "button",
                        disabled: is_loading,
                        onclick: on_export,
                        "📥 Export"
                    }
                    button {
                        class: "toolbar-btn",
                        r#type: "button",
                        disabled: is_loading,
                        onclick: on_refresh,
                        "🔄 Refresh"
                    }
                }
                div { class: "toolbar-right",
                    if sel_count > 0 {
                        span { class: "toolbar-selection",
                            "{sel_count} item(s) selected"
                        }
                    }
                }
            }

            // ── DataGrid ──
            DataGrid {
                columns: columns.clone(),
                rows: items.clone(),
                pagination: PaginationMode::Client { page_size: 10 },
                selection_mode: SelectionMode::Multi,
                striped: true,
                hoverable: true,
                row_height: RowHeight::Standard,
                selected_rows: selected_ids,
                on_row_click: on_row_click,
                on_cell_edit: on_cell_edit,
                loading: is_loading,
                skeleton: is_loading,
                skeleton_rows: 8,
                virtual_scroll: true,
                virtual_scroll_height: 500.0,
            }
        }
    }
}
