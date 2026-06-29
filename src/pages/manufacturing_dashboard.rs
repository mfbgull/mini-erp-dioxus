//! Manufacturing Dashboard Page — Overview of manufacturing operations with KPI cards,
//! quick actions, navigation links, and recent production runs.

use crate::components::common::{StatCard, StatCardVariant, StatTrend, TrendDirection};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct ProdRunSummary {
    prd_no: String,
    item_name: String,
    planned_qty: i32,
    completed_qty: i32,
    status: String,
}

#[derive(Clone, PartialEq)]
struct ManufacturingKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

fn kpi_data() -> Vec<ManufacturingKpi> {
    vec![
        ManufacturingKpi {
            title: "Active BOMs".to_string(),
            value: "24".to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Primary,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "3 new this month".to_string(),
            }),
            footer: Some("Across all product lines".to_string()),
        },
        ManufacturingKpi {
            title: "Production Orders".to_string(),
            value: "8".to_string(),
            icon: "⚙".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "2 active, 6 planned".to_string(),
            }),
            footer: Some("Next start: 28 Jun".to_string()),
        },
        ManufacturingKpi {
            title: "Completed This Month".to_string(),
            value: "12".to_string(),
            icon: "✅".to_string(),
            variant: StatCardVariant::Primary,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "33% vs last month".to_string(),
            }),
            footer: Some("Total units: 2,450".to_string()),
        },
        ManufacturingKpi {
            title: "Yield Rate".to_string(),
            value: "94.5%".to_string(),
            icon: "🎯".to_string(),
            variant: StatCardVariant::Success,
            trend: Some(StatTrend {
                direction: TrendDirection::Up,
                label: "0.8% improvement".to_string(),
            }),
            footer: Some("Target: ≥ 95%".to_string()),
        },
    ]
}

fn recent_runs() -> Vec<ProdRunSummary> {
    vec![
        ProdRunSummary {
            prd_no: "PRD-2026-0007".to_string(),
            item_name: "Premium Widget Alpha".to_string(),
            planned_qty: 500,
            completed_qty: 500,
            status: "Completed".to_string(),
        },
        ProdRunSummary {
            prd_no: "PRD-2026-0006".to_string(),
            item_name: "Steel Bracket XR-200".to_string(),
            planned_qty: 200,
            completed_qty: 185,
            status: "Completed".to_string(),
        },
        ProdRunSummary {
            prd_no: "PRD-2026-0008".to_string(),
            item_name: "Rubber Gasket Set".to_string(),
            planned_qty: 1000,
            completed_qty: 620,
            status: "In Progress".to_string(),
        },
        ProdRunSummary {
            prd_no: "PRD-2026-0009".to_string(),
            item_name: "Assembly Kit Type-B".to_string(),
            planned_qty: 300,
            completed_qty: 0,
            status: "Planned".to_string(),
        },
    ]
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Completed" => "badge badge-success",
        "In Progress" => "badge badge-primary",
        "Planned" => "badge badge-warning",
        "Cancelled" => "badge badge-danger",
        _ => "badge",
    }
}

const PAGE_CSS: &str = r##"
.page { padding: 20px; max-width: 1100px; margin: 0 auto; }
.page-header h1 { font-size: 22px; font-weight: 700; margin: 0; color: var(--text-primary); }
.page-subtitle { font-size: 13px; color: var(--text-secondary); margin: 4px 0 0 0; }
.dashboard-kpi-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 14px; margin: 20px 0; }
.dashboard-columns { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 20px; }
.dashboard-section { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); }
.dashboard-section-header { padding: 14px 18px; border-bottom: 1px solid var(--border-color, #e0e0e0); }
.dashboard-section-header h2 { font-size: 14px; font-weight: 600; margin: 0; }
.dashboard-section-body { padding: 14px 18px; }
.dashboard-actions { display: flex; flex-direction: column; gap: 8px; }
.toolbar-btn { display: flex; align-items: center; gap: 6px; padding: 8px 14px; border: 1px solid var(--border-color, #e0e0e0); border-radius: 6px; background: #fff; cursor: pointer; font-size: 13px; color: var(--text-primary); transition: all 0.15s ease; }
.toolbar-btn:hover { border-color: var(--accent, #4a90d9); background: rgba(74, 144, 217, 0.04); }
.toolbar-btn-primary { background: var(--accent, #4a90d9); color: #fff; border: none; }
.toolbar-btn-primary:hover { opacity: 0.9; }
.dashboard-section-wide { background: #fff; border: 1px solid var(--border-color, #e0e0e0); border-radius: var(--radius, 8px); margin-bottom: 20px; }
.recent-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.recent-table thead th { text-align: left; padding: 10px 14px; font-weight: 600; font-size: 11px; text-transform: uppercase; letter-spacing: 0.3px; color: var(--text-secondary); border-bottom: 2px solid var(--border-color, #e0e0e0); }
.recent-table tbody td { padding: 10px 14px; border-bottom: 1px solid var(--border-color, #e0e0e0); color: var(--text-primary); }
.recent-table tbody tr:last-child td { border-bottom: none; }
.recent-table tbody tr:hover { background: rgba(74, 144, 217, 0.03); }
.recent-table .text-right { text-align: right; }
.badge { display: inline-flex; padding: 3px 8px; border-radius: 10px; font-size: 11px; font-weight: 600; }
.badge-success { background: rgba(40, 167, 69, 0.1); color: #28a745; }
.badge-primary { background: rgba(74, 144, 217, 0.1); color: #4a90d9; }
.badge-warning { background: rgba(255, 193, 7, 0.15); color: #d4a017; }
.badge-danger { background: rgba(220, 53, 69, 0.1); color: #dc3545; }
@media (max-width: 768px) { .dashboard-columns { grid-template-columns: 1fr; } }
"##;

#[component]
pub fn ManufacturingDashboardPage() -> Element {
    let kpis = kpi_data();
    let runs = recent_runs();
    let navigator = use_navigator();

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Manufacturing Dashboard" }
                    p { class: "page-subtitle", "Overview of BOMs, production orders, and shop floor performance." }
                }
            }

            div { class: "dashboard-kpi-grid",
                {kpis.into_iter().map(|kpi| {
                    rsx! {
                        StatCard {
                            key: "{kpi.title}",
                            title: kpi.title,
                            value: kpi.value,
                            icon: kpi.icon,
                            variant: kpi.variant,
                            trend: kpi.trend,
                            footer: kpi.footer,
                        }
                    }
                })}
            }

            div { class: "dashboard-columns",
                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "⚡ Quick Actions" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn toolbar-btn-primary",
                                onclick: move |_| { navigator.push("/manufacturing/boms/new"); },
                                "＋ New BOM"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/manufacturing/production/new"); },
                                "＋ New Production Order"
                            }
                        }
                    }
                }

                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "🔗 Navigation" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/manufacturing/boms"); },
                                "📋 BOM List"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/manufacturing/production"); },
                                "⚙ Production Orders"
                            }
                        }
                    }
                }
            }

            div { class: "dashboard-section-wide",
                div { class: "dashboard-section-header",
                    h2 { "📊 Recent Production Runs" }
                }
                table { class: "recent-table",
                    thead {
                        tr {
                            th { "Production #" }
                            th { "Item" }
                            th { class: "text-right", "Planned Qty" }
                            th { class: "text-right", "Completed" }
                            th { "Status" }
                        }
                    }
                    tbody {
                        {runs.iter().map(|r| {
                            let status_class = status_badge_class(&r.status);
                            rsx! {
                                tr {
                                    td { "{r.prd_no}" }
                                    td { "{r.item_name}" }
                                    td { class: "text-right", "{r.planned_qty}" }
                                    td { class: "text-right", "{r.completed_qty}" }
                                    td {
                                        span { class: "{status_class}", "{r.status}" }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
