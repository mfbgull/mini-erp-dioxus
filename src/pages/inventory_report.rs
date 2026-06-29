//! Inventory Report Page — Stock value, warehouse breakdown, and category analysis.

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
// Mock Data
// ============================================================================

fn warehouse_values() -> Vec<WarehouseValue> {
    vec![
        WarehouseValue { name: "Main Warehouse".to_string(), items: 45, value: 185_000.0 },
        WarehouseValue { name: "Raw Materials Store".to_string(), items: 28, value: 98_500.0 },
        WarehouseValue { name: "Equipment Storage".to_string(), items: 15, value: 320_000.0 },
    ]
}

fn category_items() -> Vec<CategoryItem> {
    vec![
        CategoryItem { category: "Widgets".to_string(), items: 12, value: 145_000.0 },
        CategoryItem { category: "Fasteners".to_string(), items: 8, value: 62_000.0 },
        CategoryItem { category: "Electrical".to_string(), items: 10, value: 98_000.0 },
        CategoryItem { category: "Raw Materials".to_string(), items: 25, value: 110_000.0 },
        CategoryItem { category: "Consumables".to_string(), items: 40, value: 45_000.0 },
        CategoryItem { category: "Safety".to_string(), items: 6, value: 28_500.0 },
        CategoryItem { category: "Packaging".to_string(), items: 18, value: 15_000.0 },
        CategoryItem { category: "Equipment".to_string(), items: 5, value: 300_000.0 },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn InventoryReportPage() -> Element {
    let toast = use_toast();
    let wh_values = warehouse_values();
    let cat_items = category_items();
    let total_items: i32 = cat_items.iter().map(|c| c.items).sum();
    let total_value: f64 = cat_items.iter().map(|c| c.value).sum();
    let low_stock = 4;
    let categories = cat_items.len();

    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Inventory report will be exported as PDF."); }
    };

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
                    footer: Some("2 critically low".to_string()),
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
