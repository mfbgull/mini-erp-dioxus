//! Reports Dashboard Page — Overview of available reports with navigation cards.

use crate::components::common::{StatCard, StatCardVariant};
use dioxus::prelude::*;

// ============================================================================
// Constants & CSS
// ============================================================================

const PAGE_CSS: &str = r##"
.reports-page { max-width: 1000px; margin: 0 auto; }
.reports-header { margin-bottom: 16px; }
.reports-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }

.reports-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 12px; margin-bottom: 20px; }

.reports-filter-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 12px 16px; }
.reports-filter-bar label { font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.reports-filter-bar input[type="date"] { border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; padding: 6px 10px; font-size: 13px; background: #fff; }
.reports-filter-bar .filter-label { font-size: 12px; font-weight: 600; color: var(--text-secondary); margin-right: 4px; }

.reports-nav-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; margin-bottom: 20px; }

.report-card { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); padding: 18px; cursor: pointer; transition: all 0.15s ease; display: flex; flex-direction: column; gap: 8px; }
.report-card:hover { border-color: var(--accent, #4a90d9); box-shadow: 0 2px 8px rgba(74,144,217,0.12); }
.report-card-icon { font-size: 28px; line-height: 1; }
.report-card-title { font-size: 14px; font-weight: 600; color: var(--text-primary); margin: 0; }
.report-card-desc { font-size: 12px; color: var(--text-secondary); line-height: 1.5; margin: 0; }
.report-card-arrow { font-size: 16px; color: var(--accent, #4a90d9); align-self: flex-end; margin-top: auto; }

@media (max-width: 768px) {
    .reports-kpi-grid { grid-template-columns: 1fr 1fr; }
    .reports-nav-grid { grid-template-columns: 1fr; }
}
"##;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, PartialEq)]
struct ReportNavItem {
    icon: String,
    title: String,
    description: String,
    route: &'static str,
}

fn report_items() -> Vec<ReportNavItem> {
    vec![
        ReportNavItem {
            icon: "📅".to_string(),
            title: "AR Aging".to_string(),
            description: "View outstanding receivables by aging buckets — 0-30, 31-60, 61-90, 90+ days.".to_string(),
            route: "/reports/ar-aging",
        },
        ReportNavItem {
            icon: "👤".to_string(),
            title: "Customer Statements".to_string(),
            description: "Generate and export detailed customer account statements with running balance.".to_string(),
            route: "/reports/customer-statements",
        },
        ReportNavItem {
            icon: "💰".to_string(),
            title: "Sales Report".to_string(),
            description: "Monthly sales performance with KPIs, category breakdown, and trend analysis.".to_string(),
            route: "/reports/sales",
        },
        ReportNavItem {
            icon: "📦".to_string(),
            title: "Inventory Report".to_string(),
            description: "Stock value by warehouse, category breakdown, and low stock analysis.".to_string(),
            route: "/reports/inventory",
        },
        ReportNavItem {
            icon: "📊".to_string(),
            title: "Financial Report".to_string(),
            description: "Profit & Loss statement and Balance Sheet with period comparison.".to_string(),
            route: "/reports/financial",
        },
        ReportNavItem {
            icon: "🧾".to_string(),
            title: "Tax Summary".to_string(),
            description: "Sales Tax, Income Tax, and Withholding Tax summaries by period.".to_string(),
            route: "/reports/tax",
        },
        ReportNavItem {
            icon: "🔧".to_string(),
            title: "Custom Report Builder".to_string(),
            description: "Create custom reports with selected fields, filters, and grouping options.".to_string(),
            route: "/reports/custom",
        },
    ]
}

// ============================================================================
// Component
// ============================================================================

#[component]
pub fn ReportsDashboardPage() -> Element {
    let navigator = use_navigator();
    let reports = report_items();

    rsx! {
        style { "{PAGE_CSS}" }
        div { class: "page reports-page",

            div { class: "reports-header",
                h1 { "Reports Dashboard" }
                p { class: "page-subtitle", "Generate and view business reports across all modules." }
            }

            div { class: "reports-kpi-grid",
                StatCard {
                    title: "Available Reports".to_string(),
                    value: "7".to_string(),
                    icon: Some("📈".to_string()),
                    variant: StatCardVariant::Primary,
                    footer: Some("Across all modules".to_string()),
                }
                StatCard {
                    title: "Last Generated".to_string(),
                    value: "Sales Report".to_string(),
                    icon: Some("🕐".to_string()),
                    variant: StatCardVariant::Default,
                    footer: Some("Today, 9:30 AM".to_string()),
                }
            }

            // Quick date range filter
            div { class: "reports-filter-bar",
                span { class: "filter-label", "Quick Date Range:" }
                label { "From" }
                input { r#type: "date", value: "2026-01-01" }
                label { "To" }
                input { r#type: "date", value: "2026-06-27" }
                span { style: "color: var(--text-secondary); font-size: 12px; margin-left: 8px;", "Default filter applies to report data" }
            }

            // Navigation cards grid
            div { class: "reports-nav-grid",
                {reports.into_iter().map(|r| {
                    let nav = navigator.clone();
                    let route = r.route;
                    rsx! {
                        div {
                            key: "{r.title}",
                            class: "report-card",
                            onclick: move |_| { nav.push(route); },
                            div { class: "report-card-icon", "{r.icon}" }
                            h3 { class: "report-card-title", "{r.title}" }
                            p { class: "report-card-desc", "{r.description}" }
                            div { class: "report-card-arrow", "→" }
                        }
                    }
                })}
            }
        }
    }
}
