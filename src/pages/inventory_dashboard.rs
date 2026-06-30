//! Inventory Dashboard Page — Overview of inventory health with KPI cards,
//! low stock alerts, and quick actions.

use crate::auth::use_auth;
use crate::components::common::{StatCard, StatCardVariant, StatTrend};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
struct InventoryKpi {
    title: String,
    value: String,
    icon: String,
    variant: StatCardVariant,
    trend: Option<StatTrend>,
    footer: Option<String>,
}

#[component]
pub fn InventoryDashboardPage() -> Element {
    let navigator = use_navigator();
    let api = use_auth().api;

    let dashboard_resource = use_resource(move || {
        let api = api.clone();
        async move {
            let client = api.with(|c| c.clone());
            let summary = client.get_stock_summary().await.unwrap_or_default();
            let movement = client.get_stock_movement_summary().await.unwrap_or_default();
            let warehouses = client.list_warehouses().await.unwrap_or_default();
            let low_stock = client.list_low_stock_items().await.unwrap_or_default();
            (summary, movement, warehouses, low_stock)
        }
    });

    let is_loading = dashboard_resource.read().is_none();
    let (stock_summary, _movement, warehouses, low_stock) = dashboard_resource
        .read()
        .as_ref()
        .cloned()
        .unwrap_or_else(|| (vec![], serde_json::Value::Null, vec![], vec![]));

    let total_items: i64 = stock_summary.iter().map(|s| s["total_quantity"].as_f64().unwrap_or(0.0) as i64).sum();
    let stock_value: f64 = stock_summary.iter().map(|s| {
        let qty = s["total_quantity"].as_f64().unwrap_or(0.0);
        let cost = s["standard_cost"].as_f64().unwrap_or(0.0);
        qty * cost
    }).sum();
    let warehouse_count = warehouses.len();
    let low_stock_count = low_stock.len() as i64;

    let format_pkru = |amount: f64| -> String {
        let formatted = amount as u64;
        let s = formatted.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        let rev: String = result.chars().rev().collect();
        format!("PKR {}", rev)
    };

    let kpis = vec![
        InventoryKpi {
            title: "Total Items".to_string(),
            value: total_items.to_string(),
            icon: "📦".to_string(),
            variant: StatCardVariant::Primary,
            trend: None,
            footer: Some(format!("{} warehouses", warehouse_count)),
        },
        InventoryKpi {
            title: "Stock Value".to_string(),
            value: format_pkru(stock_value),
            icon: "💰".to_string(),
            variant: StatCardVariant::Success,
            trend: None,
            footer: Some("At standard cost".to_string()),
        },
        InventoryKpi {
            title: "Low Stock Items".to_string(),
            value: low_stock_count.to_string(),
            icon: "⚠".to_string(),
            variant: StatCardVariant::Danger,
            trend: None,
            footer: None,
        },
        InventoryKpi {
            title: "Warehouses".to_string(),
            value: warehouse_count.to_string(),
            icon: "🏭".to_string(),
            variant: StatCardVariant::Default,
            trend: None,
            footer: None,
        },
    ];

    rsx! {
        div { class: "page",
            div { class: "page-header",
                div {
                    h1 { "Inventory Dashboard" }
                    p { class: "page-subtitle", "Overview of your inventory health and stock levels." }
                }
            }

            if is_loading {
                div { class: "dashboard-kpi-grid",
                    { (0..4).map(|_| rsx! {
                        StatCard {
                            title: "Loading...".to_string(),
                            value: "--".to_string(),
                            icon: "⏳".to_string(),
                            variant: StatCardVariant::Default,
                        }
                    })}
                }
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
            }

            div { class: "dashboard-columns",
                div { class: "dashboard-section",
                    div { class: "dashboard-section-header",
                        h2 { "📦 Quick Actions" }
                    }
                    div { class: "dashboard-section-body",
                        div { class: "dashboard-actions",
                            button {
                                class: "toolbar-btn toolbar-btn-primary",
                                onclick: move |_| { navigator.push("/inventory/items/new"); },
                                "＋ New Item"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/warehouses/new"); },
                                "＋ New Warehouse"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/stock-movements/new"); },
                                "＋ Stock Movement"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/physical-counts/new"); },
                                "＋ Physical Count"
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
                                onclick: move |_| { navigator.push("/inventory/items"); },
                                "📋 Item List"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/warehouses"); },
                                "🏭 Warehouses"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/stock-movements"); },
                                "📊 Stock Movements"
                            }
                            button {
                                class: "toolbar-btn",
                                onclick: move |_| { navigator.push("/inventory/physical-counts"); },
                                "📝 Physical Counts"
                            }
                        }
                    }
                }
            }
        }
    }
}
