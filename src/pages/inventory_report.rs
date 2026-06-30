//! Inventory Report Page — Stock value, warehouse breakdown, and category analysis.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, use_toast};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.ir-page { max-width: 1000px; margin: 0 auto; }
.ir-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.ir-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.ir-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.ir-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.ir-filter-bar input[type="date"], .ir-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.ir-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.ir-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px; }
@media (max-width: 768px) { .ir-columns { grid-template-columns: 1fr; } }

.ir-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; }
.ir-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.ir-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.ir-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.ir-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.ir-table thead th.text-right { text-align: right; }
.ir-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.ir-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.ir-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.ir-table tfoot td { padding: 8px 10px; font-weight: 700; border-top: 2px solid var(--border-color, #e0e0e0); }
.ir-table tfoot td.text-right { text-align: right; font-family: monospace; }

.ir-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone)]
struct WarehouseValue {
    name: String,
    items: i32,
    value: f64,
}

#[derive(Clone)]
struct CategoryItem {
    category: String,
    items: i32,
    value: f64,
}

// ============================================================================
// Helpers — parse API JSON into view structs
// ============================================================================

fn parse_warehouse_values(data: &serde_json::Value) -> Vec<WarehouseValue> {
    data.get("warehouses").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().map(|w| WarehouseValue {
            name: w.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            items: w.get("items").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            value: w.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }).collect()
}

fn parse_category_items(data: &serde_json::Value) -> Vec<CategoryItem> {
    data.get("categories").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().map(|c| CategoryItem {
            category: c.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            items: c.get("items").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            value: c.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }).collect()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn InventoryReportPage() -> Element {
    let toast = use_toast();
    let api = use_auth().api;

    let stock_level_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_stock_level().await.unwrap_or_default()
        }
    });

    let valuation_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_stock_valuation().await.unwrap_or_default()
        }
    });

    let low_stock_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_low_stock_report().await.unwrap_or_default()
        }
    });

    let loading = stock_level_resource.read().is_none()
        || valuation_resource.read().is_none()
        || low_stock_resource.read().is_none();

    let stock_data = stock_level_resource.read().clone().unwrap_or_default();
    let valuation_data = valuation_resource.read().clone().unwrap_or_default();
    let low_stock_data = low_stock_resource.read().clone().unwrap_or_default();

    let wh_values = parse_warehouse_values(&stock_data);
    let cat_items = parse_category_items(&valuation_data);
    let total_items: i32 = cat_items.iter().map(|c| c.items).sum();
    let total_value: f64 = cat_items.iter().map(|c| c.value).sum();
    let low_stock = low_stock_data.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let categories = cat_items.len();

    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Inventory report will be exported as PDF."); }
    };

    if loading {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page ir-page",
                div { class: "ir-header",
                    div {
                        h1 { "Inventory Report" }
                        p { class: "page-subtitle", "Stock value, warehouse breakdown, and category analysis." }
                    }
                }
                div { class: "ir-loading", "Loading inventory data..." }
            }
        }
    } else {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page ir-page",

                div { class: "ir-header",
                    div {
                        h1 { "Inventory Report" }
                        p { class: "page-subtitle", "Stock value, warehouse breakdown, and category analysis." }
                    }
                    Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export Report" }
                }

                // Filter
                div { class: "ir-filter-bar",
                    label { "As of Date" }
                    input { r#type: "date", value: "2026-06-27" }
                    label { "Warehouse" }
                    select {
                        option { value: "all", selected: true, "All Warehouses" }
                        option { value: "main", "Main Warehouse" }
                        option { value: "raw", "Raw Materials Store" }
                    }
                }

                // KPI cards
                div { class: "ir-kpi-grid",
                    StatCard {
                        title: "Total Items".to_string(),
                        value: format!("{}", total_items),
                        icon: "📦".to_string(),
                        variant: StatCardVariant::Primary,
                        footer: Some(format!("Across {} categories", categories)),
                    }
                    StatCard {
                        title: "Total Stock Value".to_string(),
                        value: format!("PKR {:.0}", total_value),
                        icon: "💰".to_string(),
                        variant: StatCardVariant::Success,
                        footer: Some("At standard cost".to_string()),
                    }
                    StatCard {
                        title: "Low Stock Items".to_string(),
                        value: format!("{}", low_stock),
                        icon: "⚠".to_string(),
                        variant: StatCardVariant::Danger,
                        footer: Some(format!("{} critically low", low_stock_data.get("critical").and_then(|v| v.as_i64()).unwrap_or(0))),
                    }
                    StatCard {
                        title: "Categories".to_string(),
                        value: format!("{}", categories),
                        icon: "🏷".to_string(),
                        variant: StatCardVariant::Default,
                        footer: Some("Active categories".to_string()),
                    }
                }

                // Two-column: warehouse value + category breakdown
                div { class: "ir-columns",

                    // Warehouse value
                    div { class: "ir-section",
                        div { class: "ir-section-header",
                            h2 { "🏭 Stock Value by Warehouse" }
                        }
                        table { class: "ir-table",
                            thead { tr {
                                th { "Warehouse" } th { class: "text-right", "Items" }
                                th { class: "text-right", "Value (PKR)" }
                            }}
                            tbody {
                                {wh_values.into_iter().map(|w| {
                                    rsx! {
                                        tr {
                                            td { "{w.name}" }
                                            td { class: "text-right", "{w.items}" }
                                            td { class: "text-right", "PKR {w.value:.0}" }
                                        }
                                    }
                                })}
                            }
                            tfoot { tr {
                                td { "Total" }
                                td { class: "text-right", "{total_items}" }
                                td { class: "text-right", "PKR {total_value:.0}" }
                            }}
                        }
                    }

                    // Category breakdown
                    div { class: "ir-section",
                        div { class: "ir-section-header",
                            h2 { "🏷 Items by Category" }
                        }
                        table { class: "ir-table",
                            thead { tr {
                                th { "Category" } th { class: "text-right", "Items" }
                                th { class: "text-right", "Value (PKR)" }
                            }}
                            tbody {
                                {cat_items.into_iter().map(|c| {
                                    rsx! {
                                        tr {
                                            td { "{c.category}" }
                                            td { class: "text-right", "{c.items}" }
                                            td { class: "text-right", "PKR {c.value:.0}" }
                                        }
                                    }
                                })}
                            }
                            tfoot { tr {
                                td { "Total" }
                                td { class: "text-right", "{total_items}" }
                                td { class: "text-right", "PKR {total_value:.0}" }
                            }}
                        }
                    }
                }
            }
        }
    }
}
