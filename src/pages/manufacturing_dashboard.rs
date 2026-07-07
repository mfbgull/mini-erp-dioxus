//! Manufacturing Dashboard Page — Overview of manufacturing operations with KPI cards,
//! quick actions, navigation links, and recent production runs.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant, StatTrend};
use crate::models::{Bom, Production};
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
.loading-text { text-align: center; padding: 40px; color: var(--text-secondary); font-size: 13px; }
@media (max-width: 768px) { .dashboard-columns { grid-template-columns: 1fr; } }
"##;

#[component]
pub fn ManufacturingDashboardPage() -> Element {
    let api = use_auth().api;
    let navigator = use_navigator();

    let prod_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.get_production_status().await.ok()
        }
    });

    let boms_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_boms().await.ok().unwrap_or_default()
        }
    });

    let orders_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            client.list_production_orders().await.ok().unwrap_or_default()
        }
    });

    let is_loading = prod_resource.read().is_none();
    let prod_data = prod_resource.read().as_ref().cloned().flatten();
    let boms: Vec<Bom> = boms_resource.read().as_ref().cloned().unwrap_or_default();
    let orders: Vec<Production> = orders_resource.read().as_ref().cloned().unwrap_or_default();

    let active_boms = boms.len();
    let total_orders = orders.len();
    let active_orders = orders.iter().filter(|o| o.status == "In Progress").count();
    let planned_orders = orders.iter().filter(|o| o.status == "Planned").count();
    let completed_orders = orders.iter().filter(|o| o.status == "Completed").count();
    let total_completed_qty: f64 = orders.iter().filter(|o| o.status == "Completed").map(|o| o.output_quantity).sum();

    let yield_rate = prod_data
        .as_ref()
        .and_then(|d| d.get("yield_rate"))
        .and_then(|v| v.as_f64())
        .map(|v| format!("{:.1}%", v))
        .unwrap_or_else(|| "N/A".to_string());

    let kpis = vec![
        ManufacturingKpi {
            title: "Active BOMs".to_string(),
            value: active_boms.to_string(),
            icon: "📋".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some("Across all product lines".to_string()),
        },
        ManufacturingKpi {
            title: "Production Orders".to_string(),
            value: total_orders.to_string(),
            icon: "⚙".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some(format!("{} active, {} planned", active_orders, planned_orders)),
        },
        ManufacturingKpi {
            title: "Completed This Month".to_string(),
            value: completed_orders.to_string(),
            icon: "✅".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some(format!("Total units: {:.0}", total_completed_qty)),
        },
        ManufacturingKpi {
            title: "Yield Rate".to_string(),
            value: yield_rate,
            icon: "🎯".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some("Target: ≥ 95%".to_string()),
        },
    ];

    let runs: Vec<ProdRunSummary> = orders.iter().take(10).map(|o| ProdRunSummary {
        prd_no: o.production_no.clone(),
        item_name: o.output_item_name.clone().unwrap_or_default(),
        planned_qty: o.output_quantity as i32,
        completed_qty: 0,
        status: o.status.clone(),
    }).collect();

    rsx! {
        style { "{PAGE_CSS}" }

        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Manufacturing Dashboard" }
                    p { class: "page-subtitle", "Overview of BOMs, production orders, and shop floor performance." }
                }
            }

            if is_loading {
                div { class: "loading-text", "Loading manufacturing data…" }
            } else {
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
                    if runs.is_empty() {
                        div { class: "loading-text", "No production runs found." }
                    } else {
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
    }
}
