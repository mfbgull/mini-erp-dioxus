//! Sales Report Page — Monthly sales performance with KPIs, chart, and category breakdown.

use crate::auth::use_auth;
use crate::components::common::{Button, ButtonVariant, StatCard, StatCardVariant, StatTrend, TrendDirection, use_toast};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.sr-page { max-width: 1000px; margin: 0 auto; }
.sr-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; flex-wrap: wrap; gap: 12px; }
.sr-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.sr-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.sr-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.sr-filter-bar select { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }

.sr-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.sr-chart-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 20px; }
.sr-chart-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sr-chart-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }
.sr-chart { width: 100%; height: 200px; }
.sr-chart svg { width: 100%; height: 100%; }
.sr-chart-bar { fill: var(--accent, #4a90d9); transition: fill 0.15s ease; cursor: pointer; }
.sr-chart-bar:hover { fill: #357abd; }
.sr-chart-bar-label { font-size: 10px; fill: var(--text-secondary, #6c757d); text-anchor: middle; }
.sr-chart-bar-value { font-size: 9px; fill: var(--text-secondary, #6c757d); text-anchor: middle; font-weight: 600; }

.sr-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 16px; margin-bottom: 20px; }
.sr-section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.sr-section-header h2 { font-size: 15px; font-weight: 600; color: var(--text-primary); margin: 0; }

.sr-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.sr-table thead th { text-align: left; padding: 8px 10px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); white-space: nowrap; }
.sr-table thead th.text-right { text-align: right; }
.sr-table tbody td { padding: 8px 10px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.sr-table tbody td.text-right { text-align: right; font-family: monospace; font-size: 12px; }
.sr-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }

.sr-actions { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }

@media (max-width: 768px) {
    .sr-kpi-grid { grid-template-columns: 1fr 1fr; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, PartialEq)]
struct MonthlySale {
    month: String,
    amount: f64,
}

#[derive(Clone)]
struct CategorySale {
    category: String,
    amount: f64,
    percentage: f64,
}

// ============================================================================
// Helpers — parse API JSON into view structs
// ============================================================================

fn parse_monthly_sales(data: &serde_json::Value) -> Vec<MonthlySale> {
    data.get("monthly").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().map(|m| MonthlySale {
            month: m.get("month").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            amount: m.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }).collect()
}

fn parse_category_sales(data: &serde_json::Value) -> Vec<CategorySale> {
    data.get("categories").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().map(|c| CategorySale {
            category: c.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            amount: c.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
            percentage: c.get("percentage").and_then(|v| v.as_f64()).unwrap_or(0.0),
        }).collect()
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn SalesReportPage() -> Element {
    let toast = use_toast();
    let api = use_auth().api;

    let summary_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_sales_summary_report().await.unwrap_or_default()
        }
    });

    let by_customer_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_sales_by_customer().await.unwrap_or_default()
        }
    });

    let by_item_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_sales_by_item().await.unwrap_or_default()
        }
    });

    let loading = summary_resource.read().is_none()
        || by_customer_resource.read().is_none()
        || by_item_resource.read().is_none();

    let summary_data = summary_resource.read().clone().unwrap_or_default();
    let by_customer_data = by_customer_resource.read().clone().unwrap_or_default();
    let by_item_data = by_item_resource.read().clone().unwrap_or_default();

    let monthly = parse_monthly_sales(&summary_data);
    let categories = parse_category_sales(&by_item_data);
    let total_sales: f64 = summary_data.get("total_sales").and_then(|v| v.as_f64()).unwrap_or(
        monthly.iter().map(|m| m.amount).sum()
    );
    let total_invoices = summary_data.get("total_invoices").and_then(|v| v.as_i64()).unwrap_or(0) as f64;
    let avg_invoice = if total_invoices > 0.0 { total_sales / total_invoices } else { 0.0 };
    let unpaid_count = summary_data.get("unpaid_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let top_customer = by_customer_data.get("top_customer").and_then(|v| v.as_str()).unwrap_or("N/A").to_string();
    let top_customer_amount = by_customer_data.get("top_customer_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let max_amount = monthly.iter().map(|m| m.amount).fold(0.0_f64, f64::max);
    let bar_count = monthly.len();
    let bar_width_pct = if bar_count > 0 { 100.0 / bar_count as f64 } else { 100.0 };
    let bar_gap_pct = bar_width_pct * 0.25;
    let bar_inner_pct = bar_width_pct - bar_gap_pct;
    let chart_height = 180.0;

    let on_export = {
        let mut t = toast.clone();
        move |_| { t.info("Export", "Sales report will be exported as PDF."); }
    };

    if loading {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sr-page",
                div { class: "sr-header",
                    div {
                        h1 { "Sales Report" }
                        p { class: "page-subtitle", "Monthly sales performance analysis." }
                    }
                }
                div { class: "sr-loading", "Loading sales data..." }
            }
        }
    } else {
        rsx! {
            style { "{PAGE_CSS}" }
            div { class: "page sr-page",

                div { class: "sr-header",
                    div {
                        h1 { "Sales Report" }
                        p { class: "page-subtitle", "Monthly sales performance analysis." }
                    }
                    Button { variant: ButtonVariant::Primary, icon: Some("📥".to_string()), onclick: on_export, "Export Report" }
                }

                // Filter
                div { class: "sr-filter-bar",
                    label { "Year" }
                    select {
                        option { value: "2026", selected: true, "2026" }
                        option { value: "2025", "2025" }
                    }
                    label { "Period" }
                    select {
                        option { value: "monthly", selected: true, "Monthly" }
                        option { value: "quarterly", "Quarterly" }
                        option { value: "yearly", "Yearly" }
                    }
                    label { "Customer" }
                    select {
                        option { value: "all", selected: true, "All Customers" }
                        option { value: "alpha", "Alpha Traders" }
                        option { value: "beta", "Beta Industries" }
                    }
                }

                // KPI cards
                div { class: "sr-kpi-grid",
                    StatCard {
                        title: "Total Sales".to_string(),
                        value: format!("PKR {:.0}", total_sales),
                        icon: "💰".to_string(),
                        variant: StatCardVariant::Primary,
                        trend: Some(StatTrend { direction: TrendDirection::Up, label: "12.4% vs last period".to_string() }),
                        footer: Some("H1 2026".to_string()),
                    }
                    StatCard {
                        title: "Invoices".to_string(),
                        value: format!("{}", total_invoices as i64),
                        icon: "🧾".to_string(),
                        variant: StatCardVariant::Success,
                        trend: Some(StatTrend { direction: TrendDirection::Up, label: "8 more than last period".to_string() }),
                        footer: Some(format!("{} unpaid", unpaid_count)),
                    }
                    StatCard {
                        title: "Avg Invoice Value".to_string(),
                        value: format!("PKR {:.0}", avg_invoice),
                        icon: "📊".to_string(),
                        variant: StatCardVariant::Default,
                        footer: Some(format!("Across {} invoices", total_invoices as i64)),
                    }
                    StatCard {
                        title: "Top Customer".to_string(),
                        value: top_customer,
                        icon: "🏆".to_string(),
                        variant: StatCardVariant::Warning,
                        footer: Some(format!("PKR {:.0} total", top_customer_amount)),
                    }
                }

                // Chart section
                div { class: "sr-chart-section",
                    div { class: "sr-chart-header",
                        h2 { "📈 Monthly Sales (H1 2026)" }
                    }
                    div { class: "sr-chart",
                        svg {
                            view_box: "0 0 100 200",
                            preserve_aspect_ratio: "xMidYMid meet",
                            line { x1: "0", y1: "20", x2: "100", y2: "20", stroke: "#f0f0f0", stroke_width: "0.5" }
                            line { x1: "0", y1: "60", x2: "100", y2: "60", stroke: "#f0f0f0", stroke_width: "0.5" }
                            line { x1: "0", y1: "100", x2: "100", y2: "100", stroke: "#f0f0f0", stroke_width: "0.5" }
                            line { x1: "0", y1: "140", x2: "100", y2: "140", stroke: "#f0f0f0", stroke_width: "0.5" }

                            {monthly.into_iter().enumerate().map(|(i, m)| {
                                let bar_height = if max_amount > 0.0 { (m.amount / max_amount) * chart_height } else { 0.0 };
                                let x = i as f64 * bar_width_pct + bar_gap_pct / 2.0;
                                let y = 190.0 - bar_height;
                                rsx! {
                                    rect {
                                        key: "{i}",
                                        class: "sr-chart-bar",
                                        x: "{x:.1}",
                                        y: "{y:.1}",
                                        width: "{bar_inner_pct:.1}",
                                        height: "{bar_height:.1}",
                                        rx: "2",
                                    }
                                    text {
                                        class: "sr-chart-bar-value",
                                        x: "{x + bar_inner_pct / 2.0:.1}",
                                        y: "{y - 4.0:.1}",
                                        "{m.amount:.0}"
                                    }
                                    text {
                                        class: "sr-chart-bar-label",
                                        x: "{x + bar_inner_pct / 2.0:.1}",
                                        y: "198.0",
                                        "{m.month}"
                                    }
                                }
                            })}
                        }
                    }
                }

                // Category breakdown
                div { class: "sr-section",
                    div { class: "sr-section-header",
                        h2 { "📊 Sales by Category" }
                    }
                    table { class: "sr-table",
                        thead { tr {
                            th { "Category" } th { class: "text-right", "Amount (PKR)" }
                            th { class: "text-right", "% of Total" }
                        }}
                        tbody {
                            {categories.into_iter().map(|c| {
                                rsx! {
                                    tr {
                                        td { "{c.category}" }
                                        td { class: "text-right", "PKR {c.amount:.0}" }
                                        td { class: "text-right", "{c.percentage:.1}%" }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}
